use crate::mm::*;
use crate::process::*;
use spin::MutexGuard;

pub(super) fn sys_sbrk(pcb: &mut MutexGuard<Pcb>, inc: usize) -> usize {
    let target = pcb.memory_space.prog_break + inc;
    pcb.memory_space.prog_brk(target).0
}

pub(super) fn sys_brk(pcb: &mut MutexGuard<Pcb>, va: VirtualAddr) -> usize {
    pcb.memory_space.prog_brk(va).0
}
