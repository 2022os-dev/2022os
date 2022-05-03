use crate::sbi;
use core::fmt::{self, Write};

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            sbi::sbi_legacy_call(sbi::PUT_CHAR, [c as usize, 0, 0]);
        }
        Ok(())
    }
}

#[allow(unused)]
pub fn print(args: fmt::Arguments) {
    // 暂时锁住输出，防止多线程输出混乱
    Stdout.write_fmt(args).unwrap();
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
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! log{
    (debug $fmt: literal $(, $($arg: tt)+)?) => {
        // $crate::console::print(format_args!(concat!("\x1b[0;32m[Debug]:", $fmt, "\n\x1b[0m") $(, $($arg)+)?));
    };
    ($config: literal : $($header: literal)* >  $fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(feature = $config)]
        $crate::console::print(format_args!(concat!("{}: [", $config, "]", $("(", $header, ")",)* ": ", $fmt, "\n"), $crate::process::cpu::hartid() $(, $($arg)+)?));
    };
}
