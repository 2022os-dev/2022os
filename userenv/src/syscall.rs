#![allow(dead_code)]
use core::arch::asm;
use crate::println;
use core::mem::size_of;
use core::slice::from_raw_parts_mut;

const SYSCALL_GETCWD: usize = 17;
const SYSCALL_DUP: usize = 23;
const SYSCALL_DUP3:usize = 24;
const SYSCALL_FCNTL:usize = 25;
const SYSCALL_IOCTL:usize = 29;
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
const SYSCALL_FSTAT:usize = 80;
const SYSCALL_FSYNC:usize = 82;
const SYSCALL_UTIMENSAT:usize = 88;
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

pub const AT_FDCWD: isize = -100;
bitflags! {
    // 表示openat(2) 中的flags
    pub struct OpenFlags: usize {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 6;
        const TRUNC = 1 << 10;
        const DIRECTROY = 0200000;
        const LARGEFILE  = 0100000;
        const CLOEXEC = 02000000;
    }
    // 表示openat(2) 中的mode_t
    pub struct FileMode: usize {
    }
}
impl OpenFlags {
    pub fn readable(&self) -> bool {
        *self & OpenFlags::RDWR != OpenFlags::empty() || 
            *self & OpenFlags::RDONLY != OpenFlags::empty()
    }
    pub fn writable(&self) -> bool {
        *self & OpenFlags::RDWR != OpenFlags::empty() || 
            *self & OpenFlags::WRONLY != OpenFlags::empty()
    }

}

pub fn ls(path: &str) {
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

pub fn syscall_getcwd(buf: &mut [u8]) -> isize {
    let mut a0 = buf.as_ptr() as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") buf.len(),
            in("x17") SYSCALL_GETCWD
        )
    }
    a0 as isize
}

const PATH_LIMITS: usize = 512;
#[repr(C)]
pub struct LinuxDirent {
    pub d_ino: usize,
    pub d_off: isize,
    pub d_reclen: u16,
    pub d_type: u8,                // linux manual中d_type应该在d_name后面?
    pub d_name: [u8; PATH_LIMITS]  // 使用固定的name长度
}

pub const DT_UNKNOWN:u8 = 0;
pub const DT_DIR:u8 = 4;
pub const DT_REG:u8 = 4; //常规文件

impl LinuxDirent {
    pub fn new() -> Self {
        Self {
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
            d_name: [0; PATH_LIMITS]
        }
    }

    pub fn fill(&mut self, other: &Self) {
        self.d_ino = other.d_ino;
        self.d_off= other.d_off;
        self.d_reclen= other.d_reclen;
        self.d_type = other.d_type;
        self.d_name.copy_from_slice(&other.d_name);
    }
}

pub fn syscall_getdirents64(fd: isize, buf: &mut [u8], len: usize) -> isize {
    let mut a0 = fd as isize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") buf.as_ptr() as usize,
            in("x12") len,
            in("x17") SYSCALL_GETDENTS64
        )
    }
    a0 as isize
}

pub fn syscall_chdir(buf: &str) -> isize {
    let mut a0 = buf.as_ptr() as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x17") SYSCALL_CHDIR
        )
    }
    a0 as isize
}


pub fn syscall_pipe(pipe: &mut [isize; 2]) -> isize {
    let mut a0 = pipe as *const _ as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x17") SYSCALL_PIPE
        )
    }
    a0 as isize

}

pub fn syscall_openat(fd: isize, filename: &str, flags: OpenFlags, mode: FileMode) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") filename.as_ptr() as usize,
            in("x12") flags.bits() as usize,
            in("x13") mode.bits() as usize,
            in("x17") SYSCALL_OPENAT
        )
    }
    a0 as isize
}

pub fn syscall_dup(fd: isize) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x17") SYSCALL_DUP
        )
    }
    a0 as isize
}

pub fn syscall_dup3(oldfd: isize, newfd: isize) -> isize {
    let mut a0 = oldfd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") newfd,
            in("x17") SYSCALL_DUP3
        )
    }
    a0 as isize
}

pub fn syscall_mkdirat(fd: isize, path: &str, mode: FileMode) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") path.as_ptr() as usize,
            in("x12") mode.bits(),
            in("x17") SYSCALL_MKDIRAT
        )
    }
    a0 as isize
}

