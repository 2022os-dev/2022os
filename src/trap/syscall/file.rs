use core::ops::Add;
use alloc::sync::Arc;
use alloc::string::String;

use crate::config::PATH_LIMITS;
use crate::mm::*;
use crate::process::*;
use crate::sbi::sbi_legacy_call;
use spin::MutexGuard;
use crate::vfs::*;

const AT_FDCWD: isize = -100;
fn get_str(pa: &PhysAddr) -> &str {
    let mut len = 0;
    while len < PATH_LIMITS {
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
    let fullpath = if is_absolute_path(path) {
        // 绝对路径,忽略fd
        log!("syscall":"mkdirat">"absolute path: {}", path);
        String::from(path)
    } else if fd == AT_FDCWD {
        // 使用cwd作为父节点
        log!("syscall":"mkdirat">"relative path: {}", path);
        if let Some('/') = pcb.cwd.chars().last() {
            pcb.cwd.clone() + path
        } else {
            pcb.cwd.clone() + "/" + path
        }
    } else {
        return -1
    };
    let (rest, name) = rsplit_path(fullpath.as_str());
    if name == "." || name == ".." {
        return -1
    }
    if let Some(_) = match rest {
        Some(rest) => {
            parse_path(&pcb.root, rest).and_then(|inode| {
                inode.create(name, mode, InodeType::Directory)
            }).ok()
        }
        None => {
            pcb.root.create(name, mode, InodeType::Directory).ok()
        }
    } {
        0
    } else {
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
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let fullpath = if is_absolute_path(path) {
        String::from(path)
    } else {
        match parse_path(&pcb.root, pcb.cwd.as_str()) {
            Ok(_) => {
                if let Some('/') = pcb.cwd.chars().last() {
                    pcb.cwd.clone() + path
                } else {
                    pcb.cwd.clone() + "/" + path
                }
            }
            Err(e) => {
                log!("syscall":"chdir">"error {:?}", e);
                return -1
            }
        }
    };
    match parse_path(&pcb.root, fullpath.as_str()) {
        Ok(_) => {
            pcb.cwd = fullpath;
            0
        }
        Err(e) => {
            log!("syscall":"chdir">"error {:?}", e);
            -1
        }
    }
}

pub(super) fn sys_openat (
    pcb: &mut MutexGuard<Pcb>,
    dirfd: isize,
    path: VirtualAddr,
    flags: usize,
    mode: usize
) -> isize {
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let flags = OpenFlags::from_bits(flags).unwrap();
    let mode = FileMode::from_bits(mode).unwrap();
    // 判断path使用的父节点
    let fullpath = if is_absolute_path(path) {
        log!("syscall":"openat">"absolute path: {}", path);
        String::from(path)
    } else if dirfd == AT_FDCWD {
        // 使用cwd作为父节点
        log!("syscall":"openat">"relative path: {}", path);
        if let Some('/') = pcb.cwd.chars().last() {
            pcb.cwd.clone() + path
        } else {
            pcb.cwd.clone() + "/" + path
        }
    } else {
        log!("syscall":"openat">"invalid combination of dirfd and path: {}, {}", dirfd, path);
        return -1
    };
    // 1. 首先尝试直接解析
    match parse_path(&pcb.root, fullpath.as_str()) {
        Ok(inode) => {
            // 2. 解析成功，直接打开
            log!("syscall":"openat">"path exists: {}", fullpath);
            if let Ok(fd) = File::open(inode, flags).and_then(|file| {
                pcb.fds_insert(file).ok_or(FileErr::NotDefine)
            }) {
                // todo: O_TRUNC截断
                return fd as isize
            }
        }
        // 3. 解析失败，目录内没有该path，如果flags包含create，尝试创建文件
        Err(FileErr::InodeNotChild) if flags.contains(OpenFlags::CREATE) => {
            log!("syscall":"openat">"path not exists, create: {}", fullpath);
            let (rest, comp) = rsplit_path(fullpath.as_str());
            if comp == "." || comp == ".." {
                return -1
            }
            let parent = if let Some(rest) = rest {
                // 4. 解析Path中除去最后一个节点的剩余节点
                parse_path(&pcb.root, rest)
            } else {
                Ok(pcb.root.clone())
            };
            // 5. 判断节点是否解析成功
            if let Ok(addedfd) = parent.and_then(|inode| {
                // 6. 解析成功则新建文件
                if flags.contains(OpenFlags::DIRECTROY) {
                    inode.create(comp, mode, InodeType::Directory)
                } else {
                    inode.create(comp, mode, InodeType::File)
                }
            }).and_then(|inode| {
                File::open(inode, flags)
            }).and_then(|file| {
                // 7. 将打开的文件加入指定的fd中
                pcb.fds_insert(file).ok_or(FileErr::FdInvalid)
            }) {
                // 8. 成功，返回新的fd
                return addedfd as isize
            }
        }
        _ => {
            log!("syscall":"openat">"path not exists: {}", fullpath);
        }
    }
    -1
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
    let mut buf: PhysAddr = buf.into();
    let mut buf = buf.as_slice_mut(len);
    let file = pcb.get_fd(fd);
    match file {
        Some(file) => {
            match file.write().get_dirents(&mut buf) {
                Ok(size) => {
                    return size as isize
                }
                Err(FileErr::InodeEndOfDir) => {
                    log!("syscall":"getdents64">"dirent eof");
                    return 0
                }
                Err(_) => {
                    return -1
                }
            }
        }
        None => {
            log!("syscall":"getdents64">"invalid fd {}", fd);
            -1
        }
    }
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
                // 管道需要等待另一端，回退到ecall
                log!("vfs":"sys_write">"waiting fd({})", fd);
                pcb.trapframe()["sepc"] -= 4;
                pcb.block_fn = Some(Arc::new(move |pcb| {
                    if let Some(_) = pcb.get_fd(fd).and_then(|file| {
                        file.try_write().and_then(|file| {
                            // 通过write_ready判断是否可以写
                            if file.get_inode().write_ready() {
                                Some(())
                            } else {
                                None
                            }
                        })
                    }) {
                        return true
                    }
                    false
                }));
                pcb.set_state(PcbState::Blocking);
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
                pcb.block_fn = Some(Arc::new(move |pcb| {
                    if let Some(_) = pcb.get_fd(fd).and_then(|file| {
                        file.try_write().and_then(|file| {
                            // 通过read_ready判断是否可以读
                            if file.get_inode().read_ready() {
                                Some(())
                            } else {
                                None
                            }
                        })
                    }) {
                        return true
                    }
                    false
                }));
                pcb.set_state(PcbState::Blocking);
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