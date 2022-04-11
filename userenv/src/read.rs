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
    let mut bytes: [u8; 5] = [0;5];
    if syscall_read(0, &mut bytes) < 0 {
        println!("Read error");
    } else {
        unsafe {
            println!("{}", core::str::from_utf8_unchecked(&bytes));
        }
    }
    if syscall_read(0, &mut bytes) < 0 {
        println!("Read error");
    } else {
        unsafe {
            println!("{}", core::str::from_utf8_unchecked(&bytes));
        }
    }
}