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
    let start = syscall_sbrk(0);
    let mut num = unsafe {
        from_raw_parts_mut(syscall_sbrk(10 * size_of::<usize>()), 10)
    };
    let brk = syscall_brk(num.as_ptr() as *const u8);
    assert!(syscall_sbrk(0) == num.as_mut_ptr());
    println!("Sbrk ptr is 0x{:x}", num.as_ptr() as usize);
    for i in num.iter_mut() {
        *i = 0xff;
    }
    for i in num {
        println!("0x{:X}", i);
    }
}