use crate::process::pcb::alloc_pid;
use crate::process::*;
use crate::task::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};

pub(super) fn sys_fork(pcb: &mut MutexGuard<Pcb>) -> isize {
    let mut child_ms = pcb.memory_space.copy();
    let pid = alloc_pid();
    let child = Arc::new(Mutex::new(Pcb {
        pid: pid,
        state: PcbState::Ready,
        memory_space: child_ms,
        children: Vec::new(),
    }));
    child.lock().trapframe()["a0"] = 0;
    child.lock().trapframe()["satp"] = child_ms.pgtbl.root.page() | 0x8000000000000000;
    pcb.children.push(child.clone());
    scheduler_ready_pcb(child);
    pid as isize
}

pub(super) fn sys_getpid(pcb: &MutexGuard<Pcb>) -> isize {
    pcb.pid as isize
}

pub(super) fn sys_yield() -> isize {
    println!("[kernel] syscall Yield");
    0
}

pub(super) fn sys_exit(pcb: &mut MutexGuard<Pcb>, xstate: usize) {
    crate::println!("[kernel] Application exit with code {}", xstate);
    pcb.exit();
}
