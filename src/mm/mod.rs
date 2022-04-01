pub mod address;
pub mod kalloc;
pub mod memory_space;
pub mod pgtbl;
pub mod pte_sv39;

use crate::link_syms;
use crate::config::*;
use crate::process::cpu::*;
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
}

pub fn activate_vm() {
    log!(debug "hart {} trying VM", hartid());
    // ################### TEST ######################
    let range = kernel_range();
    let p = current_hart_pgtbl();
    for i in range.start.page()..range.end.page() {
        let pte = p
            .walk(Into::<PageNum>::into(i).offset(0), false);
        if !pte.is_valid() {
            println!("0x{:x} is invalid", i);
        }
    }
    let range = frames_range();
    for i in range.start.page()..range.end.page() {
        let pte = p
            .walk(Into::<PageNum>::into(i).offset(0), false);
        if !pte.is_valid() {
            println!("0x{:x} is invalid", i);
        }
    }
    // ###############################################
    log!(debug "Test finished before activate VM");
    use riscv::register::satp;
    use core::arch::asm;
    unsafe {
        satp::set(satp::Mode::Sv39, 0, p.root.0);
        asm!("sfence.vma");
    }
}

pub fn kernel_range() -> Range<PageNum> {
    let start = VirtualAddr(link_syms::skernel as usize).floor();
    let end = VirtualAddr(link_syms::frames as usize).floor();
    start..end
}

pub fn frames_range() -> Range<PageNum> {
    let start = VirtualAddr(link_syms::frames as usize).floor();
    let end = VirtualAddr(PHYS_FRAME_END).ceil();
    start..end
}
