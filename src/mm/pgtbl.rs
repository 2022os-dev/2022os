use super::address::*;
use super::pte_sv39::{PTEFlag, PTE};
use crate::config::*;
use crate::mm::MemorySpace;
use crate::mm::memory_space::Segments;
use core::mem::size_of;
use core::ops::Range;

use super::kalloc::KALLOCATOR;

pub struct Pgtbl {
    pub root: PageNum,
}

impl Pgtbl {
    pub fn new() -> Self {
        let page = KALLOCATOR.lock().kalloc();
        log!("pgtbl":"new">"page(0x{:x})", page.page());
        Self { root: page }
    }

    pub fn walk(&mut self, va: VirtualAddr, do_alloc: bool) -> &mut PTE {
        let page: PageNum = va.floor();
        let mut ppn = self.root;
        #[allow(unused_assignments)]
        let mut pte: &mut PTE = ppn.offset_phys(0).as_mut();
        // 固定解析三级页表，不支持巨页
        for level in (1..PAGE_TABLE_LEVEL).rev() {
            let mut physpte = ppn.offset_phys(page.vpn_block_sv39(level) * size_of::<usize>());
            pte = physpte.as_mut();
            if pte.is_valid() {
                if pte.is_leaf() {
                    self.print();
                    panic!("too short page table, level({}), va(0x{:x}), ppn(0x{:x})", level, va.0, ppn.page());
                }
                ppn = pte.ppn();
            } else {
                if do_alloc {
                    let page = KALLOCATOR.lock().kalloc();
                    page.offset_phys(0).write_bytes(0, PAGE_SIZE);
                    pte.set_ppn(page);
                    pte.set_flags(PTEFlag::V);
                    ppn = page;
                } else {
                    // panic!("walk invalid 0x{:x}", va.0)
                }
            }
        }
        unsafe {
            (ppn.offset(page.vpn_block_sv39(0) * size_of::<usize>()).0 as *mut PTE)
                .as_mut()
                .unwrap()
        }
    }

    pub fn map_pages(&mut self, pages: Range<PageNum>, mut start: PageNum, flags: PTEFlag) {
        let start_num = pages.start;
        let end_num = pages.end;
        (start_num.page()..end_num.page())
            .map(|page| {
                self.map(Into::<PageNum>::into(page).into(), start, flags);
                start = start + 1;
                0
            })
            .count();
    }

    pub fn map(&mut self, vpage: PageNum, page: PageNum, flags: PTEFlag) {
        // log!("pgtbl":"map">"vpate(0x{:x}) -> page(0x{:x}) {:?}", vpage.page(), page.page(), flags);
        let pte = self.walk(vpage.offset(0), true);
        if pte.is_valid() {
            log!("pgtbl":"map""warn"> "remap page 0x{:x} -> 0x{:x}", vpage.page(), page.page());
        }
        pte.set_ppn(page);
        pte.set_flags(flags | PTEFlag::V);
    }

    fn _unmap_page_table(ppn: PageNum, addr: usize) {
        for idx in 0..(PAGE_SIZE / size_of::<usize>()) {
            let mut physpte = ppn.offset_phys(idx * size_of::<usize>());
            let pte: &mut PTE = physpte.as_mut();
            if pte.is_valid() {
                if pte.is_leaf() {
                    log!("pgtbl":"_unmap_page_table""warn"> "unmaping valid leaf page 0x{:x}", ((addr << SV39_VPN_BIT) + idx) << PAGE_OFFSET_BIT);
                } else {
                    Self::_unmap_page_table(pte.ppn(), (addr << SV39_VPN_BIT) + idx);
                }
            }
        }
        KALLOCATOR.lock().kfree(ppn);
    }

    #[allow(unused)]
    pub fn unmap_page_table(&mut self) {
        Self::_unmap_page_table(self.root, 0);
    }

    #[allow(unused)]
    pub fn unmap_pages(&mut self, vpages: Range<PageNum>, do_free: bool) {
        for page in vpages.start.page()..vpages.end.page() {
            self.unmap(page.into(), do_free);
        }
    }

    // 不会报错当尝试两次unmap同一个页，因为memory_space的unmap_segments需要
    pub fn unmap(&mut self, vpage: PageNum, do_free: bool) {
        // Fixme: when unmap an invalid page
        let pte = self.walk(vpage.offset(0), false);
        if do_free && pte.is_valid() {
            KALLOCATOR.lock().kfree(pte.ppn());
        }
        pte.set_flags(PTEFlag::empty());
    }

