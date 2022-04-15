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
    assert!(syscall_mkdirat(AT_FDCWD, "linkdir\0", FileMode::empty()) == 0);

    //@ linkat
    // 创建一个file1文件
    let fd1 = syscall_openat(AT_FDCWD, "linkdir/file1\0", OpenFlags::RDWR | OpenFlags::CREATE, FileMode::empty());
    assert!(fd1 > 0);

    // 链接到文件
    assert!(syscall_linkat(AT_FDCWD, "linkdir/file1\0", AT_FDCWD, "linkdir/file2\0", 0) == 0);

    // 打开链接文件
    let fd2 = syscall_openat(AT_FDCWD, "linkdir/file2\0", OpenFlags::RDWR, FileMode::empty());
    assert!(fd2 > 0);

    let content = "12345678901234567890";

    // 往文件写入数据
    assert!(syscall_write(fd1, content.as_bytes()) == content.len() as INT);

    // 从链接处读出数据
    let mut buf: [u8; 20] = [0; 20];
    assert!(syscall_read(fd2, &mut buf) == content.len() as INT);

    // 比较数据是否一致
    assert!(unsafe {core::str::from_utf8_unchecked(&buf)} == content);

    //@ unlink

    // 未删除前打开成功
    assert!(syscall_openat(AT_FDCWD, "linkdir/file2\0", OpenFlags::RDWR, FileMode::empty()) > 0);

    assert!(syscall_unlinkat(AT_FDCWD, "linkdir/file2\0", 0) == 0);
    // 尝试打开删除的文件失败
    assert!(syscall_openat(AT_FDCWD, "linkdir/file2\0", OpenFlags::RDWR, FileMode::empty()) == -1);

    assert!(syscall_unlinkat(AT_FDCWD, "linkdir/file1\0", 0) == 0);
    // 尝试打开删除的文件失败
    assert!(syscall_openat(AT_FDCWD, "linkdir/file1\0", OpenFlags::RDWR, FileMode::empty()) == -1);
}