pub mod address;
pub mod kalloc;
pub mod memory_space;
pub mod pgtbl;
pub mod pte_sv39;

use crate::{config::PHYS_FRAME_END, link_syms};
use crate::KERNEL_PGTBL;
use core::ops::Range;

pub use kalloc::KALLOCATOR;
pub use memory_space::MemorySpace;
// use page_table::PageTable;
pub use address::*;
pub use pgtbl::Pgtbl;
pub use pte_sv39::*;

pub fn init() {
    // phys_frame::init();
    let frame_start = kernel_range().end;
    let frame_end = PhysAddr(PHYS_FRAME_END).ceil();
    KALLOCATOR.lock().init(frame_start..frame_end);

    let mut kernel_memory_space = MemorySpace::new(None);

    // 为内核页表映射全部地址空间，页表可能占用过多空间
    kernel_memory_space.pgtbl.map_pages(
        kernel_range(),
        kernel_range().start,
        PTEFlag::R | PTEFlag::W | PTEFlag::X,
    );

    kernel_memory_space.pgtbl.map_pages(
        frames_range(),
        frames_range().start,
        PTEFlag::R | PTEFlag::W,
    );

    kernel_memory_space.map_trampoline();

    // ################### TEST ######################
    let range = kernel_range();
    for i in range.start.page()..range.end.page() {
        let pte = kernel_memory_space
            .pgtbl
            .walk(Into::<PageNum>::into(i).offset(0), false);
        if !pte.is_valid() {
            println!("0x{:x} is invalid", i);
        }
    }
    let range = frames_range();
    for i in range.start.page()..range.end.page() {
        let pte = kernel_memory_space
            .pgtbl
            .walk(Into::<PageNum>::into(i).offset(0), false);
        if !pte.is_valid() {
            println!("0x{:x} is invalid", i);
        }
    }
    // ###############################################
    unsafe {
        KERNEL_PGTBL = Some(kernel_memory_space.pgtbl);
    };
}

pub fn activate_vm(ppn: usize) {
    use riscv::register::satp;
    use core::arch::asm;
    unsafe {
        satp::set(satp::Mode::Sv39, 0, ppn);
        asm!("sfence.vma");
    }
}

#[allow(unused)]
fn set_sstatus_sum() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}

#[allow(unused)]
fn set_sstatus_mxr() {
    unsafe {
        riscv::register::sstatus::set_mxr();
    }
}

fn kernel_range() -> Range<PageNum> {
    let start = VirtualAddr(link_syms::skernel as usize).floor();
    let end = VirtualAddr(link_syms::frames as usize).floor();
    start..end
}

fn frames_range() -> Range<PageNum> {
    let start = VirtualAddr(link_syms::frames as usize).floor();
    let end = VirtualAddr(PHYS_FRAME_END).ceil();
    start..end
}
