#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;

fn main() {
    
    let num: u8 = 5;
    let pointer = &num as *const u8;
    syscall_mount("/dev/sdb1\0", "/\0", "fat32\0", 0 as u32, pointer);

    syscall_mount("/dev/sdb2\0", "/mnt\0", "ext4\0", 0 as u32, pointer);
    
    syscall_mount("/dev/sdb3\0", "/\0", "ext4\0", 0 as u32, pointer);

}