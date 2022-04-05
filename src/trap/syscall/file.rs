use core::ops::Add;

use crate::mm::*;
use crate::process::*;
use crate::sbi::sbi_legacy_call;
use spin::MutexGuard;
use crate::vfs::*;

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
    // Test:目前只在Root建立文件
    let filename: PhysAddr = filename.into();
    let mut len = 0;
    while len < 512 {
        if unsafe { *(filename.0 as *const u8).add(len) } != 0 {
            len += 1;
        } else {
            break
        }
    }
    let filename = unsafe { core::str::from_utf8_unchecked(filename.as_slice(len)) };
    let flags = OpenFlags::from_bits(flags).unwrap();
    let mode = FileMode::from_bits(mode).unwrap();

    let f = ROOT.open_child(filename, flags);
    if let Ok(file) = f {
        log!("syscall":"openat">"ok");
        pcb.fds.push(file);
        return (pcb.fds.len() - 1) as isize
    } else if flags.contains(OpenFlags::CREATE){
        if let Ok(inode) = ROOT.create(filename, mode) {
            if let Ok(file) = File::open(inode, flags) {
                pcb.fds.push(file);
                return (pcb.fds.len() - 1) as isize
            }
        }
    }
    -1
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

pub(super) fn sys_lseek (
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
    offset: isize,
    whence: usize
) -> isize {
    if let Some(file) = pcb.get_mut_fd(fd) {
        if let Ok(pos) = file.lseek(whence, offset) {
            pos as isize
        } else {
            -1
        }
    } else {
        -1
    }
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
        if let Err(_) = fd.write(buf) {
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
        if let Err(_) = fd.read(buf) {
            -1 
        } else {
            len as isize
        }
    } else {
        -1
    }

}