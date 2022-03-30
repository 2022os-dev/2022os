use super::TrapFrame;
use crate::mm::MemorySpace;
use crate::task::scheduler_ready_pcb;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PcbState {
    Ready,
    Running,
    Exit(isize),
    Blocking(fn(Arc<Mutex<Pcb>>) -> bool)
}

pub struct Pcb {
    pub parent: Pid,
    pub pid: Pid,
    pub state: PcbState,
    pub memory_space: MemorySpace,
    pub children: Vec<Arc<Mutex<Pcb>>>,
    pub sabounds: SigActionBounds
}

impl Pcb {
    pub fn new(memory_space: MemorySpace, parent: Pid) -> Self {
        let pcb = Self {
            parent,
            pid: alloc_pid(),
            state: PcbState::Ready,
            memory_space,
            children: Vec::new(),
            sabounds: SigActionBounds::new()
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

    pub fn trapframe(&mut self) -> &mut TrapFrame {
        self.memory_space.trapframe()
    }

    pub fn exit(&mut self, xcode: isize) {
        self.state = PcbState::Exit(xcode);
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