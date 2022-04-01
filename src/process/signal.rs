use core::cell::RefCell;

use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use spin::RwLock;
use super::Pid;
use crate::mm::PageNum;
use crate::mm::kalloc::KALLOCATOR;

// Feature: 使用AtomicUsize原子类型
lazy_static!{
static ref SIGQUEUE: RwLock<BTreeMap<Pid, (Signal, Signal)>> = RwLock::new(BTreeMap::new());
}
pub const SIGTMIN: usize = 32;
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

pub fn sigqueue_send(pid: Pid, signal: Signal) {
    // 目前signal只能为单个信号
    if let Some((pending, mask)) = SIGQUEUE.write().get_mut(&pid) {
        if signal & *mask == Signal::empty() {
            *pending |= signal;
            log!("signal":"send">"successed (pid({}), signal({:?}))", pid, signal);
        } else {
            log!("signal":"send">"masked (pid({}), signal({:?}))", pid, signal);
        }
    } else {
        // 不存在，表明进程已经退出
    }
}
pub fn sigqueue_clear(pid: Pid) {
    // 清除进程的sigqueue
    log!("signal":"clear">"pid({})", pid);
    SIGQUEUE.write().remove(&pid);
}

pub fn sigqueue_init(pid: Pid) {
    log!("signal":"init">"pid({})", pid);
    if let Some(_) = SIGQUEUE.write().insert(pid, (Signal::empty(), Signal::empty())) {
        panic!("dumplicated sigqueue for pid {}", pid)
    }
}

pub fn sigqueue_pending(pid: Pid) -> bool {
    if let Some((pending, _)) = SIGQUEUE.read().get(&pid) {
        return !pending.is_empty()
    }
    false
}

pub fn sigqueue_mask(pid: Pid, mask: Signal) -> Signal {
    let mut sigqueue = SIGQUEUE.write();
    let(_, oldmask) = sigqueue.get_mut(&pid).unwrap();
    let old = oldmask.clone();
    *oldmask = mask;
    old
}

// 返回一个信号并且将sigqueue里相应的pending清空
pub fn sigqueue_fetch(pid: Pid) -> Option<Signal> {
    let mut sigqueue = SIGQUEUE.write();
    if let Some((pending, _)) = sigqueue.get_mut(&pid) {
        for i in 0..SIGTMIN {
            let testsig= Signal::from_bits(1 << i).unwrap_or(Signal::empty());
            if *pending & testsig != Signal::empty() {
                *pending &= !testsig;
                return Some(testsig)
            }
        }
        None
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub enum SigAction {
    Term,
    Ign,
    Core,
    Stop,
    Cont,
    Custom(Arc<RefCell<CustomSigAction>>)
}
unsafe impl Send for SigAction {}
unsafe impl Sync for SigAction {}

#[derive(Clone, Debug)]
pub struct CustomSigAction {
    pub sa_handler:usize,
    // pub sa_sigaction:usize,
    pub sa_mask:Signal,
    pub sa_flags:SaFlags,
    pub trapframe: PageNum,
    pub user_stack: PageNum
}

impl Drop for CustomSigAction {
    fn drop(&mut self) {
        KALLOCATOR.lock().kfree(self.trapframe);
        KALLOCATOR.lock().kfree(self.user_stack);
    }
}

pub type SigActionBinds = Vec<(Signal, SigAction)>;

pub fn sigactionbinds_default(signal: Signal) -> SigAction {
    match signal {
        Signal::SIGHUP => {
            SigAction::Term
        }
        Signal::SIGINT => {
            SigAction::Term
        }
        Signal::SIGQUIT => {
            SigAction::Core
        }
        Signal::SIGILL => {
            SigAction::Core
        }
        Signal::SIGABRT => {
            SigAction::Core
        }
        Signal::SIGFPE => {
            SigAction::Core
        }
        Signal::SIGKILL => {
            SigAction::Term
        }
        Signal::SIGSEGV => {
            SigAction::Core
        }
        Signal::SIGPIPE => {
            SigAction::Term
        }
        Signal::SIGALRM => {
            SigAction::Term
        }
        Signal::SIGTERM => {
            SigAction::Term
        }
        Signal::SIGUSR1 => {
            SigAction::Term
        }
        Signal::SIGUSR2 => {
            SigAction::Term
        }
        Signal::SIGCHLD => {
            SigAction::Ign
        }
        Signal::SIGCONT => {
            SigAction::Cont
        }
        Signal::SIGSTOP => {
            SigAction::Stop
        }
        Signal::SIGTSTP => {
            SigAction::Stop
        }
        Signal::SIGTTIN => {
            SigAction::Stop
        }
        Signal::SIGTTOU => {
            SigAction::Stop
        }
        Signal::SIGBUS => {
            SigAction::Core
        }
        Signal::SIGPROF => {
            SigAction::Term
        }
        Signal::SIGTRAP => {
            SigAction::Core
        }
        Signal::SIGURG => {
            SigAction::Ign
        }
        Signal::SIGVTALRM => {
            SigAction::Term
        }
        Signal::SIGXCPU => {
            SigAction::Core
        }
        Signal::SIGXFSZ => {
            SigAction::Core
        }
        Signal::SIGSTKFLT => {
            SigAction::Term
        }
        Signal::SIGIO => {
            SigAction::Term
        }
        Signal::SIGWINCH => {
            SigAction::Ign
        }
        _ => {
            panic!("Error")
        }
    }
}