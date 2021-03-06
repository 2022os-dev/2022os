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
        for i in 0..10 {
            syscall_yield();
        }
        syscall_kill(forkret, Signal::SIGUSR2);
        for i in 0..10 {
            syscall_yield();
        }
        syscall_kill(forkret, Signal::SIGKILL);
    } else {
        let sa = rt_sigaction {
            sa_handler: sig_handler as usize,
            sa_flags: SaFlags::empty().bits(),
            sa_mask: Signal::empty().bits(),
        };
        syscall_sigaction(Signal::SIGUSR2, &sa,  &sa);
        loop {}
    }
}

extern "C" fn sig_handler(sig: Signal) {
    for i in 0..5 {
        println!("Hello world")
    }
    syscall_sigreturn();
}