pub fn syscall_close(fd: isize) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x17") SYSCALL_CLOSE
        )
    }
    a0 as isize

}

pub const SEEK_SET: usize = 0;
pub const SEEK_CUR : usize = 1;
pub const SEEK_END : usize = 2;
pub fn syscall_lseek(fd: isize, offset: isize, whence: usize) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") offset as usize,
            in("x12") whence,
            in("x17") SYSCALL_LSEEK
        )
    }
    a0 as isize
}

pub fn syscall_write(fd: isize, buf: &[u8]) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") buf.as_ptr() as usize,
            in("x12") buf.len(),
            in("x17") SYSCALL_WRITE
        )
    }
    a0 as isize
}
pub fn syscall_read(fd: isize, buf: &mut [u8]) -> isize {
    let mut a0 = fd as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") buf.as_ptr() as usize,
            in("x12") buf.len(),
            in("x17") SYSCALL_READ
        )
    }
    a0 as isize
}

pub fn syscall_exit(xcode: isize) -> !{
    unsafe {
        asm!("ecall", in("x10") xcode,
            in("x17") SYSCALL_EXIT
        );
    }
    loop {}
}

pub fn syscall_yield() {
    unsafe {
        asm!("ecall",in("x17") SYSCALL_YIELD);
    }
}

pub fn syscall_fork() -> isize {
    let mut ret = 0;
    unsafe {
        asm!("ecall", out("x10") ret, in("x17") SYSCALL_FORK);
    }
    ret
}

pub fn syscall_mount() -> isize {
    let mut ret = 0;
    unsafe {
        asm!("ecall", out("x10") ret, in("x17") SYSCALL_MOUNT);
    }
    ret
}

pub fn syscall_umount2() -> isize {
    let mut ret = 0;
    unsafe {
        asm!("ecall", out("x10") ret, in("x17") SYSCALL_UMOUNT2);
    }
    ret
}

bitflags! {
    pub struct CloneFlags: usize{
        const SIGCHLD = 17;
        const CLONE_CHILD_CLEARTID = 0x00200000;
        const CLONE_CHILD_SETTID = 0x01000000;
    }
}

pub fn syscall_clone(flags: CloneFlags, stack_top: *const u8, ptid: usize, ctid: usize, newtls: usize) -> isize {
    let mut a0 = flags.bits() as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
         in("x11") stack_top as usize,
         in("x12") ptid,
         in("x13") ctid,
         in("x14") newtls,
         in("x17") SYSCALL_CLONE);
    }
    a0 as isize

}

pub fn syscall_execve(path: &str, argv: &[usize], envp: &[usize]) {
    let mut a0 = path.as_ptr() as usize; 
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") argv.as_ptr() as usize,
            in("x12") envp.as_ptr() as usize,
            in("x17") SYSCALL_EXEC
        )
    }
}


pub fn syscall_getpid() -> usize {
    let mut ret = 0;
    unsafe {
        asm!("ecall",inout("x10") ret, in("x17") SYSCALL_GETPID);
    }
    ret
}

pub fn syscall_wait4(pid: isize, wstatus: &mut isize, options: usize, rusage: &mut usize) -> isize {
    let mut pid = pid as usize;
    unsafe {
        asm!("ecall", inout("x10") pid, 
            in("x11") wstatus as *const _ as usize,
            in("x12") options, 
            in("x13") rusage as *const _ as usize, 
            in("x17") SYSCALL_WAIT4
        )
    }
    pid as isize
}

pub fn syscall_sbrk(mut inc: usize) -> *mut u8 {
    unsafe {
        asm!("ecall", inout("x10") inc,
            in("x17") SYSCALL_SBRK
        )
    }
    inc as *mut u8
}

pub fn syscall_brk(addr: *const u8) -> isize {
    let mut addr = addr as usize;
    unsafe {
        asm!("ecall", inout("x10") addr,
            in("x17") SYSCALL_BRK
        )
    }
    addr as isize
}

