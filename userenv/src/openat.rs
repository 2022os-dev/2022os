#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;
use core::slice::from_raw_parts_mut;
use core::mem::size_of;
use core::assert;

fn main() {
    let mode = FileMode::empty();
    // AT_FDCWD = -100

    // 绝对路径创建文件
    let flags = OpenFlags::CREATE | OpenFlags::RDWR;
    let name = "/file\0";
    let fd = syscall_openat(0, name, flags, mode);
    assert!(fd >= 0);
    // 判断写入
    assert!(syscall_write(fd, name.as_bytes()) == name.len() as isize);
    let mut buf: [u8; 6] = [0; 6];
    assert!(syscall_lseek(fd, 0, SEEK_SET) == 0);
    assert!(syscall_read(fd, &mut buf) == name.len() as isize);
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == name);
    // EOF
    assert!(syscall_read(fd, &mut buf) == -1);
    let oldfd = fd;

    // 只读文件
    let fd = syscall_openat(0, name, OpenFlags::RDONLY, mode);
    assert!(fd >= 0);
    // 写失败
    assert!(syscall_write(fd, name.as_bytes()) == -1);
    // 读出
    assert!(syscall_read(fd, &mut buf) == name.len() as isize);
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == name);
    assert!(syscall_close(fd) == 0);
    drop(fd);

    // 只写文件
    let fd = syscall_openat(0, name, OpenFlags::WRONLY, mode);
    assert!(fd >= 0);
    let name = "/newname\0";
    // 读失败
    assert!(syscall_read(fd, &mut buf) == -1);
    // 写入 
    assert!(syscall_write(fd, name.as_bytes()) == name.len() as isize);
    assert!(syscall_close(fd) == 0);
    drop(fd);

    // 重新读出
    let mut buf: [u8; 9] = [0; 9];
    assert!(syscall_lseek(oldfd, 0, SEEK_SET) == 0);
    assert!(syscall_read(oldfd, &mut buf) == name.len() as isize);
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == name);


    // 相对路径创建文件
    let flags = OpenFlags::CREATE | OpenFlags::RDWR;
    let name = "rfile\0";
    let fd = syscall_openat(-100, name, flags, mode);
    assert!(fd >= 0);
    // 判断写入
    assert!(syscall_write(fd, name.as_bytes()) == name.len() as isize);
    let mut buf: [u8; 6] = [0; 6];
    assert!(syscall_lseek(fd, 0, SEEK_SET) == 0);
    assert!(syscall_read(fd, &mut buf) == name.len() as isize);
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == name);
    // EOF
    assert!(syscall_read(fd, &mut buf) == -1);
    let oldfd = fd;

    // 只读文件
    let fd = syscall_openat(-100, name, OpenFlags::RDONLY, mode);
    assert!(fd >= 0);
    // 写失败
    assert!(syscall_write(fd, name.as_bytes()) == -1);
    // 读出
    assert!(syscall_read(fd, &mut buf) == name.len() as isize);
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == name);
    assert!(syscall_close(fd) == 0);
    drop(fd);

    // 只写文件
    let fd = syscall_openat(-100, name, OpenFlags::WRONLY, mode);
    assert!(fd >= 0);
    let name = "newname\0";
    // 读失败
    assert!(syscall_read(fd, &mut buf) == -1);
    // 写入 
    assert!(syscall_write(fd, name.as_bytes()) == name.len() as isize);
    assert!(syscall_close(fd) == 0);
    drop(fd);

    // 重新读出
    let mut buf: [u8; 8] = [0; 8];
    assert!(syscall_lseek(oldfd, 0, SEEK_SET) == 0);
    assert!(syscall_read(oldfd, &mut buf) == name.len() as isize);
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == name);

}