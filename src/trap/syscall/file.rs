use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ops::Add;

use crate::config::*;
use crate::mm::*;
use crate::process::*;
use crate::sbi::sbi_legacy_call;
use crate::vfs::*;
use crate::user::INT;
use spin::MutexGuard;

const AT_FDCWD: isize = -100;
fn get_str(pa: &PhysAddr) -> &str {
    let mut len = 0;
    // Note: 将从用户空间传入的字符串大小作一个限制
    while len < PATH_LIMITS {
        if unsafe { *(pa.0 as *const u8).add(len) } != 0 {
            len += 1;
        } else {
            break;
        }
    }
    unsafe { core::str::from_utf8_unchecked(pa.as_slice(len)) }
}

fn get_cstr(pa: &PhysAddr) -> &str {
    let mut len = 0;
    // Note: 将从用户空间传入的字符串大小作一个限制
    while len < PATH_LIMITS {
        if unsafe { *(pa.0 as *const u8).add(len) } != 0 {
            len += 1;
        } else {
            len += 1;
            break;
        }
    }
    unsafe { core::str::from_utf8_unchecked(pa.as_slice(len)) }
}

fn get_usize_array(pa: &PhysAddr) -> &[usize] {
    // 可能数组过大或末尾未置0导致循环无法停止
    let mut len = 0;
    loop {
        if unsafe { *(pa.0 as *const usize).add(len) } != 0 {
            len += 1;
        } else {
            len += 1;
            break;
        }
    }
    unsafe { core::slice::from_raw_parts(pa.0 as *const usize, len) }
}

// 将fd和path的组合解析为(Inode, String)的元组，方便parse_path的调用
fn make_path_tuple(pcb: &Pcb, fd: isize, path: &str) -> Option<(Inode, String)> {
    if is_absolute_path(path) {
        Some((pcb.root.clone(), String::from(path)))
    } else if fd == AT_FDCWD {
        // 如果是相对于当前cwd的路径，构造一个绝对路径
        if let Some('/') = pcb.cwd.chars().last() {
            Some((pcb.root.clone(), pcb.cwd.clone() + path))
        } else {
            Some((pcb.root.clone(), pcb.cwd.clone() + "/" + path))
        }
    } else {
        match pcb.get_fd(fd) {
            Some(fd) => Some((fd.read().inode.clone(), String::from(path))),
            None => None,
        }
    }
}

// 获取由一个Inode和相对于Inode的路径指定的文件的父Inode
// pre-cond: 路径的最后节点必须不能是"."或".."
fn get_parent_inode<'a, 'b>(node: &'a Inode, path: &'b str) -> Result<(Inode, &'b str), FileErr> {
    let (rest, name) = rsplit_path(path);
    if name == "." || name == ".." || name.len() == 0 {
        return Err(FileErr::NotDefine);
    }
    if let Some(rest) = rest {
        let parent = parse_path(&node, rest)?;
        Ok((parent, name))
    } else {
        Ok((node.clone(), name))
    }
}

pub(super) fn sys_getcwd(pcb: &mut MutexGuard<Pcb>, buf: VirtualAddr, len: usize) -> VirtualAddr {
    if buf.0 == 0 {
        // 由系统分配缓存区，不支持
        VirtualAddr(0)
    } else {
        let mut buf: PhysAddr = buf.into();
        // Fixme: 考虑 len长度限制
        buf.write(pcb.cwd.as_bytes());
        // 最后一位写0
        (buf + pcb.cwd.len()).write_bytes('\0' as u8, 1);
        VirtualAddr(buf.0)
    }
}

pub(super) fn sys_mkdirat(
    pcb: &mut MutexGuard<Pcb>,
    dirfd: isize,
    path: VirtualAddr,
    mode: usize,
) -> isize {
    let phys: PhysAddr = path.into();
    let path = get_str(&phys);
    let mode = FileMode::from_bits(mode).unwrap();
    let path_tuple = make_path_tuple(&mut *pcb, dirfd, path);
    if path_tuple.is_none() {
        return -1;
    }

    let (node, path) = path_tuple.unwrap();
    match get_parent_inode(&node, path.as_str()) {
        Ok((parent, name)) => {
            if let Ok(_) = parent.create(name, FileMode::empty(), InodeType::Directory) {
                return 0;
            }
        }
        Err(e) => {
            log!("syscall":"mkdirat">"{:?}", e);
        }
    }
    return -1;
}

