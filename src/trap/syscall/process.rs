use crate::config::*;
use crate::mm::PhysAddr;
use crate::mm::VirtualAddr;
use crate::process::pcb::alloc_pid;
use crate::process::signal::*;
use crate::process::*;
use crate::task::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use spin::{Mutex, MutexGuard};

pub(super) fn sys_fork(pcb: &mut MutexGuard<Pcb>) -> isize {
    return sys_clone(pcb, CloneFlags::empty(), VirtualAddr(0), 0, 0, 0);
}

bitflags! {
    pub struct CloneFlags: usize{
        const SIGCHLD = 17;
        const CLONE_CHILD_CLEARTID = 0x00200000;
        const CLONE_CHILD_SETTID = 0x01000000;
    }
}

pub(super) fn sys_clone(
    pcb: &mut MutexGuard<Pcb>,
    flags: CloneFlags,
    stack_top: VirtualAddr,
    ptid: usize,
    ctid: usize,
    newtls: usize,
) -> isize {
    // Note: 与Linux的clone不同，参考于UltraOs
    let child = pcb.clone_child();
    let childpid = child.lock().pid;
    // 设置栈
    if stack_top.0 != 0 {
        child.lock().trapframe()["sp"] = stack_top.0;
    }
    scheduler_ready_pcb(child);
    childpid as isize
}

pub(super) fn sys_getpid(pcb: &MutexGuard<Pcb>) -> isize {
    pcb.pid as isize
}

pub(super) fn sys_yield() -> isize {
    0
}

pub(super) fn sys_exit(pcb: &mut MutexGuard<Pcb>, xstate: isize) {
    pcb.exit(xstate);
    sigqueue_send(pcb.parent, Signal::SIGCHLD);
}

// 忽略rusage
pub(super) fn sys_wait4(
    pcb: &mut MutexGuard<Pcb>,
    pid: isize,
    wstatus: VirtualAddr,
    _: usize,
    _: VirtualAddr,
) {
    // 阻塞直到某个子进程退出
    // 找到pid指定的退出的子进程
    let find_child_exit = move |_pcb: &mut Pcb| -> Option<usize> {
        if let Some((idx, child)) = _pcb.children.iter().enumerate().find(|(_idx, child)| {
            let child = child.lock();
            if pid == -1 || child.pid == pid.abs() as usize {
                if let PcbState::Zombie(_) = child.state() {
                    return true;
                }
            }
            false
        }) {
            return Some(idx);
        }
        None
    };
    // 如果找到
    if let Some(idx) = find_child_exit(pcb) {
        let child = pcb.children.remove(idx);
        let child = child.lock();
        if let PcbState::Zombie(xcode) = child.state() {
            pcb.trapframe()["a0"] = child.pid;
            pcb.cutimes_add(child.utimes());
            pcb.cstimes_add(child.stimes());
            if wstatus.0 != 0 {
                let mut wstatus: PhysAddr = wstatus.into();
                let wstatus: &mut usize = wstatus.as_mut();
                *wstatus = (xcode << 8) as usize;
            }
        }
    } else {
        // 如果找不到，退回这条系统调用指令
        pcb.trapframe()["sepc"] -= 4;
        // 进程进入阻塞，知道指定子进程退出
        pcb.block_fn = Some(Arc::new(move |pcb| find_child_exit(pcb).is_some()));
        pcb.set_state(PcbState::Blocking);
    }
}
#[repr(C)]
struct Tms {
    utime: usize,
    stime: usize,
    cutime: usize,
    cstime: usize,
}
pub(super) fn sys_times(pcb: &mut MutexGuard<Pcb>, tms: VirtualAddr) -> usize {
    let mut tms: PhysAddr = tms.into();
    let tms: &mut Tms = tms.as_mut();
    tms.utime = pcb.utimes();
    tms.stime = pcb.stimes();
    tms.cutime = pcb.cutimes();
    tms.cstime = pcb.cstimes();
    // Fix: 只是简单返回times
    cpu::get_time()
}

#[repr(C)]
pub(super) struct TimeSpec {
    pub tv_sec: usize,
    pub tv_nsec: usize,
}

pub(super) fn sys_gettimeofday(timespec: VirtualAddr, _: VirtualAddr) -> isize {
    let mut timespec: PhysAddr = timespec.into();
    let timespec: &mut TimeSpec = timespec.as_mut();
    let time = cpu::get_time();
    timespec.tv_sec = time / RTCLK_FREQ;
    timespec.tv_nsec = (time % RTCLK_FREQ) / 1000_000;
    0
}

pub(super) fn sys_getppid(pcb: &MutexGuard<Pcb>) -> usize {
    pcb.parent
}
