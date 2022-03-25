mod file;
mod process;
use crate::mm::address::*;
use crate::process::*;
use crate::task::*;
use file::*;
use process::*;

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

pub fn syscall_handler() -> ! {
    let pcb = current_pcb().unwrap();
    let mut pcb = pcb.lock();
    let trapframe = pcb.trapframe();

    // 指向下一条指令
    trapframe["sepc"] += 4;

    match trapframe["a7"] {
        SYSCALL_WRITE => {
            let fd = trapframe["a0"];
            let buf = VirtualAddr(trapframe["a1"]);
            let len = trapframe["a2"];
            pcb.trapframe()["a0"] = sys_write(&mut pcb, fd, buf, len) as usize;
        }
        SYSCALL_EXIT => {
            let xcode = trapframe["a0"];
            drop(trapframe);
            sys_exit(&mut pcb, xcode);
        }
        SYSCALL_YIELD => {
            trapframe["a0"] = sys_yield() as usize;
        }
        SYSCALL_GETPID => {
            pcb.trapframe()["a0"] = sys_getpid(&pcb) as usize;
        }
        SYSCALL_FORK => {
            pcb.trapframe()["a0"] = sys_fork(&mut pcb) as usize;
        }
        _ => {
            println!("unsupported syscall {}", trapframe["a7"]);
        }
    }
    if let PcbState::Running = pcb.state() {
        drop(pcb);
        scheduler_ready_pcb(current_pcb().unwrap());
    }
    schedule();
}
