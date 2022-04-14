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
    const SIZE: usize = 2048;
    let mut num = unsafe {
        from_raw_parts_mut(syscall_sbrk(SIZE * size_of::<usize>()) as *mut usize, SIZE)
    };
    syscall_brk(num.as_ptr() as *const u8);
    assert!(syscall_sbrk(0) == num.as_mut_ptr() as *mut u8);
    println!("Sbrk ptr is 0x{:x}", num.as_ptr() as usize);
    let mut count: usize = 0;
    for i in num.iter_mut() {
        *i = count;
        count += 1;
    }
    count = 0;
    for i in num {
        assert!(*i == count);
        count += 1;
    }
}