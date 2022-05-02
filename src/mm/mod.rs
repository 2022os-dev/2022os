pub mod address;
pub mod kalloc;
pub mod memory_space;
pub mod pgtbl;
pub mod pte_sv39;

use crate::config::*;
use crate::link_syms;
use crate::process::cpu::*;
use core::ops::Range;

pub use address::*;
pub use address::*;
pub use kalloc::*;
pub use memory_space::*;
pub use pgtbl::*;
pub use pte_sv39::*;

pub fn init() {
    // phys_frame::init();
    let frame_start = kernel_range().end;
    let frame_end = PhysAddr(PHYS_FRAME_END).ceil();
    KALLOCATOR.lock().init(frame_start..frame_end);
}

pub fn activate_vm() {
    log!("vm":>"hart {} trying VM", hartid());
    // ################### TEST ######################
    let range = kernel_range();
    let p = current_hart_pgtbl();
    for i in range.start.page()..range.end.page() {
        let pte = p.walk(Into::<PageNum>::into(i).offset(0), false);
        if !pte.is_valid() {
            println!("0x{:x} is invalid", i);
        }
    }
    let range = frames_range();
    for i in range.start.page()..range.end.page() {
        let pte = p.walk(Into::<PageNum>::into(i).offset(0), false);
        if !pte.is_valid() {
            println!("0x{:x} is invalid", i);
        }
    }
    // ###############################################
    log!("vm":>"Test finished before activate VM");
    use core::arch::asm;
    // use riscv::register::satp;
    unsafe {
        asm!("csrw satp, {}",
            "sfence.vma",
            "li a0, 68",
            "li a7, 1",
            "ecall", in(reg) 8usize << 60 | p.root.0);
        // satp::set(satp::Mode::Sv39, 0, p.root.0);
        // asm!("sfence.vma");
    }
    log!("vm":>"Activated vm");
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
