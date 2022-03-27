use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use spin::{Mutex, MutexGuard};
use crate::mm::PhysAddr;
use crate::mm::VirtualAddr;
use crate::process::pcb::alloc_pid;
use crate::process::*;
use crate::task::*;

pub(super) fn sys_fork(pcb: &mut MutexGuard<Pcb>) -> isize {
    let child_ms = pcb.memory_space.copy();
    let pid = alloc_pid();
    let child = Arc::new(Mutex::new(Pcb {
        parent: pcb.pid,
        pid: pid,
        state: PcbState::Ready,
        memory_space: child_ms,
        children: Vec::new(),
    }));
    let mut childlock = child.lock();
    childlock.trapframe()["a0"] = 0;
    childlock.trapframe()["satp"] = childlock.memory_space.pgtbl.get_satp();
    drop(childlock);
    pcb.children.push(child.clone());
    scheduler_ready_pcb(child);
    pid as isize
}

pub(super) fn sys_getpid(pcb: &MutexGuard<Pcb>) -> isize {
    log!(debug "[sys_getpid]: {}", pcb.pid);
    pcb.pid as isize
}

pub(super) fn sys_yield() -> isize {
    log!(debug "[sys_yield]");
    0
}

pub(super) fn sys_exit(pcb: &mut MutexGuard<Pcb>, xstate: isize) {
    log!(debug "[sys_exit]: pid {} exit with code {}", pcb.pid, xstate);
    pcb.exit(xstate);
}

// 忽略rusage
pub(super) fn sys_wait4(pcb: &mut MutexGuard<Pcb>, pid: isize, wstatus: VirtualAddr, _: usize, _: VirtualAddr) -> Result<usize, ()> {
    // 阻塞直到某个子进程退出
    // 如果找不到退出的子进程，返回Err
    let mut xcode = 0;
    let res = pcb.children.iter().enumerate().find(|(_idx, child)| {
        let child = child.lock();
        if pid == -1 || child.pid == pid.abs() as usize {
            if let PcbState::Exit(_xcode) = child.state() {
                xcode = _xcode;
                return true
            }
            return false
        } else {
            return false
        }
    });
    if let Some((idx, child)) = res {
        let child_pid = child.lock().pid;
        log!(debug "[sys_wait4]: child {} exited with {}",child_pid, xcode);
        pcb.memory_space.copy_to_user(wstatus, PhysAddr::from(&xcode).as_slice(size_of::<usize>()));
        pcb.children.remove(idx);
        Ok(child_pid)
    }  else {
        Err(())
    }
}