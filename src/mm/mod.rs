pub mod address;
pub mod kalloc;
pub mod memory_space;
pub mod pgtbl;
pub mod pte_sv39;

use crate::{config::PHYS_FRAME_END, link_syms};
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
    let frame_end = Into::<PhysAddr>::into(PHYS_FRAME_END).ceil();
    KALLOCATOR.lock().init(frame_start..frame_end);

    let mut kernel_memory_space = MemorySpace {
        pgtbl: Pgtbl::new(),
        entry: 0,
    };

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

    // set_sstatus_sum();
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
    println!("[kernel] Try to activate VM");
    kernel_memory_space.pgtbl.activate();
    // 测试开启虚拟内存后的内存分配功能
    let page = KALLOCATOR.lock().kalloc();
    println!("test alloc 0x{:x}", page.page());
    KALLOCATOR.lock().kfree(page);
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
    let start = Into::<VirtualAddr>::into(link_syms::skernel as usize).floor();
    let end = Into::<VirtualAddr>::into(link_syms::frames as usize).floor();
    start..end
}

fn frames_range() -> Range<PageNum> {
    let start = Into::<VirtualAddr>::into(link_syms::frames as usize).floor();
    let end = Into::<VirtualAddr>::into(PHYS_FRAME_END).ceil();
    start..end
}
