use core::ops::Add;
use alloc::sync::Arc;

use crate::mm::*;
use crate::process::*;
use crate::sbi::sbi_legacy_call;
use spin::MutexGuard;
use crate::vfs::*;

fn get_str(pa: &PhysAddr) -> &str {
    let mut len = 0;
    // Fixme: 假设路径最长为512
    while len < 512 {
        if unsafe { *(pa.0 as *const u8).add(len) } != 0 {
            len += 1;
        } else {
            break
        }
    }
    unsafe { core::str::from_utf8_unchecked(pa.as_slice(len)) }
}

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

pub(super) fn sys_mkdirat(
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    path: VirtualAddr,
    mode: usize
) -> isize {
    let phys: PhysAddr = path.into();
    let path = get_str(&phys);
    let mode = FileMode::from_bits(mode).unwrap();
    if is_absolute_path(path) {
        // 绝对路径,忽略fd
        let (rest, name) = rsplit_path(path);
        match rest.and_then(|rest| {
            parse_path(&pcb.root, rest).and_then(|inode| {
                inode.create(name, mode, InodeType::Directory)
            }).ok()
        }) {
            Some(_) => {
                log!("syscall":"mkdirat">"success");
                0
            }
            None => {
                -1
            }
        }
    } else {
        // todo: 暂时不支持相对路径
        // path是fd的相对路径
        -1
    }
}

pub(super) fn sys_pipe (
    pcb: &mut MutexGuard<Pcb>,
    pipe: VirtualAddr
) -> isize {
    let mut phys: PhysAddr = pipe.into();
    let pipe: &mut [usize; 2] = phys.as_mut();
    if let Ok((reader, writer)) = make_pipe().and_then(|(reader, writer)| {
        pcb.fds_insert(reader).and_then(|rfd| {
            pcb.fds_insert(writer).and_then(|wfd| {
                Some((rfd, wfd))
            })
        }).ok_or(FileErr::NotDefine)
    }) {
        pipe[0] = reader;
        pipe[1] = writer;
        0
    } else {
        log!("syscall":"pipe">"fail");
        -1
    }
}

pub(super) fn sys_dup (
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
) -> isize {
    match pcb.get_fd(fd).and_then(|fd| {
        pcb.fds_insert(fd)
    }) {
        Some(fd) => {
            fd as isize
        }
        None => {
            -1
        }
    }
}

pub(super) fn sys_dup3 (
    pcb: &mut MutexGuard<Pcb>,
    oldfd: isize,
    newfd: isize
) -> isize {
    // Fixme: 2021初赛中没有指定flags选项
    if oldfd == newfd {
        return newfd as isize
    }
    match pcb.get_fd(oldfd).and_then(|fd| {
        pcb.fds_close(newfd);
        if pcb.fds_add(newfd, fd) {
            Some(())
        } else {
            None
        }
    }) {
        Some(_) => {
            newfd as isize
        }
        None => {
            -1
        }
    }
}

pub(super) fn sys_chdir (
    pcb: &mut MutexGuard<Pcb>,
    path: VirtualAddr
) -> isize {
    unimplemented!();
}

pub(super) fn sys_openat (
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    path: VirtualAddr,
    flags: usize,
    mode: usize
) -> isize {
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let flags = OpenFlags::from_bits(flags).unwrap();
    let mode = FileMode::from_bits(mode).unwrap();
    if is_absolute_path(path) {
        log!("syscall":"openat">"absolute path: {}", path);
        match parse_path(&pcb.root, path) {
            Ok(inode) => {
                log!("syscall":"openat">"path exists: {}", path);
                if let Ok(_) = File::open(inode, flags).and_then(|file| {
                    if pcb.fds_add(fd, file) {
                        Ok(())
                    } else {
                        Err(FileErr::FdInvalid)
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
                    // 解析Path中除去最后一个节点的剩余节点
                    if let Ok(_) = parse_path(&pcb.root, rest).and_then(|inode| {
                        inode.create(comp, mode, InodeType::File)
                    }).and_then(|inode| {
                        File::open(inode, flags)
                    }).and_then(|file| {
                        if pcb.fds_add(fd, file) {
                            Ok(())
                        } else {
                            Err(FileErr::FdInvalid)
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
    fd: isize,
) -> isize {
    if pcb.fds_close(fd) {
        0
    } else {
        -1
    }
}

pub(super) fn sys_getdents64 (
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    buf: VirtualAddr,
    len: usize
) -> isize {
    unimplemented!();
}

pub(super) fn sys_lseek (
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    offset: isize,
    whence: usize
) -> isize {
    if let Some(file) = pcb.get_fd(fd) {
        if let Ok(pos) = file.write().lseek(whence, offset) {
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
    fd: isize,
    buf: VirtualAddr,
    len: usize,
) -> isize {
    let mut buf: PhysAddr = buf.into();
    let buf: &[u8] = buf.as_slice_mut(len);
    if let Some(file) = pcb.get_fd(fd) {
        match file.write().write(buf) {
            Ok(size) => {
                size as isize
            }
            Err(FileErr::PipeWriteWait) => {
                // 管道需要等待另一端读出，回退到ecall
                pcb.trapframe()["sepc"] -= 4;
                // 返回fd用于修改trapframe["a0"]，保证下次调用正确
                fd as isize
            }
            _ => {
                -1
            }
        }
    } else {
        -1
    }
}

pub(super) fn sys_read(
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    buf: VirtualAddr,
    len: usize,
) -> isize {
    let mut buf: PhysAddr = buf.into();
    let buf: &mut [u8] = buf.as_slice_mut(len);
    if let Some(file) = pcb.get_fd(fd) {
        match file.write().read(buf) {
            Ok(size) => {
                size as isize
            }
            Err(FileErr::PipeReadWait) => {
                // 管道需要等待另一端，回退到ecall
                pcb.trapframe()["sepc"] -= 4;
                // 返回fd用于修改trapframe["a0"]，保证下次调用正确
                fd as isize
            }
            _ => {
                -1
            }
        }
    } else {
        -1
    }

}