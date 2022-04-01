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
    println!("start to sleep");
    syscall_nanosleep(5, 0);
    println!("wake up");
}