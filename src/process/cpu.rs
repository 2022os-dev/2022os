use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

use super::Pcb;
use crate::config::BOOT_STACK_SIZE;
use crate::config::USER_STACK;
use crate::link_syms;
use crate::mm::address::PhysAddr;
use crate::mm::pgtbl::Pgtbl;
use crate::mm::*;
use crate::process::PcbState;
use crate::asm;

// 最多支持4核
static mut _HARTS :[Hart; 5] = [Hart::default(), Hart::default(),Hart::default(),Hart::default(),Hart::default(),];

pub struct Hart {
    pub hartid: usize,
    pub pcb: Option<Arc<Mutex<Pcb>>>,
    pub kernel_sp: usize,
    pub pgtbl: Option<Pgtbl>
}

impl const Default for Hart {
    fn default() -> Self {
        Self {
            hartid: 0,
            pcb: None,
            kernel_sp: 0,
            pgtbl: None
        }
    }
}

lazy_static! {
    static ref HARTS: Mutex<Vec<Hart>> = Mutex::new(Vec::new());
}

pub fn init_hart() {
    log!("hart":>"init");
    let sp: usize = link_syms::boot_stack_top as usize - BOOT_STACK_SIZE * hartid();
    // Note: 进入该函数时栈大小不应该超过 1 页
    let sp :PhysAddr = PhysAddr(sp).ceil().into();
    current_hart().hartid = hartid();
    current_hart().kernel_sp = sp.0;
    current_hart().pgtbl = Some(Pgtbl::new());
    current_hart_pgtbl().map_pages(
        kernel_range(),
        kernel_range().start,
        PTEFlag::R | PTEFlag::W | PTEFlag::X,
    );

    current_hart_pgtbl().map_pages(
        frames_range(),
        frames_range().start,
        PTEFlag::R | PTEFlag::W,
    );

    current_hart_pgtbl().map_trampoline();
    unsafe {
        riscv::register::sstatus::set_sum();
        // 目前还不支持中断
        riscv::register::sstatus::clear_sie();
    }
    activate_vm();
}

pub fn current_hart() -> &'static mut Hart {
    unsafe {
        &mut _HARTS[hartid()]
    }
}

pub fn current_hart_leak() {
    log!("hart":"leak">"hartid({})", hartid());
    if let Some(current) = current_hart().pcb.take() {
        log!("hart":"leak">"hartid({}) unmap segments", hartid());
        current_hart_pgtbl().unmap_segments(
            current.lock().memory_space.segments(), false);
        unsafe {
            asm!("sfence.vma");
        }
        // 用户栈
        log!("hart":"leak">"hartid({}) unmap user stack", hartid());
        current_hart_pgtbl().unmap(VirtualAddr(USER_STACK).floor(), false);
        drop(current);
    }
}

pub fn current_hart_run(pcb: Arc<Mutex<Pcb>>) {
    current_hart_leak();
    // 需要设置进程的代码数据段、用户栈、trapframe、内核栈
    log!("hart":"run">"map segments");
    // map_segments将代码数据段映射到页表
    current_hart_pgtbl().map_segments(pcb.lock().memory_space.segments());
    // 因为是在对当前使用的页表进行映射，所以可能需要刷新快表
    unsafe {
        asm!("sfence.vma");
    }
    // 在原地映射用户栈,U flags
    let stack = pcb.lock().memory_space.user_stack;
    log!("hart":"run">"map user stack page 0x{:x}", stack.page());
    current_hart_pgtbl().map(VirtualAddr(USER_STACK).floor(), stack, PTEFlag::R | PTEFlag::W | PTEFlag::U);

    // 设置内核栈
    pcb.lock().trapframe().kernel_sp = current_hart().kernel_sp;
    pcb.lock().set_state(PcbState::Running);
    current_hart().pcb = Some(pcb);
}

pub fn current_hart_pgtbl() -> &'static mut Pgtbl {
    current_hart().pgtbl.as_mut().unwrap()
}

pub fn hartid() -> usize {
    let ret: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) ret);
    }
    ret
}
