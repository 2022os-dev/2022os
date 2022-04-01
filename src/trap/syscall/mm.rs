use crate::mm::*;
use crate::process::*;
use spin::MutexGuard;

pub(super) fn sys_sbrk(pcb: &mut MutexGuard<Pcb>, inc: usize) -> usize {
    pcb.memory_space.prog_sbrk(inc).0
}

pub(super) fn sys_brk(pcb: &mut MutexGuard<Pcb>, va: VirtualAddr) -> isize {
    if let Ok(_) = pcb.memory_space.prog_brk(va) {
        0
    } else {
        -1
    }
}
