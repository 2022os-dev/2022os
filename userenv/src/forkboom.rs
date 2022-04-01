#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use console::*;
use syscall::*;

fn main() {
    loop {
        println!("{}", syscall_getpid());
        syscall_fork();
    }
}