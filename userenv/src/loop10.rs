#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use console::*;
use syscall::*;

pub fn main() {
    
    for i in 0..=9 {
        println!("{}", i);
        syscall::syscall_yield();
    }
    let forkret = syscall::syscall_fork();
    if forkret > 0 {
        println!("I'm parent");
    } else {
        println!("I'm child");
        for i in 0..=9 {
            println!("{}", i);
            syscall::syscall_yield();
        }
    }
    return;
}
