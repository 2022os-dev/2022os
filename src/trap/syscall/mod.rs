#![allow(unused)]
mod file;
mod process;
mod mm;
mod signal;
use alloc::sync::Arc;
use crate::mm::address::*;
use crate::process::*;
use crate::process::cpu::current_hart;
use crate::task::*;
use file::*;
use mm::*;
use process::*;
use signal::*;

const SYSCALL_GETCWD: usize = 17;
const SYSCALL_DUP: usize = 23;
const SYSCALL_DUP3: usize = 24;
const SYSCALL_FCNTL: usize = 25;
const SYSCALL_IOCTL: usize = 29;
const SYSCALL_MKDIRAT: usize = 34;
const SYSCALL_UNLINKAT: usize = 35;
const SYSCALL_LINKAT: usize = 37;
const SYSCALL_UMOUNT2: usize = 39;
const SYSCALL_MOUNT: usize = 40;
const SYSCALL_FACCESSAT: usize = 48;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_OPENAT: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_GETDENTS64: usize = 61;
const SYSCALL_LSEEK: usize = 62;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_WRITEV: usize = 66;
const SYSCALL_SENDFILE: usize = 71;
const SYSCALL_PSELECT6: usize = 72;
const SYSCALL_READLINKAT: usize = 78;
const SYSCALL_NEW_FSTATAT: usize = 79;
const SYSCALL_FSTAT: usize = 80;
const SYSCALL_FSYNC: usize = 82;
const SYSCALL_UTIMENSAT: usize = 88;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_EXIT_GRUOP: usize = 94;
const SYSCALL_SET_TID_ADDRESS: usize = 96;
const SYSCALL_NANOSLEEP: usize = 101;
const SYSCALL_GETITIMER: usize = 102;
const SYSCALL_SETITIMER: usize = 103;
const SYSCALL_CLOCK_GETTIME: usize = 113;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_SIGACTION: usize = 134;
const SYSCALL_SIGRETURN: usize = 139;
const SYSCALL_TIMES: usize = 153;
const SYSCALL_UNAME: usize = 160;
const SYSCALL_GETRUSAGE: usize = 165;
const SYSCALL_GET_TIME_OF_DAY: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_GETPPID: usize = 173;
const SYSCALL_GETUID: usize = 174;
const SYSCALL_GETEUID: usize = 175;
const SYSCALL_GETGID: usize = 176;
const SYSCALL_GETEGID: usize = 177;
const SYSCALL_GETTID: usize = 177;
const SYSCALL_SBRK: usize = 213;
const SYSCALL_BRK: usize = 214;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_CLONE: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_MPROTECT: usize = 226;
const SYSCALL_WAIT4: usize = 260;
const SYSCALL_PRLIMIT: usize = 261;
const SYSCALL_RENAMEAT2: usize = 276;

// Not standard POSIX sys_call
const SYSCALL_FORK: usize = 451;
const SYSCALL_LS: usize = 500;
const SYSCALL_SHUTDOWN: usize = 501;
const SYSCALL_CLEAR: usize = 502;

