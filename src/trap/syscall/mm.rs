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

pub(super) fn sys_mmap(
    pcb: &mut MutexGuard<Pcb>,
    start: VirtualAddr,
    length: usize,
    prot: usize,
    flags: usize,
    fd: isize,
    offset: usize,
) -> VirtualAddr {
    let prot = MapProt::from_bits(prot).unwrap();
    let flags = MapFlags::from_bits(flags).unwrap();
    // todo: 支持匿名映射
    if let Some(va) = pcb.get_fd(fd).and_then(|file| {
        match pcb.memory_space.mmap(start, Some(file.read().inode.clone()), offset, length, prot, flags) {
            Ok(va) => {
                Some(va)
            }
            Err(e) => {
                None
            }
        }
    }) {
        return va
    } else {
        return VirtualAddr(-1 as isize as usize)
    }
}
