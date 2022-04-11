#![allow(unused)]
mod file;
mod mm;
mod process;
mod signal;
use crate::config::RTCLK_FREQ;
use crate::mm::address::*;
use crate::process::cpu::{current_hart, get_time};
use crate::task::*;
use crate::{process::*, trap};
use alloc::sync::Arc;
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
        SYSCALL_GETCWD => {
            let buf = VirtualAddr(trapframe["a0"]);
            let size = trapframe["a1"];
            log!("syscall":"getcwd" > "pid({}) (0x{:x})", pcblock.pid, buf.0);
            pcblock.trapframe()["a0"] = sys_getcwd(&mut pcblock, buf, size).0;
        }
        SYSCALL_PIPE => {
            let pipe = VirtualAddr(trapframe["a0"]);
            log!("syscall":"pipe" > "pid({}) (0x{:x})", pcblock.pid, pipe.0);
            pcblock.trapframe()["a0"] = sys_pipe(&mut pcblock, pipe) as usize;
        }
        SYSCALL_DUP => {
            let fd = trapframe["a0"] as isize;
            log!("syscall":"dup" > "pid({}) ({})", pcblock.pid, fd);
            pcblock.trapframe()["a0"] = sys_dup(&mut pcblock, fd) as usize;
        }
        SYSCALL_DUP3 => {
            let oldfd = trapframe["a0"] as isize;
            let newfd = trapframe["a1"] as isize;
            log!("syscall":"dup3" > "pid({}) ({}, {})", pcblock.pid, oldfd, newfd);
            pcblock.trapframe()["a0"] = sys_dup3(&mut pcblock, oldfd, newfd) as usize;
        }
        SYSCALL_MKDIRAT => {
            let fd = trapframe["a0"] as isize;
            let path = VirtualAddr(trapframe["a1"]);
            let mode = trapframe["a2"];
            log!("syscall":"mkdirat" > "pid({}) ({}, 0x{:x})", pcblock.pid, fd, mode);
            pcblock.trapframe()["a0"] = sys_mkdirat(&mut pcblock, fd, path, mode) as usize;
        }
        SYSCALL_CHDIR => {
            let path = VirtualAddr(trapframe["a0"]);
            log!("syscall":"chdir" > "pid({}) (0x{:x})", pcblock.pid, path.0);
            pcblock.trapframe()["a0"] = sys_chdir(&mut pcblock, path) as usize;
        }
        SYSCALL_OPENAT => {
            let fd = trapframe["a0"] as isize;
            let filename = VirtualAddr(trapframe["a1"]);
            let flags = trapframe["a2"];
            let mode = trapframe["a3"];
            log!("syscall":"openat" > "pid({}) ({}, 0x{:x})", pcblock.pid, fd, filename.0);
            pcblock.trapframe()["a0"] =
                sys_openat(&mut pcblock, fd, filename, flags, mode) as usize;
        }
        SYSCALL_CLOSE => {
            let fd = trapframe["a0"] as isize;
            drop(trapframe);
            log!("syscall":"close" > "pid({}) ({})", pcblock.pid, fd);
            pcblock.trapframe()["a0"] = sys_close(&mut pcblock, fd) as usize;
        }
        SYSCALL_GETDENTS64 => {
            let fd = trapframe["a0"] as isize;
            let buf = VirtualAddr(trapframe["a1"]);
            let len = trapframe["a2"];
            drop(trapframe);
            log!("syscall":"getdents64" > "pid({}) ({}, 0x{:x}, {})", pcblock.pid, fd, buf.0, len);
            pcblock.trapframe()["a0"] = sys_getdents64(&mut pcblock, fd, buf, len) as usize;
        }
        SYSCALL_LSEEK => {
            let fd = trapframe["a0"] as isize;
            let offset = trapframe["a1"] as isize;
            let whence = trapframe["a2"];
            drop(trapframe);
            log!("syscall":"lseek" > "pid({}) ({}, {}, {})", pcblock.pid, fd, offset, whence);
            pcblock.trapframe()["a0"] = sys_lseek(&mut pcblock, fd, offset, whence) as usize;
        }
        SYSCALL_WRITE => {
            let fd = trapframe["a0"] as isize;
            let buf = VirtualAddr(trapframe["a1"]);
            let len = trapframe["a2"];
            drop(trapframe);
            log!("syscall":"write" > "pid({}) ({}, 0x{:x}, {})", pcblock.pid, fd, buf.0, len);
            pcblock.trapframe()["a0"] = sys_write(&mut pcblock, fd, buf, len) as usize;
        }
        SYSCALL_READ => {
            let fd = trapframe["a0"] as isize;
            let buf = VirtualAddr(trapframe["a1"]);
            let len = trapframe["a2"];
            drop(trapframe);
            log!("syscall":"read" > "pid({}) ({}, 0x{:x}, {})", pcblock.pid, fd, buf.0, len);
            pcblock.trapframe()["a0"] = sys_read(&mut pcblock, fd, buf, len) as usize;
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
        }
        SYSCALL_GETPID => {
            log!("syscall": "getpid"> "pid({})", pcblock.pid);
            pcblock.trapframe()["a0"] = sys_getpid(&pcblock) as usize;
        }
        SYSCALL_GETPPID => {
            log!("syscall": "getppid"> "pid({})", pcblock.pid);
            pcblock.trapframe()["a0"] = sys_getppid(&pcblock);
        }
        SYSCALL_WAIT4 => {
            let waitpid = trapframe["a0"] as isize;
            let wstatus = VirtualAddr(trapframe["a1"]);
            let options = trapframe["a2"];
            let rusage = VirtualAddr(trapframe["a3"]);
            drop(trapframe);
            log!("syscall":"wait4" > "pid({}) ({}, 0x{:x}, 0x{:x})", pcblock.pid, waitpid, wstatus.0, options);
            sys_wait4(&mut pcblock, waitpid, wstatus, options, rusage);
        }
        SYSCALL_SBRK => {
            let inc = trapframe["a0"];
            drop(trapframe);
            log!("syscall":"sbrk" > "pid({}) (0x{:x})", pcblock.pid, inc);
            pcblock.trapframe()["a0"] = sys_sbrk(&mut pcblock, inc);
        }
        SYSCALL_BRK => {
            let va = VirtualAddr(trapframe["a0"]);
            drop(trapframe);
            log!("syscall":"brk" > "pid({}) (0x{:x})", pcblock.pid, va.0);
            pcblock.trapframe()["a0"] = sys_brk(&mut pcblock, va) as usize;
        }
        SYSCALL_CLONE => {
            let flags = CloneFlags::from_bits(trapframe["a0"]).unwrap_or(CloneFlags::empty());
            let stack_top = VirtualAddr(trapframe["a1"]);
            let ptid = trapframe["a2"];
            let ctid = trapframe["a3"];
            let newtls = trapframe["a4"];
            drop(trapframe);
            log!("syscall":"clone" > "pid({}) flags({:?}), stack(0x{:x})", pcblock.pid, flags, stack_top.0);
            pcblock.trapframe()["a0"] =
                sys_clone(&mut pcblock, flags, stack_top, ptid, ctid, newtls) as usize;
        }
        SYSCALL_EXEC => {
            let path = VirtualAddr(trapframe["a0"]);
            let argv = VirtualAddr(trapframe["a1"]);
            let envp = VirtualAddr(trapframe["a2"]);
            sys_execve(&mut pcblock, path, argv, envp);
        }
        SYSCALL_KILL => {
            let pid = trapframe["a0"];
            let sig = trapframe["a1"];
            drop(trapframe);
            log!("syscall":"times">"pid({})", pcblock.pid);
            pcblock.trapframe()["a0"] = sys_kill(pid, sig) as usize;
        }
        SYSCALL_SIGACTION => {
            let signum = trapframe["a0"];
            let act = VirtualAddr(trapframe["a1"]);
            let oldact = VirtualAddr(trapframe["a2"]);
            drop(trapframe);
            log!("syscall":"sigaction">"pid({}) signal({:?})", pcblock.pid, crate::process::signal::Signal::from_bits(signum));
            pcblock.trapframe()["a0"] =
                sys_rt_sigaction(&mut pcblock, signum, act, oldact) as usize;
        }
        SYSCALL_SIGRETURN => {
            drop(trapframe);
            assert!(if let PcbState::SigHandling(_, _) = pcblock.state() {
                true
            } else {
                false
            });
            pcblock.signal_return();
            pcblock.set_state(PcbState::Running);
        }
        SYSCALL_TIMES => {
            let tms = trapframe["a0"];
            drop(trapframe);
            pcblock.trapframe()["a0"] = sys_times(&mut pcblock, VirtualAddr(tms));
            log!("syscall":"times">"pid({})", pcblock.pid);
        }
        SYSCALL_GET_TIME_OF_DAY => {
            let timespec = VirtualAddr(trapframe["a0"]);
            let timezone = VirtualAddr(trapframe["a1"]);
            pcblock.trapframe()["a0"] = sys_gettimeofday(timespec, timezone) as usize;
        }
        SYSCALL_NANOSLEEP => {
            let timespec = PhysAddr(trapframe["a0"]);
            let timespec: &TimeSpec = timespec.as_ref();
            let current_time = get_time();
            trapframe["a0"] = 0;
            let wakeup_time =
                get_time() + timespec.tv_sec * RTCLK_FREQ + timespec.tv_nsec * RTCLK_FREQ / 1000;
            pcblock.block_fn = Some(Arc::new(move |pcb| {
                if wakeup_time <= get_time() {
                    return true;
                }
                false
            }));
            pcblock.set_state(PcbState::Blocking);
        }
        SYSCALL_FORK => {
            drop(trapframe);
            log!("syscall":"fork" > "pid({}) ()", pcblock.pid);
            pcblock.trapframe()["a0"] = sys_fork(&mut pcblock) as usize;
        }
        _ => {
            log!("syscall":>"unsupported syscall {}", trapframe["a7"]);
        }
    }
    let state = pcblock.state;
    drop(pcblock);
    if let PcbState::Zombie(_) = state {
    } else {
        scheduler_ready_pcb(pcb.clone());
    }
    // Note: 这里必须显式调用drop释放进程锁
    drop(pcb);
    schedule();
}
