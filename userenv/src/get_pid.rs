#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use console::*;

fn main() {
    println!("get pid {}", syscall::syscall_getpid());
}