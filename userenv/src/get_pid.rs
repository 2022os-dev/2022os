#![no_std]
#![no_main]

mod syscall;
mod runtime;

fn main() {
    let pid = syscall::syscall_getpid();
    syscall::syscall_exit(pid as isize);
}