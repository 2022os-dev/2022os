
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
        let sa = rt_sigaction {
            sa_handler: sig_handler as usize,
            sa_flags: SaFlags::empty().bits(),
            sa_mask: Signal::empty().bits(),
        };
        syscall_sigaction(Signal::SIGCHLD, &sa,  &sa);
        for i in 0..10 {
            syscall_yield();
        }
    } else {
        for i in 0..3 {
            syscall_yield();
        }
    }
}

extern "C" fn sig_handler(sig: Signal) {
    println!("child dead {:?}",sig);
    syscall_exit(0);
}