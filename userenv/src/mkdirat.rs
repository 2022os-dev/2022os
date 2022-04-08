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

    // 使用绝对路径创建文件夹
    assert!(syscall_mkdirat(0, "/absolute\0", mode) == 0);
    // 路径解析错误
    assert!(syscall_mkdirat(0, "/absolute/a/b\0", mode) == -1);
    // 创建重复文件夹
    assert!(syscall_mkdirat(0, "/absolute\0", mode) == -1);
    // 创建根目录
    assert!(syscall_mkdirat(0, "/\0", mode) == -1);

    // 使用相对路径创建文件夹
    assert!(syscall_mkdirat(-100, "relative\0", mode) == 0);
    // 路径解析错误
    assert!(syscall_mkdirat(-100, "relative/a/b\0", mode) == -1);
    // 创建重复文件夹
    assert!(syscall_mkdirat(-100, "relative\0", mode) == -1);
    // 创建空文件
    assert!(syscall_mkdirat(-100, "\0", mode) == -1);

    // 使用子进程查看文件
    if syscall_fork() > 0 {
        return;
    }

    // 创建重复文件夹
    assert!(syscall_mkdirat(0, "/absolute\0", mode) == -1);
    // 路径解析错误
    assert!(syscall_mkdirat(0, "/absolute/a/b\0", mode) == -1);
    // 创建重复文件夹
    assert!(syscall_mkdirat(-100, "relative\0", mode) == -1);

}