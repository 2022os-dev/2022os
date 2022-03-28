use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    crate::syscall::syscall_exit(-1)
}

#[no_mangle]
fn _start() {
    crate::main();
    crate::syscall::syscall_exit(0);
}

