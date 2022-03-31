#![no_std]
#![no_main]

mod syscall;
mod console;
mod runtime;

use console::*;
use syscall::*;

fn main() {
    let pid = syscall_getpid();
    let forkret = syscall_fork();
    if forkret > 0 {
        loop {
            println!("Looping");
        }
    } else {
        for i in 0..10 {
            syscall_yield();
        }
        syscall_kill(pid, Signal::SIGKILL);
    }
}