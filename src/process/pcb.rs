use super::TrapFrame;
use crate::mm::MemorySpace;
use crate::mm::PageNum;
use super::signal::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use spin::Mutex;

pub type Pid = usize;

lazy_static! {
    static ref PIDALLOCATOR: AtomicUsize = AtomicUsize::new(1);
}

pub fn alloc_pid() -> usize {
    PIDALLOCATOR.fetch_add(1, Ordering::Relaxed)
}
// Note: 使用Atomic类型会出错
#[cfg(feature = "pcb")]
pub static mut DROPPCBS: Mutex<usize> = Mutex::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PcbState {
    Ready,
    Running,
    Exit(isize),
    // 保持信号处理前的trapframe和signal mask
    SigHandling(PageNum, Signal),
    Blocking(fn(Arc<Mutex<Pcb>>) -> bool)
}

pub struct Pcb {
    pub parent: Pid,
    pub pid: Pid,
    pub state: PcbState,
    pub memory_space: MemorySpace,
    pub children: Vec<Arc<Mutex<Pcb>>>,
    pub sabinds: SigActionBinds,
}

impl Pcb {
    pub fn new(memory_space: MemorySpace, parent: Pid) -> Self {
        let pcb = Self {
            parent,
            pid: alloc_pid(),
            state: PcbState::Ready,
            memory_space,
            children: Vec::new(),
            sabinds: SigActionBinds::new()
        };
        #[cfg(feature = "pcb")]
        unsafe { *DROPPCBS.lock() += 1; }
        sigqueue_init(pcb.pid);
        pcb
    }

    pub fn state(&self) -> PcbState {
        self.state
    }

    pub fn set_state(&mut self, state: PcbState) -> PcbState {
        let old_state = self.state;
        self.state = state;
        old_state
    }

    // 将进程状态重新设置为可调度的状态 Ready、SigHandling
    pub fn reset_state(&mut self) -> PcbState {
        match self.state {
            PcbState::Running => {
                self.state = PcbState::Ready;
            }
            PcbState::Ready => {

            }
            PcbState::Blocking(_) | PcbState::Exit(_) => {
                panic!("can't reset block or exited pcb");
            }
            PcbState::SigHandling(_, _) => {

            }
        }
        self.state
    }

    pub fn trapframe(&mut self) -> &mut TrapFrame {
        self.memory_space.trapframe()
    }

    pub fn exit(&mut self, xcode: isize) {
        self.state = PcbState::Exit(xcode);
    }

    pub fn sigaction(&mut self, signal: Signal) -> SigAction {
        let mut act = self.sabinds.iter_mut().find(|(sig, _)| {
            if *sig == signal {
                true
            } else {
                false
            }
        });
        if let None = act {
            sigactionbinds_default(signal)
        } else {
            act.unwrap().clone().1
        }
    }

    pub fn sigaction_bind(&mut self, signal: Signal, act: SigAction) {
        log!("signal":"bind">"signal({:?}) -> handler({:?})", signal, act);
        self.sabinds.push((signal, act));
    }

    pub fn try_handle_signal(&mut self) -> PcbState {
        // 信号处理
        // 应该加上一层循环，等待所有信号处理完毕后再调度
        while let Some(signal) = sigqueue_fetch(self.pid) {
            log!("signal":"handle">"pid({}) try handle signal({:?})", self.pid, signal);
            let act = self.sigaction(signal);
            match act {
                SigAction::Cont => {
                    // 不做处理
                    continue;
                }
                SigAction::Term => {
                    self.exit(-1);
                    return PcbState::Exit(-1)
                }
                SigAction::Core => {
                    self.exit(-1);
                    return PcbState::Exit(-1)
                }
                SigAction::Stop => {
                    self.exit(-1);
                    return PcbState::Exit(-1)
                }
                SigAction::Ign => {
                    // 不做处理
                    continue;
                }
                SigAction::Custom(act) => {
                    // 使用原有的用户栈
                    // 暂时不支持处理信号时处理其他信号
                    // 不支持使用新的信号处理栈
                    log!("signal":"handle">"pid({}) custom action", self.pid);
                    let mask = sigqueue_mask(self.pid, act.borrow().sa_mask);
                    let oldsp = self.trapframe()["sp"];
                    let oldtf = self.sigaction_swap_trapframe(act.borrow().trapframe);
                    self.trapframe().init(oldsp, act.borrow().sa_handler);
                    self.trapframe()["a0"] = signal.bits();
                    self.set_state(PcbState::SigHandling(oldtf, mask));
                    return PcbState::SigHandling(oldtf, mask)
                }
            }
        }
        self.set_state(PcbState::Running);
        PcbState::Running
    }

    // 从信号处理的上下文中恢复
    pub fn signal_return(&mut self) {
        if let PcbState::SigHandling(tf, mask) = self.state() {
            log!("signal":"return">"pid({})", self.pid);
            self.sigaction_swap_trapframe(tf);
            sigqueue_mask(self.pid, mask);
        } else {
            panic!("Not handling signal");
        }
    }

    fn sigaction_swap_trapframe(&mut self, tf: PageNum) -> PageNum {
        let old = self.memory_space.trapframe;
        self.memory_space.trapframe = tf;
        old
    }
}

impl Drop for Pcb {
    fn drop(&mut self) {
        #[cfg(feature = "pcb")]
        unsafe { *DROPPCBS.lock() -= 1; }
        log!("pcb":"drop">"pid({})", self.pid);
        sigqueue_clear(self.pid);
    }
}

/*
pub fn pcb_block_slot(pcb: Arc<Mutex<Pcb>>, reason: BlockReason) {
    log!("pcb":"slot">"pid({}) - Reason({:?})", pcb.lock().pid, reason);
    match reason {
        BlockReason::Wait => {
            scheduler_ready_pcb(pcb);
        }
    }
}
*/