pub(super) fn sys_linkat(
    pcb: &mut MutexGuard<Pcb>,
    olddirfd: isize,
    oldpath: VirtualAddr,
    newdirfd: isize,
    newpath: VirtualAddr,
    _: usize,
) -> isize {
    let oldpath: PhysAddr = oldpath.into();
    let oldpath = get_str(&oldpath);
    let old_path_tuple = make_path_tuple(&mut *pcb, olddirfd, oldpath);
    if old_path_tuple.is_none() {
        // fd和path的组合不正确
        log!("syscall":"linkat">"invalid combinations old(fd:{}, path:\"{}\"", olddirfd, oldpath);
        return -1;
    }
    let (oldnode, oldpath) = old_path_tuple.unwrap();
    let newpath: PhysAddr = newpath.into();
    let newpath = get_str(&newpath);
    let new_path_tuple = make_path_tuple(&mut *pcb, newdirfd, newpath);
    if new_path_tuple.is_none() {
        // fd和path的组合不正确
        log!("syscall":"linkat">"invalid combinations neew(fd:{}, path:\"{}\"", newdirfd, newpath);
        return -1;
    }
    let (newnode, newpath) = new_path_tuple.unwrap();
    match parse_path(&oldnode, oldpath.as_str()).and_then(|oldinode| {
        get_parent_inode(&newnode, newpath.as_str()).and_then(|(parent, name)| {
            parent.create(name, FileMode::empty(), InodeType::HardLink(oldinode))
        })
    }) {
        Ok(_) => {
            log!("syscall":"linkat""successed">"{}", newpath);
            0
        }
        Err(e) => {
            log!("syscall":"linkatl""failed">"{:?}", e);
            -1
        }
    }
}

pub(super) fn sys_unlinkat(
    pcb: &mut MutexGuard<Pcb>,
    dirfd: isize,
    path: VirtualAddr,
    flags: usize,
) -> isize {
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let path_tuple = make_path_tuple(&mut *pcb, dirfd, path);
    if path_tuple.is_none() {
        // fd和path的组合不正确
        log!("syscall":"unlinkat">"invalid combinations (fd:{}, path:\"{}\"", dirfd, path);
        return -1;
    }
    let (node, path) = path_tuple.unwrap();
    match get_parent_inode(&node, path.as_str()).and_then(|(parent, name)| {
        // todo: REMOVEDIR
        parent.unlink_child(name, false)
    }) {
        Ok(linknum) => {
            log!("syscall":"unlinkat""successed">"remain linknum {}", linknum);
            0
        }
        Err(e) => {
            log!("syscall":"unlinkat""failed">"{:?}", e);
            -1
        }
    }
}

pub(super) fn sys_pipe(pcb: &mut MutexGuard<Pcb>, pipe: VirtualAddr) -> isize {
    let mut phys: PhysAddr = pipe.into();
    // sizeof(int) == 4
    let pipe: &mut [INT; 2] = phys.as_mut();
    if let Ok((reader, writer)) = make_pipe().and_then(|(reader, writer)| {
        pcb.fds_insert(reader)
            .and_then(|rfd| pcb.fds_insert(writer).and_then(|wfd| Some((rfd, wfd))))
            .ok_or(FileErr::NotDefine)
    }) {
        pipe[0] = reader as INT;
        pipe[1] = writer as INT;
        0
    } else {
        log!("syscall":"pipe">"fail");
        -1
    }
}

pub(super) fn sys_dup(pcb: &mut MutexGuard<Pcb>, fd: isize) -> isize {
    match pcb.get_fd(fd).and_then(|fd| pcb.fds_insert(fd)) {
        Some(fd) => fd as isize,
        None => -1,
    }
}

pub(super) fn sys_dup3(pcb: &mut MutexGuard<Pcb>, oldfd: isize, newfd: isize) -> isize {
    // Fixme: 2021初赛中没有指定flags选项
    if oldfd == newfd {
        return newfd as isize;
    }
    match pcb.get_fd(oldfd).and_then(|fd| {
        pcb.fds_close(newfd);
        if pcb.fds_add(newfd, fd) {
            Some(())
        } else {
            None
        }
    }) {
        Some(_) => newfd as isize,
        None => -1,
    }
}