pub const SIGTMIN: usize = 32;
use bitflags::bitflags;
bitflags!{
    pub struct Signal: usize{
        const	SIGHUP		= 1 << ( 1-1);  
        const	SIGINT		= 1 << ( 2-1);  
        const	SIGQUIT		= 1 << ( 3-1);  
        const	SIGILL		= 1 << ( 4-1);  
        const	SIGTRAP		= 1 << ( 5-1);	
        const	SIGABRT		= 1 << ( 6-1);	
        // const	SIGIOT		= 1 << ( 6-1);  
        const	SIGBUS		= 1 << ( 7-1);  
        const	SIGFPE		= 1 << ( 8-1);  
        const	SIGKILL		= 1 << ( 9-1);  
        const	SIGUSR1		= 1 << (10-1);	
        const	SIGSEGV		= 1 << (11-1);	
        const	SIGUSR2		= 1 << (12-1);	
        const	SIGPIPE		= 1 << (13-1);	
        const	SIGALRM		= 1 << (14-1);	
        const	SIGTERM		= 1 << (15-1);	
        const	SIGSTKFLT	= 1 << (16-1);	
        const	SIGCHLD		= 1 << (17-1);	
        const	SIGCONT		= 1 << (18-1);	
        const	SIGSTOP		= 1 << (19-1);	
        const	SIGTSTP		= 1 << (20-1);	
        const	SIGTTIN		= 1 << (21-1);	
        const	SIGTTOU		= 1 << (22-1);	
        const	SIGURG		= 1 << (23-1);	
        const	SIGXCPU		= 1 << (24-1);	
        const	SIGXFSZ		= 1 << (25-1);	
        const	SIGVTALRM	= 1 << (26-1);	
        const	SIGPROF		= 1 << (27-1);	
        const	SIGWINCH	= 1 << (28-1);	
        const	SIGIO		= 1 << (29-1);	
    }
    pub struct SaFlags: usize{
        const SA_NOCLDSTOP = 1		   ;     /* Don't send SIGCHLD when children stop.  */
        const SA_NOCLDWAIT = 2		   ;     /* Don't create zombie on child death.  */
        const SA_SIGINFO   = 4		   ;     /* Invoke signal-catching function with three arguments instead of one.  */
        const SA_ONSTACK   = 0x08000000;    /* Use signal stack by using `sa_restorer'. */
        const SA_RESTART   = 0x10000000;    /* Restart syscall on signal return.  */
        const SA_NODEFER   = 0x40000000;    /* Don't automatically block the signal when its handler is being executed.  */
        const SA_RESETHAND = 0x80000000;    /* Reset to SIG_DFL on entry to handler.  */
        const SA_INTERRUPT = 0x20000000;    /* Historical no-op.  */
    }
}
pub fn syscall_kill(mut pid: isize, sig: Signal) -> isize {
    unsafe {
        asm!("ecall", inout("x10") pid,
            in("x11") sig.bits(),
            in("x17") SYSCALL_KILL
        )
    }
    pid as isize
}

#[repr(C)]
pub struct rt_sigaction {
    pub sa_handler: usize,
    pub sa_flags: usize,
    pub sa_mask: usize
}

pub fn syscall_sigaction(signal: Signal, act: &rt_sigaction, old: &rt_sigaction) -> isize {
    let mut a0 = signal.bits();
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x11") act as *const _ as usize,
            in("x12") old as *const _ as usize,
            in("x17") SYSCALL_SIGACTION
        )
    }
    a0 as isize
}
pub fn syscall_sigreturn() {
    unsafe {
        asm!("ecall", in("x17") SYSCALL_SIGRETURN
        )
    }
}
#[repr(C)]
#[derive(Default)]
pub struct Tms {
    pub utime: usize,
    pub stime: usize,
    pub cutime: usize,
    pub cstime: usize,
}

pub fn syscall_times(tms: &mut Tms) -> usize {
    let mut a0 = tms as *const _ as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x17") SYSCALL_TIMES
        )
    }
    a0
}

#[repr(C)]
pub struct TimeSpec {
    pub tv_sec: usize,
    pub tv_nsec: usize
}

pub fn syscall_nanosleep(sec: usize, nsec: usize) -> isize {
    let ts = TimeSpec {
        tv_nsec: nsec,
        tv_sec: sec
    };
    let mut a0 = &ts as *const _ as usize;
    unsafe {
        asm!("ecall", inout("x10") a0,
            in("x17") SYSCALL_NANOSLEEP
        )
    }
    a0 as isize
}