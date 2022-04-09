#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;
use core::slice::from_raw_parts_mut;
use core::mem::size_of;
use core::assert;

fn ls(path: &str) {
    let mut buf: [u8; 1024] = [0; 1024];
    let fd = syscall_openat(AT_FDCWD, path, OpenFlags::RDONLY, FileMode::empty());
    if fd < 0 {
        println!("invalid path {}", path);
        return;
    }
    while true {
        let nread = syscall_getdirents64(fd, &mut buf, 1024);
        if nread == 0 {
            println!("EOF");
            return;
        }
        if nread == -1 {
            println!("error");
            return ;
        }
        let nread = nread as usize;
        let dirents = unsafe {from_raw_parts_mut(&mut buf as *mut _ as *mut LinuxDirent, nread / size_of::<LinuxDirent>())};
        for i in 0..(nread/size_of::<LinuxDirent>()) {
            println!("dirent: {}", unsafe { core::str::from_utf8_unchecked(&dirents[i].d_name)});
        }
    }

}

fn main() {
    assert!(syscall_mkdirat(AT_FDCWD, "./dir1\0", FileMode::empty()) == 0);
    assert!(syscall_mkdirat(AT_FDCWD, "./dir2\0", FileMode::empty()) == 0);
    assert!(syscall_mkdirat(AT_FDCWD, "./dir3\0", FileMode::empty()) == 0);
    ls(".\0");
    ls("/dfadjfa\0");
}