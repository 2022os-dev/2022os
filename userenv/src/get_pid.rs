#![no_std]
#![no_main]

mod syscall;
mod runtime;

fn main() {
    syscall::syscall_write(1, "get pid ".as_bytes());
    let pid = syscall::syscall_getpid();
    syscall::syscall_exit(pid as isize);
}