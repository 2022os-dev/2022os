#![no_std]
#![no_main]

mod syscall;
mod runtime;

pub fn main() {
    let nums = ["0\n", "1\n", "2\n", "3\n", "4\n", "5\n", "6\n", "7\n", "8\n", "9\n"];
    for i in 0..=9 {
        syscall::syscall_write(1, nums[i].as_bytes());
        syscall::syscall_yield();
    }
    syscall::syscall_exit(0);
}
