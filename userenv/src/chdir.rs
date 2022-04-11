#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;
use core::slice::from_raw_parts_mut;
use core::mem::size_of;
use core::assert;

fn main() {
    let mode = FileMode::empty();
    // AT_FDCWD = -100

    assert!(syscall_mkdirat(-100, "./chdir_dir\0", mode) == 0);
    assert!(syscall_mkdirat(-100, "./chdir_dir/dir\0", mode) == 0);

    // 重复创建失败
    assert!(syscall_mkdirat(-100, "./chdir_dir/dir\0", mode) == -1);

    // #####
    assert!(syscall_chdir("/chdir_dir\0") == 0);
    let mut buf: [u8; 11] = [0; 11];
    assert!(syscall_getcwd(&mut buf) > 0);
    assert!(unsafe{core::str::from_utf8_unchecked(&buf)} == "/chdir_dir\0");

    // 重复创建失败
    assert!(syscall_mkdirat(-100, "./dir\0", mode) == -1);

    assert!(syscall_mkdirat(-100, "./dir2\0", mode) == 0);
    assert!(syscall_mkdirat(0, "/chdir_dir/dir2\0", mode) == -1);

    // 不存在的路径
    assert!(syscall_chdir("/null\0") == -1);

    // 相对路径, cwd = "/chdir"
    assert!(syscall_chdir("./dir2\0") == 0);
    assert!(syscall_chdir("..\0") == 0);
    assert!(syscall_chdir("/chdir_dir/.\0") == 0);
}