#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;

fn main() {
    
    
    syscall_umount2("/dev/sdb3\0", 0 as u32);

}