#![no_std]
#![no_main]

mod syscall;
mod runtime;

pub fn main() {
    let str = "Hello world\n";
    syscall::syscall_write(1, str.as_bytes());
    syscall::syscall_exit(0);
}
