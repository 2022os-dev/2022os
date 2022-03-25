use core::mem::size_of;
use core::ops::Range;
use riscv::register::satp;
use crate::config::*;
use crate::asm;
use super::address::*;
use super::pte_sv39::{PTEFlag, PTE};

use super::kalloc::KALLOCATOR;

#[derive(Copy, Clone)]
pub struct Pgtbl {
    pub root: PageNum,
}

impl Pgtbl {
    pub fn new() -> Self {
        let page = KALLOCATOR.lock().kalloc();
        page.offset_phys(0).write_bytes(0, PAGE_SIZE);
        Self { root: page }
    }

    pub fn walk(&mut self, va: VirtualAddr, do_alloc: bool) -> &mut PTE {
        let page: PageNum = va.floor();
        let mut ppn = self.root;
        let mut pte: &mut PTE = ppn.offset_phys(0).as_mut();
        // 固定解析三级页表，不支持巨页
        for level in (1..PAGE_TABLE_LEVEL).rev() {
            let mut physpte = ppn.offset_phys(page.vpn_block_sv39(level) * size_of::<usize>());
            pte = physpte.as_mut();
            if pte.is_valid() {
                if pte.is_leaf() {
                    panic!("too short page table")
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

    pub fn map_pages(
        &mut self,
        pages: Range<PageNum>,
        mut start: PageNum,
        flags: PTEFlag,
    ) {
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
        let pte = self.walk(vpage.offset(0), true);
        if pte.is_valid() {
            panic!("remap page 0x{:x}", vpage.page())
        }
        pte.set_ppn(page);
        pte.set_flags(flags | PTEFlag::V);
    }

    pub fn unmap_pages(&mut self, vpages: Range<PageNum>, do_free: bool) {
        for page in vpages.start.page()..vpages.end.page() {
            self.unmap(page.into(), do_free);
        }
    }

    pub fn unmap(&mut self, vpage: PageNum, do_free: bool) {
        // Fixme: when unmap an invalid page
        let pte = self.walk(vpage.offset(0), false);
        if(do_free && pte.is_valid()) {
            KALLOCATOR.lock().kfree(pte.ppn());
        }
        pte.set_flags(!PTEFlag::V);
    }

    fn copy_page(from: PageNum, to: PageNum, alloc_mem: bool) {
        // Fixme: 不用搜索整个空间，只复制用户空间和trapframe,trampoline
        for idx in 0..(PAGE_SIZE / size_of::<usize>()) {
            let mut physpte = from.offset_phys(idx * size_of::<usize>());
            let pte :&mut PTE = physpte.as_mut();
            if pte.is_valid() {
                let mut child_phys = to.offset_phys(idx * size_of::<usize>());
                let child_pte :&mut PTE = child_phys.as_mut();
                child_pte.set_flags(pte.flags());

                if pte.is_leaf() {
                    if alloc_mem {
                        let child_page = KALLOCATOR.lock().kalloc();
                        child_pte.set_ppn(child_page);
                        child_page.offset_phys(0).write(pte.ppn().offset_phys(0).as_slice(PAGE_SIZE));
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
    pub fn copy(&self, alloc_mem: bool) -> Self {
        let child = Pgtbl::new();
        Self::copy_page(self.root, child.root, alloc_mem);
        child
    }

    pub fn activate(&self) {
        unsafe {
            satp::set(satp::Mode::Sv39, 0, self.root.page());
            asm!("sfence.vma");
        }
    }
}