pub(super) fn sys_chdir(pcb: &mut MutexGuard<Pcb>, path: VirtualAddr) -> isize {
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let path_tuple = make_path_tuple(&mut *pcb, AT_FDCWD, path);
    if path_tuple.is_none() {
        return -1;
    }
    let (node, path) = path_tuple.unwrap();
    match parse_path(&node, path.as_str()) {
        Ok(_) => {
            pcb.cwd = path;
            0
        }
        Err(e) => {
            log!("syscall":"chdir">"error {:?}", e);
            -1
        }
    }
}

pub(super) fn sys_openat(
    pcb: &mut MutexGuard<Pcb>,
    dirfd: isize,
    path: VirtualAddr,
    flags: usize,
    mode: usize,
) -> isize {
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let flags = OpenFlags::from_bits(flags).unwrap();
    let mode = FileMode::from_bits(mode).unwrap();

    let path_tuple = make_path_tuple(&mut *pcb, dirfd, path);
    if path_tuple.is_none() {
        return -1;
    }
    let (node, path) = path_tuple.unwrap();
    // 不能使用get_parent_inode，因为路径的最后为"."或".."是合理的
    match parse_path(&node, path.as_str()).and_then(|inode| {
        File::open(inode, flags)
    }).and_then(|file| pcb.fds_insert(file).ok_or(FileErr::NotDefine)) {
        Ok(fd) => {
            return fd as isize
        }
        Err(FileErr::InodeNotChild) => {
            return get_parent_inode(&node, path.as_str()).and_then(|(parent, name)| {
                parent.create(name, FileMode::empty(), InodeType::File)
            }).and_then(|child| File::open(child, flags))
                .and_then(|file| pcb.fds_insert(file).ok_or(FileErr::NotDefine))
                .unwrap_or_else(|e| {
                    log!("syscall":"openat">"create error {:?}", e);
                    -1 as isize as usize
                }) as isize;
        }
        Err(_) => {
            return -1
        }
    };
}

pub(super) fn sys_close(pcb: &mut MutexGuard<Pcb>, fd: isize) -> isize {
    if pcb.fds_close(fd) {
        0
    } else {
        -1
    }
}

pub(super) fn sys_getdents64(
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    buf: VirtualAddr,
    len: usize,
) -> isize {
    let mut buf: PhysAddr = buf.into();
    let mut buf = buf.as_slice_mut(len);
    let file = pcb.get_fd(fd);
    match file {
        Some(file) => match file.write().get_dirents(&mut buf) {
            Ok(size) => return size as isize,
            Err(FileErr::InodeEndOfDir) => {
                log!("syscall":"getdents64">"dirent eof");
                return 0;
            }
            Err(_) => return -1,
        },
        None => {
            log!("syscall":"getdents64">"invalid fd {}", fd);
            -1
        }
    }
}

pub(super) fn sys_lseek(
    pcb: &mut MutexGuard<Pcb>,
    fd: isize,
    offset: isize,
    whence: usize,
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
            Ok(size) => size as isize,
            Err(FileErr::PipeWriteWait) => {
                // 需要等待另一端，回退到ecall
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
                        return true;
                    }
                    false
                }));
                pcb.set_state(PcbState::Blocking);
                // 返回fd用于修改trapframe["a0"]，保证下次调用正确
                fd as isize
            }
            Err(e) => {
                log!("syscall":"sys_write">"error {:?}", e);
                -1
            }
        }
    } else {
        log!("syscall":"sys_write">"fd invalid");
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
            Ok(size) => size as isize,
            Err(FileErr::PipeReadWait) => {
                // 需要等待另一端，回退到ecall
                log!("vfs":"sys_read">"waiting fd({})", fd);
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
                        return true;
                    }
                    false
                }));
                pcb.set_state(PcbState::Blocking);
                // 返回fd用于修改trapframe["a0"]，保证下次调用正确
                fd as isize
            }
            Err(e) => {
                log!("syscall":"sys_read">"error {:?}", e);
                -1
            }
        }
    } else {
        log!("syscall":"sys_read">"fd invalid");
        -1
    }
}

