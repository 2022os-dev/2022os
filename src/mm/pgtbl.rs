use super::address::*;
use super::pte_sv39::{PTEFlag, PTE};
use crate::asm;

use super::kalloc::KALLOCATOR;
use crate::config::*;
use riscv::register::satp;

#[derive(Copy, Clone)]
pub struct Pgtbl {
    pub root: PageNum,
}

impl Pgtbl {
    pub fn new() -> Self {
        let page = KALLOCATOR.lock().kalloc();
        Self { root: page }
    }

    pub fn walk(&mut self, va: VirtualAddr, do_alloc: bool) -> &mut PTE {
        let page: PageNum = va.floor();
        let mut ppn = self.root;
        let mut pte;
        for level in (1..PAGE_TABLE_LEVEL).rev() {
            let pte_ptr = ppn
                .offset(page.vpn_block_sv39(level) * core::mem::size_of::<usize>())
                .0 as *mut PTE;
            pte = unsafe { pte_ptr.as_mut().unwrap() };
            if pte.is_valid() {
                if pte.is_leaf() {
                    panic!("too short page table")
                }
                ppn = pte.ppn();
            } else {
                if do_alloc {
                    let page = KALLOCATOR.lock().kalloc();
                    pte.set_ppn(page);
                    pte.set_flags(PTEFlag::V);
                    ppn = page;
                } else {
                    panic!("walk invalid 0x{:x}", va.0)
                }
            }
        }
        unsafe {
            (ppn.offset(page.vpn_block_sv39(0) * core::mem::size_of::<usize>())
                .0 as *mut PTE)
                .as_mut()
                .unwrap()
        }
    }

    pub fn map_pages(
        &mut self,
        pages: core::ops::Range<PageNum>,
        mut start: PageNum,
        flags: PTEFlag,
    ) {
        println!(
            "Mapping pages 0x{:x} - 0x{:x}",
            pages.start.page(),
            pages.end.page()
        );
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

    pub fn activate(&self) {
        unsafe {
            satp::set(satp::Mode::Sv39, 0, self.root.page());
            asm!("sfence.vma");
        }
    }
}
