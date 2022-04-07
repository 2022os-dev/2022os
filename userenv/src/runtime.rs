use core::panic::PanicInfo;
use crate::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[00;31mPanicked at {}:{}\x1b[00m",
            location.file(),
            location.line(),
        );
    }
    crate::syscall::syscall_exit(-1)
}

#[no_mangle]
fn _start() {
    crate::main();
    crate::syscall::syscall_exit(0);
}

