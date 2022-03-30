use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::RwLock;
use super::Pid;

lazy_static!{
pub static ref SIGQUEUE: RwLock<BTreeMap<Pid, (Signal, Signal)>> = RwLock::new(BTreeMap::new());
}

bitflags!{
    pub struct Signal: usize{
        const	SIGHUP		= 1 << ( 1-1);  
        const	SIGINT		= 1 << ( 2-1);  
        const	SIGQUIT		= 1 << ( 3-1);  
        const	SIGILL		= 1 << ( 4-1);  
        const	SIGTRAP		= 1 << ( 5-1);	
        const	SIGABRT		= 1 << ( 6-1);	
        const	SIGIOT		= 1 << ( 6-1);  
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

pub fn sigqueue_send(pid: Pid, signal: Signal) {
    // 目前signal只能为单个信号
    if let Some((pending, mask)) = SIGQUEUE.write().get_mut(&pid) {
        if signal & *mask == Signal::empty() {
            *pending |= signal;
        }
    } else {
        // 不存在，表明进程已经退出
    }
}
pub fn sigqueue_clear(pid: Pid) {
    // 清除进程的sigqueue
    SIGQUEUE.write().remove(&pid);
}

pub fn sigqueue_init(pid: Pid) {
    if let Some(_) = SIGQUEUE.write().insert(pid, (Signal::empty(), Signal::empty())) {
        panic!("dumplicated sigqueue for pid {}", pid)
    }
}

#[derive(Clone)]
pub struct SigAction {
    pub sa_handler:usize,
    // pub sa_sigaction:usize,
    pub sa_mask:Vec<Signal>,
    pub sa_flags:SaFlags,
}

pub type SigActionBounds = Vec<(Signal, SigAction)>;