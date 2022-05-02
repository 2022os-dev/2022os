use core::panic::PanicInfo;
use core::arch::asm;
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

fn get_str(pa: *const u8) -> &'static str {
    let mut len = 0;
    // Note: 将从用户空间传入的字符串大小作一个限制
    while len < 512 {
        let a = unsafe { * pa.add(len )};
        if a != 0 {
            len += 1;
        } else {
            break;
        }
    }
    unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(pa, len)) }
}

fn get_array(pa: *const usize) -> &'static [*const u8] {
    // 可能数组过大或末尾未置0导致循环无法停止
    let mut len = 0;
    loop {
        if unsafe { *pa.add(len) } != 0 {
            len += 1;
        } else {
            break;
        }
    }
    unsafe { core::slice::from_raw_parts(pa as *const *const u8, len) }
}

pub fn get_argv(idx: usize) -> Option<&'static str> {
    unsafe {
        match ARGV.and_then(|arr| {
            arr.get(idx)
        }) {
            Some(&p) => {
                Some(get_str(p))
            }
            None => {
                None
            }
        }
    }
}

pub fn get_envp(idx: usize) -> Option<&'static str> {
    unsafe {
        match ENVP.and_then(|arr| {
            arr.get(idx)
        }) {
            Some(&p) => {
                Some(get_str(p))
            }
            None => {
                None
            }
        }
    }

}

static mut ARGV: Option<&'static [*const u8]> = None;
static mut ARGC: usize = 0;
static mut ENVP: Option<&'static [*const u8]> = None;

#[no_mangle]
extern "C" fn _start(_argc: usize, _argv: *const *const u8, _envp: *const *const u8) {
    println!("fxxk!");
    unsafe {
        ARGC = _argc;
        ARGV = if _argv as usize != 0 { Some(core::slice::from_raw_parts(_argv, _argc)) }  else { None };
        ENVP = if _envp as usize != 0 { Some(get_array(_envp as *const usize)) } else { None };
    };
    crate::main();
    crate::syscall::syscall_exit(0);
}