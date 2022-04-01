#![no_std]
#![no_main]

mod syscall;
mod console;
mod runtime;

use console::*;
use syscall::*;

fn main() {
    let mut tms = Tms {
        utime: 0,
        stime: 0,
        cutime: 0,
        cstime: 0
    };
    let forkret = syscall_fork();
    if forkret > 0 {
        let times = syscall_times(&mut tms);
        println!("(times, utime, stime, cutime, cstime): ({}, {}, {}, {}, {})", times, tms.utime, tms.stime, tms.cutime, tms.cstime);
        let mut xcode = 0;
        let mut rusage = 0;
        syscall_wait4(-1, &mut xcode, 0, &mut rusage);
        for i in 0..5 {
            let times = syscall_times(&mut tms);
            println!("(times, utime, stime, cutime, cstime): ({}, {}, {}, {}, {})", times, tms.utime, tms.stime, tms.cutime, tms.cstime);
        }
    } else {
    }
}