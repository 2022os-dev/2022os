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
    let forkret = syscall::syscall_fork();
    if forkret > 0 {
        syscall::syscall_write(1, "I'm parent\n".as_bytes());
    } else {
        syscall::syscall_write(1, "I'm child\n".as_bytes());
        let nums = ["0\n", "1\n", "2\n", "3\n", "4\n", "5\n", "6\n", "7\n", "8\n", "9\n"];
        for i in 0..=9 {
            syscall::syscall_write(1, nums[i].as_bytes());
            syscall::syscall_yield();
        }
    }
    syscall::syscall_exit(0);
}
