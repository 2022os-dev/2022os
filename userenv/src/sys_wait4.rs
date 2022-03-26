#![no_std]
#![no_main]

mod syscall;
mod runtime;

use syscall::*;

fn main() {
    syscall_write(1, "I'am sys_wait4, try forking\n".as_bytes());
    let forkret = syscall_fork();
    if forkret > 0 {
        syscall_write(1, "sys_wait4 waiting all childrent\n".as_bytes());
        let mut wstatus = 0;
        let mut rusage = 0;
        let childpid = syscall_wait4(-1, &mut wstatus, 0, &mut rusage);
        if childpid == forkret {
            syscall_write(1, "sys_wait4 waited child pid is right\n".as_bytes());
        } else {
            syscall_write(1, "sys_wait4 waited child pid is wrong\n".as_bytes());
        }
        syscall_write(1, "sys_wait4 waited child exited is ".as_bytes());
        syscall_exit(wstatus as isize);
    } else {
        syscall_write(1, "sys_wait4 child exited with ".as_bytes());
        syscall_exit(121);
    }
}