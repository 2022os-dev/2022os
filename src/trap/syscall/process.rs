use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use spin::{Mutex, MutexGuard};
use crate::mm::PhysAddr;
use crate::process::signal::*;
use crate::mm::VirtualAddr;
use crate::process::pcb::alloc_pid;
use crate::process::*;
use crate::task::*;

pub(super) fn sys_fork(pcb: &mut MutexGuard<Pcb>) -> isize {
    let child_ms = pcb.memory_space.copy();
    let child = Arc::new(Mutex::new(Pcb::new(child_ms, pcb.pid)));
    let mut childlock = child.lock();
    let childpid = childlock.pid;
    childlock.trapframe()["a0"] = 0;
    drop(childlock);
    pcb.children.push(child.clone());
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
    log!("syscall":"exit"> "pid({})", pcb.pid);
    pcb.exit(xstate);
    sigqueue_send(pcb.parent, Signal::SIGCHLD);
}

// 忽略rusage
pub(super) fn sys_wait4(pcb: &mut MutexGuard<Pcb>, pid: isize, wstatus: VirtualAddr, _: usize, _: VirtualAddr) -> Result<usize, ()> {
    // 阻塞直到某个子进程退出
    // 如果找不到退出的子进程，返回Err
    let mut xcode = 0;
    let mut childutimes = 0;
    let mut childstimes = 0;
    let res = pcb.children.iter().enumerate().find(|(_idx, child)| {
        let child = child.lock();
        if pid == -1 || child.pid == pid.abs() as usize {
            if let PcbState::Exit(_xcode) = child.state() {
                xcode = _xcode;
                childutimes = child.utimes();
                childstimes = child.stimes();
                return true
            }
            return false
        } else {
            return false
        }
    });
    if let Some((idx, child)) = res {
        let child_pid = child.lock().pid;
        // wstatus = xcode
        let mut wstatus: PhysAddr = wstatus.into();
        let wstatus: &mut usize = wstatus.as_mut();
        *wstatus = xcode as usize;
        pcb.cutimes_add(childutimes);
        pcb.cstimes_add(childstimes);
        // 清理子进程
        pcb.children.remove(idx);
        Ok(child_pid)
    }  else {
        Err(())
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