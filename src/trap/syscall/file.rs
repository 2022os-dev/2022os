use crate::mm::*;
use crate::process::*;
use crate::sbi::sbi_legacy_call;
use spin::MutexGuard;

pub(super) fn sys_getcwd (
    pcb: &mut MutexGuard<Pcb>,
    buf: VirtualAddr,
    len: usize,
) -> VirtualAddr {
    if buf.0 == 0 {
        // 由系统分配缓存区，不支持
        VirtualAddr(0)
    }  else {
        let mut buf: PhysAddr = buf.into();
        // Fixme: 考虑 len长度限制
        buf.write(pcb.cwd.as_bytes());
        VirtualAddr(buf.0)
    }
}

pub(super) fn sys_pipe (
    pcb: &mut MutexGuard<Pcb>,
    pipe: VirtualAddr
) -> isize {
    unimplemented!();
}

pub(super) fn sys_dup (
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
) -> isize {
    unimplemented!();
}

pub(super) fn sys_dup3 (
    pcb: &mut MutexGuard<Pcb>,
    old: usize,
    new: usize
) -> isize {
    unimplemented!();
}

pub(super) fn sys_chdir (
    pcb: &mut MutexGuard<Pcb>,
    path: VirtualAddr
) -> isize {
    unimplemented!();
}

pub(super) fn sys_openat (
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
    filename: VirtualAddr,
    flags: usize,
    mode: usize
) -> isize {
    unimplemented!();
}

pub(super) fn sys_close (
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
) -> isize {
    unimplemented!();
}

pub(super) fn sys_getdents64 (
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
    buf: VirtualAddr,
    len: usize
) -> isize {
    unimplemented!();
}


pub(super) fn sys_write(
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
    buf: VirtualAddr,
    len: usize,
) -> isize {
    let mut buf: PhysAddr = buf.into();
    let buf: &[u8] = buf.as_slice_mut(len);
    if let Some(fd) = pcb.get_mut_fd(fd) {
        if let Err(_) = fd.write().write(buf) {
            -1 
        } else {
            len as isize
        }
    } else {
        -1
    }
}

pub(super) fn sys_read(
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
    buf: VirtualAddr,
    len: usize,
) -> isize {
    let mut buf: PhysAddr = buf.into();
    let buf: &mut [u8] = buf.as_slice_mut(len);
    if let Some(fd) = pcb.get_mut_fd(fd) {
        if let Err(_) = fd.write().read(buf) {
            -1 
        } else {
            len as isize
        }
    } else {
        -1
    }

}