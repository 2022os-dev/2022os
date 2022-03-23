#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod syscall;

fn main() {
    let str = "Good morning\n";
    syscall::syscall_write(1, str.as_bytes());
}


#[no_mangle]
fn _start() {
    main();
    syscall::syscall_exit(0);
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    syscall::syscall_exit(-1)
}

