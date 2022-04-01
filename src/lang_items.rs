use crate::println;
use crate::process::cpu::hartid;
/// core library doesn't provide us
/// a panic_handler, we need one to
/// handle panic
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "{}: Panicked at {}:{} {}",
            hartid(),
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("{}: Panicked: {}", hartid(), info.message().unwrap());
    }
    crate::sbi::shutdown();
}
