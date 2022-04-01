use crate::mm::address::*;
use crate::mm::kalloc::KALLOCATOR;
use crate::process::signal::*;
use crate::process::*;
use alloc::sync::Arc;
use core::cell::RefCell;
use spin::mutex::MutexGuard;

#[repr(C)]
pub(super) struct rt_sigaction {
    pub sa_handler: usize,
    pub sa_flags: usize,
    pub sa_mask: usize,
}

/**
 * struct sigaction {
 *      uninon {
 *          void (*sa_handler)(int),
 *          void (*sa_sigaction)(int, siginto_t*, void*)
 *      };
 *      int sa_flag;
 *      sigset_t sa_mask;
 *      void (*ra_restorer)(void);
 * };
 */

pub(super) fn sys_rt_sigaction(
    pcb: &mut MutexGuard<Pcb>,
    signum: usize,
    act: VirtualAddr,
    oldact: VirtualAddr,
) -> isize {
    let mut sa = PhysAddr::from(act);
    let sa: &mut rt_sigaction = sa.as_mut();
    if let Some(signal) = Signal::from_bits(signum) {
        let tf = KALLOCATOR.lock().kalloc();
        let stack = KALLOCATOR.lock().kalloc();
        pcb.sigaction_bind(
            signal,
            SigAction::Custom(Arc::new(RefCell::new(CustomSigAction {
                sa_handler: sa.sa_handler,
                sa_flags: SaFlags::from_bits(sa.sa_flags).unwrap(),
                sa_mask: Signal::from_bits(sa.sa_mask).unwrap(),
                trapframe: tf,
                user_stack: stack,
            }))),
        );
        0
    } else {
        -1
    }
}

pub(super) fn sys_kill(pid: usize, sig: usize) -> isize {
    let signal = Signal::from_bits(sig).unwrap();
    log!("syscall":"kill">"-> (pid({}), sig({:?}))", pid, signal);
    sigqueue_send(pid, signal);
    0
}
