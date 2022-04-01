use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

use super::Pcb;
use crate::config::*;
use crate::link_syms;
use crate::mm::address::PhysAddr;
use crate::mm::pgtbl::Pgtbl;
use crate::mm::*;
use crate::sbi::*;
use crate::process::restore_trapframe;
use crate::asm;

// 最多支持4核
static mut _HARTS :[Hart; 5] = [Hart::default(), Hart::default(),Hart::default(),Hart::default(),Hart::default(),];

pub struct Hart {
    pub hartid: usize,
    pub pcb: Option<Arc<Mutex<Pcb>>>,
    pub kernel_sp: usize,
    pub pgtbl: Option<Pgtbl>,
    // 保存hart在进入内核态时或将要进入用户态前的时钟，用于计算用户态和内核态运行时间
    pub times: usize
}

impl const Default for Hart {
    fn default() -> Self {
        Self {
            hartid: 0,
            pcb: None,
            kernel_sp: 0,
            pgtbl: None,
            times: 0
        }
    }
}

lazy_static! {
    static ref HARTS: Mutex<Vec<Hart>> = Mutex::new(Vec::new());
}

pub fn get_time() -> usize {
    riscv::register::time::read()
}

fn hart_set_timecmp(timecmp: usize) {
    crate::sbi::sbi_legacy_call(SET_TIMER, [timecmp, 0, 0]);
}

pub fn hart_set_next_trigger() {
    hart_set_timecmp(get_time() + RTCLK_FREQ * 1);
}

pub fn hart_enable_timer_interrupt() {
    use riscv::register::*;
    unsafe {
        sie::set_stimer();
    }
    hart_set_next_trigger();
}

pub fn init_hart() {
    log!("hart":>"init");
    let sp: usize = link_syms::boot_stack_top as usize - BOOT_STACK_SIZE * hartid();
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
        // 内核态不支持中断
        riscv::register::sstatus::clear_sie();
    }
    activate_vm();

    current_hart_set_trap_times(get_time());
}

pub fn current_hart() -> &'static mut Hart {
    unsafe {
        &mut _HARTS[hartid()]
    }
}

pub fn current_hart_trap_times() -> usize {
    current_hart().times
}

pub fn current_hart_set_trap_times(times: usize) -> usize {
    let t = current_hart_trap_times();
    current_hart().times = times;
    t
}

pub fn current_hart_leak() {
    if let Some(current) = current_hart().pcb.take() {
        let pid = current.lock().pid;
        log!("hart":"leak">"pid({}) unmap segments", pid);
        current_hart_pgtbl().unmap_segments(
            current.lock().memory_space.segments(), false);
        unsafe {
            asm!("sfence.vma");
        }
        // 用户栈
        log!("hart":"leak">"pid({}) unmap user stack",pid);
        current_hart_pgtbl().unmap(MemorySpace::get_stack_start().floor(), false);
        drop(current);
    }
}

pub fn current_hart_run(pcb: Arc<Mutex<Pcb>>) -> !{
    current_hart_leak();
    let pid = pcb.lock().pid;
    log!("hart":"run">"pid({})", pid);
    // 需要设置进程的代码数据段、用户栈、trapframe、内核栈
    log!("hart":"run">"map segments");
    // map_segments将代码数据段映射到页表
    current_hart_pgtbl().map_segments(pcb.lock().memory_space.segments());
    // 因为是在对当前使用的页表进行映射，所以可能需要刷新快表
    unsafe {
        asm!("sfence.vma");
    }
    // 映射用户栈,U flags
    let stack = pcb.lock().memory_space.user_stack;
    log!("hart":"run">"map user stack page 0x{:x}", stack.page());
    current_hart_pgtbl().map(MemorySpace::get_stack_start().floor(), stack, PTEFlag::R | PTEFlag::W | PTEFlag::U);

    // 设置内核栈
    pcb.lock().trapframe().kernel_sp = current_hart().kernel_sp;
    let sepc = pcb.lock().trapframe()["sepc"];
    log!("hart":"run">"sepc: 0x{:x}", sepc);
    let tf = VirtualAddr(pcb.lock().trapframe() as *const _ as usize);
    pcb.lock().stimes_add(get_time() - current_hart_set_trap_times(get_time()));
    current_hart().pcb = Some(pcb);
    restore_trapframe(tf);
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
