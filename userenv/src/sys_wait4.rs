#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;

fn main() {
    println!("I'am sys_wait4, try forking");
    let forkret = syscall_fork();
    if forkret > 0 {
        println!("sys_wait4 waiting all childrent");
        let mut wstatus = 0;
        let mut rusage = 0;
        let childpid = syscall_wait4(-1, &mut wstatus, 0, &mut rusage);
        if childpid == forkret {
            println!("sys_wait4 waited child pid is right");
        } else {
            println!("sys_wait4 waited child pid is wrong");
        }
        println!("sys_wait4 waited child exited is {}", wstatus as isize);
    } else {
        for _ in 0..10 {
            syscall_yield();
        }
        println!("sys_wait4 child exited with ");
        syscall_exit(121);
    }
}