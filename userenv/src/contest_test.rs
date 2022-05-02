#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;
fn run_app(name: &str) {
    println!("Runing {}", name);
    let fork_ret = syscall_fork();
    if fork_ret > 0 {
        let mut wstatus = 0;
        let mut rusage = 0;
        syscall_wait4(-1, &mut wstatus, 0, &mut rusage);
    } else {
        let mut argv: [usize; 2] = [0; 2];
        let envp: [usize; 1] = [0];
        let mut n: [u8; 512] = [0; 512];
        n[..name.len()].copy_from_slice(name.as_bytes());
        argv[0] = n.as_ptr() as usize;
        syscall_execve(unsafe {
            core::str::from_utf8_unchecked(&n[..name.len() + 1])
        }, &argv, &envp);
    }
}
fn main() {
    run_app("brk");
    run_app("dup");
    run_app("fork");
    run_app("getpid");
    run_app("mmap");
    run_app("open");
    run_app("sleep");
    run_app("umount");
    run_app("waitpid");
    run_app("chdir");
    run_app("dup2");
    run_app("fstat");
    run_app("getppid");
 
    run_app("openat");
    run_app("uname");
    run_app("write");
 
    run_app("clone");
    run_app("execve");
    run_app("getcwd");
    run_app("gettimeofday");
 
    run_app("mount");
    run_app("pipe");
    run_app("unlink");
    run_app("yield");
 
    run_app("close");
    run_app("exit");
    run_app("getdents");
    run_app("mkdir_");
    run_app("munmap");
 
    run_app("read");
    run_app("times");
    run_app("wait");
}