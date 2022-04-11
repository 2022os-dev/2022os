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
    let mut argv: [usize; 3] = [0; 3];
    let mut envp: [usize; 3] = [0; 3];
    let argv0 = "hello\0";
    let argv1 = "world\0";
    let envp0 = "HOME=/\0";
    let envp1 = "CWD=/\0";
    argv[0] = argv0.as_ptr() as usize;
    argv[1] = argv1.as_ptr() as usize;
    envp[0] = envp0.as_ptr() as usize;
    envp[1] = envp1.as_ptr() as usize;
    println!("execve");
    syscall_execve("/loop10\0", &argv, &envp);
}