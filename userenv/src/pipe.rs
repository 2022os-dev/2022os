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
    let mut fds: [INT; 2] = [0,0];
    let hello = "hello world, it is a good day";
    let mut buf: [u8; 29] = [0; 29];
    assert!(syscall_pipe(&mut fds) == 0);
    // 判断可读可写
    assert!(syscall_read(fds[1], &mut buf) == -1);
    assert!(syscall_write(fds[0], hello.as_bytes()) == -1);
    // 管道写入
    syscall_write(fds[1], hello.as_bytes());
    if syscall_fork() > 0 {
        return;
    }
    // 另一端读出
    syscall_read(fds[0], &mut buf);
    let s = unsafe {core::str::from_utf8_unchecked(&buf)};
    println!("{}", s);
}