pub(super) fn sys_execve(
    pcb: &mut MutexGuard<Pcb>,
    path: VirtualAddr,
    argv: VirtualAddr,
    envp: VirtualAddr,
) {
    let path: PhysAddr = path.into();
    let path = get_str(&path);
    let argv: PhysAddr = argv.into();
    let argv = get_usize_array(&argv);
    let envp: PhysAddr = envp.into();
    let envp = get_usize_array(&envp);

    // 构造路径tuple
    let path_tuple = make_path_tuple(&mut *pcb, AT_FDCWD, path);
    if path_tuple.is_none() {
        log!("syscall":"execve">"invalid path {}", path);
        return;
    }
    let (node, path) = path_tuple.unwrap();
    log!("execve":>"path {}", path);
    if let Ok(_) = parse_path(&node, path.as_str()).and_then(|inode| {
        let mut ms = MemorySpace::from_elf_inode(inode)?;
        // 用户栈底的物理地址(栈由上往下增长)
        let mut user_stack_high = ms.user_stack.offset_phys(USER_STACK_SIZE);
        // 将argv和envp数组拷贝到用户栈上
        match copy_execve_str_array(user_stack_high, argv, user_stack_high).and_then(
            |(argv_pa, start_pa)| {
                copy_execve_str_array(user_stack_high, envp, start_pa)
                    .and_then(|(envp_pa, stack_pa)| Ok((argv_pa, envp_pa, stack_pa)))
            },
        ) {
            Ok((argv_pa, envp_pa, stack_pa)) => {
                log!("execve":>"copying argv, envp");
                let sp = MemorySpace::get_stack_sp().0 - (user_stack_high.0 - stack_pa.0);
                // 更新栈
                ms.trapframe()["sp"] = sp;
                // 更新args
                ms.trapframe()["a0"] = argv.len() - 1;
                // 计算argv数组的虚拟地址
                let a1 = MemorySpace::get_stack_sp().0 - (user_stack_high.0 - argv_pa.0);
                ms.trapframe()["a1"] = a1;
                // 计算envp数组的虚拟地址
                let a2 = MemorySpace::get_stack_sp().0 - (user_stack_high.0 - envp_pa.0);
                ms.trapframe()["a2"] = a2;

                let sp = ms.trapframe()["sp"];
                // 释放了原本的用户MemorySpace，不能再读写了
                pcb.memory_space = ms;
                Ok(())
            }
            Err(_) => Err(FileErr::NotDefine),
        }
    }) {
        log!("syscall":"execve""success">"");
    } else {
        log!("syscall":"execve""fail">"");
    }
}

/**
 *       |--------------| <- stack_pa
 *       |       0      |
 *       |--------------|
 *       |  str_arr[-2] |
 *       |--------------|
 *       |  str_arr[-3] | -----\
 *       |--------------|       \
 *       |      ...     |       |
 *       |--------------|       |
 *       |  str_arr[0]  | --- \ |
 *       |--------------|     | |
 *       |     str[0]   |     | |
 *       |--------------| <---- |
 *       |     str[1]   |       |
 *       |--------------|       |
 *       |      ...     |       |
 *       |--------------| <-----|
 */
// 将execve的argv和envp字符数组复制道新进程的栈中, 返回接下来的栈地址
fn copy_execve_str_array(
    stack_high: PhysAddr,
    str_array: &[usize],
    stack_pa: PhysAddr,
) -> Result<(PhysAddr, PhysAddr), ()> {
    // 计算字符串复制的地址
    let mut str_pa = stack_pa - size_of::<usize>() * (str_array.len());
    // 计算数组复制的地址
    let mut arr_pa = str_pa;
    let arr_pa_ret = arr_pa;
    for &va in str_array {
        // va为0，数组结尾
        if va == 0 {
            let arr_i: &mut usize = arr_pa.as_mut();
            *arr_i = 0;
            break;
        }
        // 获得str_array指向的字符串
        let str_pa_orignal: PhysAddr = VirtualAddr(va).into();
        let str_original = get_cstr(&str_pa_orignal);
        log!("execve":"copy_str_array">"str({}): \"{}\"", str_original.len(), str_original);

        // 通过字符串长度计算要写入的地址
        str_pa = str_pa - str_original.len();
        // todo: 判断str_pa是否溢出栈

        // 计算虚拟地址，写入栈中
        let arr_i: &mut usize = arr_pa.as_mut();
        *arr_i = MemorySpace::get_stack_sp().0 - (stack_high.0 - str_pa.0);
        // 复制字符串
        str_pa.write(str_original.as_bytes());
        // 指向下一个
        arr_pa = arr_pa + size_of::<usize>();
    }
    Ok((arr_pa_ret, str_pa))
}
