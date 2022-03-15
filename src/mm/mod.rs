pub mod address;
pub mod kalloc;
pub mod memory_space;
pub mod pgtbl;
pub mod pte_sv39;
use kalloc::KALLOCATOR;
use memory_space::MemorySpace;
// use page_table::PageTable;
use crate::{config::PHYS_FRAME_END, link_syms};
use address::*;
use pgtbl::Pgtbl;
use pte_sv39::PTEFlag;

pub static mut KERNEL_MEMORY_SPACE: MemorySpace = MemorySpace::default();

pub fn init() {
    // phys_frame::init();
    let frame_start = link_syms::frames as usize;
    let frame_start: PageNum = Into::<PhysAddr>::into(frame_start).into();
    let frame_start: PageNum = frame_start + Into::<PageNum>::into(1usize);
    let frame_end: PageNum = Into::<PhysAddr>::into(PHYS_FRAME_END).into();
    KALLOCATOR.lock().init(frame_start..frame_end);

    // Initialize the kernel page table
    let pgtbl = Pgtbl::new();
    unsafe {
        KERNEL_MEMORY_SPACE.page_table = pgtbl;
        // 为内核页表映射全部地址空间，页表可能占用过多空间
        KERNEL_MEMORY_SPACE.page_table.mappages(
            (link_syms::skernel as usize).into()..PHYS_FRAME_END.into(),
            (Into::<PhysAddr>::into(link_syms::skernel as usize)).into(),
            PTEFlag::V | PTEFlag::R | PTEFlag::W | PTEFlag::X,
        )
    };
    set_sstatus_sum();
    // set_sstatus_mxr();
    kernel_map_trampoline();
    println!("[kernel] Try to activate VM");
    activate_vm();
    // 测试开启虚拟内存后的内存分配功能
    let page = KALLOCATOR.lock().kalloc();
    println!("test alloc 0x{:x}", page.0);
    KALLOCATOR.lock().kfree(page);
}

#[allow(unused)]
fn map_kernel_memory_space() {
    let kernel_start: VirtualAddr = (link_syms::skernel as usize).into();
    let kernel_end: VirtualAddr = (link_syms::frames as usize).into();
    println!(
        "[kernel] Maping kernel (0x{:x}, 0x{:x})",
        kernel_start.0, kernel_end.0
    );
    unsafe {
        KERNEL_MEMORY_SPACE.page_table.mappages(
            kernel_start..kernel_end + 1.into(),
            kernel_start.into(),
            PTEFlag::R | PTEFlag::W | PTEFlag::X,
        );
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

fn kernel_map_trampoline() {
    unsafe {
        KERNEL_MEMORY_SPACE.map_trampoline();
    }
}

fn activate_vm() {
    unsafe {
        KERNEL_MEMORY_SPACE.page_table.activate();
    };
}