pub fn syscall_handler() {
    log!("syscall":"handler" > "Enter");
    let pcb = current_pcb().unwrap();
    let mut pcblock = pcb.lock();
    log!(debug "pid {} in trap", pcblock.pid);
    let trapframe = pcblock.trapframe();
    let syscall_id = trapframe["a7"];

    // 指向下一条指令
    trapframe["sepc"] += 4;

    match syscall_id {
        SYSCALL_WRITE => {
            let fd = trapframe["a0"];
            let buf = VirtualAddr(trapframe["a1"]);
            let len = trapframe["a2"];
            drop(trapframe);
            log!("syscall":"write" > "pid({}) ({}, 0x{:x}, {})", pcblock.pid, fd, buf.0, len);
            pcblock.trapframe()["a0"] = sys_write(&mut pcblock, fd, buf, len) as usize;
            pcblock.reset_state();
        }
        SYSCALL_EXIT => {
            let xcode = trapframe["a0"];
            drop(trapframe);
            log!("syscall":"exit" > "pid({})->({})", pcblock.pid, xcode);
            sys_exit(&mut pcblock, xcode as isize);
        }
        SYSCALL_YIELD => {
            trapframe["a0"] = sys_yield() as usize;
            log!("syscall": "yield" > "pid({})", pcblock.pid);
            pcblock.reset_state();
        }
        SYSCALL_GETPID => {
            log!("syscall": "getpid"> "pid({})", pcblock.pid);
            pcblock.trapframe()["a0"] = sys_getpid(&pcblock) as usize;
            pcblock.reset_state();
        }
        SYSCALL_WAIT4 => {
            let pid = trapframe["a0"] as isize;
            let wstatus = VirtualAddr(trapframe["a1"]);
            let options = trapframe["a2"];
            let rusage = VirtualAddr(trapframe["a3"]);
            drop(trapframe);
            log!("syscall":"wait4" > "pid({}) ({}, 0x{:x}, 0x{:x})", pcblock.pid, pid, wstatus.0, options);
            let ret = sys_wait4(&mut pcblock, pid, wstatus, options, rusage);
            if let Ok(child_pid) = ret {
                pcblock.trapframe()["a0"] = child_pid;
                pcblock.reset_state();
            } else {
                // 回退上一条ecall指针，等待子进程信号
                pcblock.trapframe()["sepc"] -= 4;
                // 进入阻塞态，如果某个子进程退出则恢复
                pcblock.set_state(PcbState::Blocking(|pcb| { 
                    let pcblock = pcb.try_lock();
                    if let Some(pcblock) = pcblock {
                        for i in pcblock.children.iter() {
                            if let Some(childlock) = i.try_lock() {
                                if let PcbState::Exit(_) = childlock.state {
                                    return true;
                                }
                            }
                        }
                    }
                    false
                }));
                log!("syscall":"wait4" > "pid({})", pcblock.pid);
            }
        }
        SYSCALL_SBRK => {
            let inc = trapframe["a0"];
            drop(trapframe);
            log!("syscall":"sbrk" > "pid({}) (0x{:x})", pcblock.pid, inc);
            pcblock.trapframe()["a0"] = sys_sbrk(&mut pcblock, inc);
            pcblock.reset_state();
        }
        SYSCALL_BRK => {
            let va = VirtualAddr(trapframe["a0"]);
            drop(trapframe);
            log!("syscall":"brk" > "pid({}) (0x{:x})", pcblock.pid, va.0);
            pcblock.trapframe()["a0"] = sys_brk(&mut pcblock, va) as usize;
            pcblock.reset_state();
        }
        SYSCALL_KILL => {
            let pid = trapframe["a0"];
            let sig = trapframe["a1"];
            drop(trapframe);
            pcblock.trapframe()["a0"] = sys_kill(pid, sig) as usize;
            pcblock.reset_state();
        }
        SYSCALL_SIGACTION => {
            let signum = trapframe["a0"];
            let act = VirtualAddr(trapframe["a1"]);
            let oldact = VirtualAddr(trapframe["a2"]);
            drop(trapframe);
            log!("syscall":"sigaction">"pid({}) signal({:?})", pcblock.pid, crate::process::signal::Signal::from_bits(signum));
            pcblock.trapframe()["a0"] = sys_rt_sigaction(&mut pcblock, signum, act, oldact) as usize;
            pcblock.reset_state();
        }
        SYSCALL_SIGRETURN => {
            assert!(if let PcbState::SigHandling(_, _) = pcblock.state() { true } else { false});
            pcblock.signal_return();
            pcblock.set_state(PcbState::Ready);
        }
        SYSCALL_FORK => {
            drop(trapframe);
            log!("syscall":"fork" > "pid({}) ()", pcblock.pid);
            pcblock.trapframe()["a0"] = sys_fork(&mut pcblock) as usize;
            pcblock.reset_state();
        }
        _ => {
            println!("unsupported syscall {}", trapframe["a7"]);
            pcblock.reset_state();
        }
    }
    let state = pcblock.state;
    drop(pcblock);
    if let PcbState::Exit(_) = state {
    } else {
        scheduler_ready_pcb(pcb.clone());
    }
    // Note: 这里必须显式调用drop释放进程锁
    drop(pcb);
    schedule();
}
