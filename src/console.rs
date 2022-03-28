use crate::sbi;
use core::fmt::{self, Write};
use spin::Mutex;

static mut KERNEL_LOG: bool = true;

pub static STDOUTLOCK : Mutex<()> = Mutex::new(());

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            sbi::sbi_legacy_call(sbi::PUT_CHAR, [c as usize, 0, 0]);
        }
        Ok(())
    }
}
impl Stdout {
    pub fn is_log() -> bool {
        return unsafe { KERNEL_LOG };
    }
}

pub fn print(args: fmt::Arguments) {
    // 暂时锁住输出，防止多线程输出混乱
    // let lock = STDOUTLOCK.lock();
    Stdout.write_fmt(args).unwrap();
    // drop(lock);
}

#[allow(unused)]
pub fn turn_off_log() {
    unsafe {
        KERNEL_LOG = false;
    };
}
#[allow(unused)]
pub fn turn_on_log() {
    unsafe {
        KERNEL_LOG = true;
    };
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        let _lock = $crate::console::STDOUTLOCK.lock();
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
        drop(_lock);
    }
}

#[macro_export]
macro_rules! log{
    (debug $fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::Stdout::is_log() {
            let _lock = $crate::console::STDOUTLOCK.lock();
            $crate::console::print(format_args!(concat!("\x1b[0;32m[Debug]:", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
            drop(_lock);
        }
    };
}
