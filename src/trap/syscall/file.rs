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
    path: VirtualAddr,
    flags: usize,
    mode: usize
) -> isize {
    let filename: PhysAddr = path.into();
    let mut len = 0;
    // Fixme: 假设路径最长为512
    while len < 512 {
        if unsafe { *(filename.0 as *const u8).add(len) } != 0 {
            len += 1;
        } else {
            break
        }
    }
    let path = unsafe { core::str::from_utf8_unchecked(filename.as_slice(len)) };
    let flags = OpenFlags::from_bits(flags).unwrap();
    let mode = FileMode::from_bits(mode).unwrap();
    if is_absolute_path(path) {
        log!("syscall":"openat">"absolute path: {}", path);
        match parse_path(&*ROOT, path) {
            Ok(inode) => {
                log!("syscall":"openat">"path exists: {}", path);
                if let Ok(_) = File::open(inode, flags).and_then(|file| {
                    if pcb.fds_add(fd, file) {
                        Ok(())
                    } else {
                        Err(FileErr::InvalidFd)
                    }
                }) {
                    return fd as isize
                } else {
                    return -1
                }
            }
            Err(FileErr::InodeNotChild) if flags.contains(OpenFlags::CREATE) => {
                log!("syscall":"openat">"path not exists, create: {}", path);
                let (rest, comp) = rsplit_path(path);
                if let Some(rest) = rest {
                    if let Ok(_) = parse_path(&*ROOT, rest).and_then(|inode| {
                        inode.create(comp, mode)
                    }).and_then(|inode| {
                        File::open(inode, flags)
                    }).and_then(|file| {
                        if pcb.fds_add(fd, file) {
                            Ok(())
                        } else {
                            Err(FileErr::InvalidFd)
                        }
                    }) {
                        return fd as isize
                    }
                }
                return -1
            }
            _ => {
                log!("syscall":"openat">"path not exists: {}", path);
                return -1
            }
        }
    } else {
        log!("syscall":"openat">"relative path: {}", path);
        // 目前暂时不支持相对路径
        return -1
    }
}

pub(super) fn sys_close (
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
) -> isize {
    if pcb.fds_close(fd) {
        0
    } else {
        -1
    }
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