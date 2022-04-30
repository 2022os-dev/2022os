use alloc::sync::Arc;
use spin::Mutex;

use super::Pcb;
use crate::asm;
use crate::config::*;
use crate::link_syms;
use crate::mm::address::PhysAddr;
use crate::mm::pgtbl::Pgtbl;
use crate::mm::*;
use crate::sbi::*;

// 最多支持4核
static mut _HARTS: [Hart; 5] = [
    Hart::default(),
    Hart::default(),
    Hart::default(),
    Hart::default(),
    Hart::default(),
];

pub struct Hart {
    pub hartid: usize,
    pub pcb: Option<Arc<Mutex<Pcb>>>,
    pub kernel_sp: usize,
    pub pgtbl: Option<Pgtbl>,
    // 保存hart在进入内核态时或将要进入用户态前的时钟，用于计算用户态和内核态运行时间
    pub times: usize,
}

impl const Default for Hart {
    fn default() -> Self {
        Self {
            hartid: 0,
            pcb: None,
            kernel_sp: 0,
            pgtbl: None,
            times: 0,
        }
    }
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
    let sp: PhysAddr = PhysAddr(sp).ceil().into();
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

    current_hart_pgtbl().map_pages(0x10040.into()..0x10051.into(), 0x10040.into(), PTEFlag::R | PTEFlag::W);

    unsafe {
        riscv::register::sstatus::set_sum();
        // 内核态不支持中断
        riscv::register::sstatus::clear_sie();
    }
    activate_vm();

    current_hart_set_trap_times(get_time());
}

pub fn current_hart() -> &'static mut Hart {
    unsafe { &mut _HARTS[hartid()] }
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
        let pcblock = current.lock();
        log!("hart":"leak">"pid({}) unmap segments", pcblock.pid);
        current_hart_pgtbl().unmap_segments(pcblock.memory_space.segments(), false);
        unsafe {
            asm!("sfence.vma");
        }
        // 用户栈
        log!("hart":"leak">"pid({}) unmap user stack", pcblock.pid);
        current_hart_pgtbl().unmap(MemorySpace::get_stack_start().floor(), false);
        unmap_mmap_areas(&*pcblock);
        drop(pcblock);
    }
}

// 映射mmap区域
fn map_mmap_areas(pcb: &Pcb) {
    for mappage in pcb.memory_space.mmap_areas.pages() {
        if let Some(ppage) = mappage.ppage {
            log!("mmap":"map">"vpage 0x{:x} -> ppage 0x{:x} ({:?})", mappage.vpage.page(), ppage.page(), mappage.get_pte_flags());
            current_hart_pgtbl().map(mappage.vpage, ppage, mappage.get_pte_flags() | PTEFlag::U);
        }
    }
}

// 取消映射mmap区域
fn unmap_mmap_areas(pcb: &Pcb) {
    for mappage in pcb.memory_space.mmap_areas.pages() {
        if let Some(ppage) = mappage.ppage {
            log!("mmap":"unmap">"vpage 0x{:x} -> ppage 0x{:x} ({:?})", mappage.vpage.page(), ppage.page(), mappage.get_pte_flags());
            current_hart_pgtbl().unmap(mappage.vpage, false);
        }
    }
}

pub fn current_hart_run(pcb: Arc<Mutex<Pcb>>) -> ! {
    current_hart_leak();
    let mut pcblock = pcb.lock();
    log!("hart":"run">"pid({})", pcblock.pid);
    // 需要设置进程的代码数据段、用户栈、trapframe、内核栈
    log!("hart":"run">"map segments");
    // map_segments将代码数据段映射到页表
    current_hart_pgtbl().map_segments(pcblock.memory_space.segments());
    map_mmap_areas(&*pcblock);
    // 映射用户栈,U flags
    let stack = pcblock.memory_space.user_stack;
    log!("hart":"run">"map user stack page 0x{:x}", stack.page());
    current_hart_pgtbl().map(
        MemorySpace::get_stack_start().floor(),
        stack,
        PTEFlag::R | PTEFlag::W | PTEFlag::U,
    );

    // 设置内核栈
    pcblock.trapframe().kernel_sp = current_hart().kernel_sp;
    log!("hart":"run">"sepc: 0x{:x}", pcblock.trapframe()["sepc"]);
    let tf = VirtualAddr(pcblock.trapframe() as *const _ as usize);
    pcblock.stimes_add(get_time() - current_hart_set_trap_times(get_time()));
    drop(pcblock);
    current_hart().pcb = Some(pcb);
    // 因为是在对当前使用的页表进行映射，所以可能需要刷新快表
    unsafe {
        asm!("sfence.vma");
    }
    unsafe { crate::trap::__restore(tf.0); }
    loop {}
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
