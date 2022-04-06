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
    let name = "/gg\0";
    let flags = OpenFlags::CREATE | OpenFlags::RDWR;
    let mode = FileMode::empty();
    // 创建文件
    let fd = syscall_openat(0, name, flags, mode);
    if fd < 0 {
        println!("openat failed");
        return;
    }
    let mut buf: [u8; 10] = [0; 10];
    // 从控制台读入数据
    let ret = syscall_read(0, &mut buf);
    // 数据写回文件
    if ret < 0 {
        println!("read from console fail");
    }
    let ret = syscall_write(fd as usize, &buf);
    if ret < 0 {
        println!("write back file fail");
    }
    if syscall_fork() > 0 {
        return
    }
    // ########### CHILD ####################
    let fd = syscall_openat(10, name, flags, mode);
    // 子进程继续
    let mut buf: [u8; 10] = [0; 10];
    syscall_lseek(fd as usize, 0, SEEK_SET);
    // 读出文件
    let ret = syscall_read(fd as usize, &mut buf);
    if ret < 0 {
        println!("re read from file fail");
    }
    unsafe {
        println!("{}", core::str::from_utf8_unchecked(&buf));
    }
}