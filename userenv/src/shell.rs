#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;
use core::slice::from_raw_parts_mut;
use core::str::from_utf8_unchecked;
use core::mem::size_of;
use core::assert;

fn main() {
    let mut argv: [usize; 3] = [0; 3];
    let mut envp: [usize; 3] = [0; 3];
    let mut path: [u8; 512] = [0; 512];
    let mut i = 0;
    let mut ch: [u8; 1] = [0];
    print!("bash$ ");
    while syscall_read(0, &mut ch) > 0 {
        if ch[0] == 13 {
            // \n
            path[i] = '\0' as u8;
            if syscall_fork() > 0 {
                let mut wstatus = 0;
                let mut rusage = 0;
                let childpid = syscall_wait4(-1, &mut wstatus, 0, &mut rusage);
                path = [0; 512];
                i = 0;
                print!("bash$ ");
                continue;
            } else {
                let path = unsafe {from_utf8_unchecked(path.as_slice())};
                syscall_execve(path, &argv, &envp);
                println!("execve error: {}", path);
                return;
            }
        } else if ch[0] == 127 {
            // delete
            if i > 0 {
                i -= 1;
                path[i] = '\0' as u8;
                let path = unsafe {from_utf8_unchecked(path.as_slice())};
                print!("\nbash$ {}", path);
            }
            continue;
        }
        path[i] = ch[0];
        i += 1;
    }
    println!("shell end");
}