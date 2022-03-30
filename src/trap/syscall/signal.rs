use crate::mm::address::*;

#[repr(C)]
pub(super) struct rt_sigaction {
    pub sa_handler: usize,
    pub sa_flags: usize,
    pub sa_mask: usize
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

bitflags! {
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

pub(super) fn rt_sigaction(signum: usize, act: VirtualAddr, oldact: VirtualAddr) -> isize {
    let sa: &mut rt_sigaction = PhysAddr::from(act).as_mut();
    0
}