    #[allow(unused)]
    fn copy_page(from: PageNum, to: PageNum, alloc_mem: bool) {
        // Fixme: 不用搜索整个空间，只复制用户空间和trapframe,trampoline
        for idx in 0..(PAGE_SIZE / size_of::<usize>()) {
            let mut physpte = from.offset_phys(idx * size_of::<usize>());
            let pte: &mut PTE = physpte.as_mut();
            if pte.is_valid() {
                let mut child_phys = to.offset_phys(idx * size_of::<usize>());
                let child_pte: &mut PTE = child_phys.as_mut();
                child_pte.set_flags(pte.flags());

                if pte.is_leaf() {
                    if alloc_mem {
                        let child_page = KALLOCATOR.lock().kalloc();
                        child_pte.set_ppn(child_page);
                        child_page
                            .offset_phys(0)
                            .write(pte.ppn().offset_phys(0).as_slice(PAGE_SIZE));
                    } else {
                        child_pte.set_ppn(pte.ppn());
                    }
                } else {
                    child_pte.set_ppn(KALLOCATOR.lock().kalloc());
                    Self::copy_page(pte.ppn(), child_pte.ppn(), alloc_mem)
                }
            }
        }
    }

    #[allow(unused)]
    pub fn copy(&self, alloc_mem: bool) -> Self {
        let child = Pgtbl::new();
        Self::copy_page(self.root, child.root, alloc_mem);
        child
    }

    pub fn get_satp(&self) -> usize {
        self.root.page() | 0x8000000000000000
    }

    fn _print(&self, ppn: PageNum, addr: usize, level: usize) {
        for idx in 0..(PAGE_SIZE / size_of::<usize>()) {
            let mut physpte = ppn.offset_phys(idx * size_of::<usize>());
            let pte: &mut PTE = physpte.as_mut();
            if pte.is_valid() {
                if pte.is_leaf() {
                    if level == 1 {
                        log!(debug ". addr 0x{:x}, ppn 0x{:x}", ((addr << SV39_VPN_BIT) + idx) << PAGE_OFFSET_BIT, pte.ppn().page());
                    } else if level == 2 {
                        log!(debug ".. addr 0x{:x}, ppn 0x{:x}", ((addr << SV39_VPN_BIT) + idx) << PAGE_OFFSET_BIT, pte.ppn().page());
                    } else if level == 3 {
                        log!(debug "... addr 0x{:x}, ppn 0x{:x}", ((addr << SV39_VPN_BIT) + idx) << PAGE_OFFSET_BIT, pte.ppn().page());
                    }
                } else {
                    self._print(pte.ppn(), (addr << SV39_VPN_BIT) + idx, level + 1);
                }
            }
        }
    }

    pub fn print(&self) {
        self._print(self.root, 0, 1);
    }

    pub fn map_trampoline(&mut self) {
        let page = MemorySpace::trampoline_page();
        let pn = KALLOCATOR.lock().kalloc();
        self.map(page, pn, PTEFlag::R | PTEFlag::X | PTEFlag::V);
        pn.offset_phys(0).write(unsafe {
            core::slice::from_raw_parts(
                crate::trap::__alltraps as *const u8,
                crate::trap::trampoline as usize - crate::trap::__alltraps as usize,
            )
        });
    }

    pub fn unmap_segments(&mut self, segments: &Segments, do_free: bool) {
        for (virt, (_, _)) in segments.iter() {
            log!(debug "unmap seg 0x{:x}", virt.page());
            log!("pgtbl":"unmap_segments"> "vpage 0x{:x} -> 0x{:x}", virt.page(), phys.page());
            self.unmap(*virt, do_free);
        }
    }

    pub fn map_segments(&mut self, segments: &Segments) {
        for (virt, (phys, flags)) in segments.iter() {
            log!("pgtbl":"map_segments">"vpage 0x{:x} -> 0x{:x} ({:?})", virt.page(), phys.page(), flags);
            self.map(*virt, *phys, *flags);
        }
    }

}

impl Drop for Pgtbl {
    fn drop(&mut self) {
        panic!("freeing page table");
    }
}
