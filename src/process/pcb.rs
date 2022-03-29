use super::TrapFrame;
use crate::config::*;
use crate::mm::kalloc::*;
use crate::mm::MemorySpace;
use crate::mm::address::*;
use crate::task::scheduler_ready_pcb;
use crate::task::scheduler_signal;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockReason {
    Wait
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PcbState {
    Ready,
    Running,
    Block(BlockReason),
    Exit(isize),
}

pub struct Pcb {
    pub parent: Pid,
    pub pid: Pid,
    pub state: PcbState,
    pub memory_space: MemorySpace,
    pub children: Vec<Arc<Mutex<Pcb>>>,
}

impl Pcb {
    pub fn new(memory_space: MemorySpace, parent: Pid) -> Self {
        let pcb = Self {
            parent,
            pid: alloc_pid(),
            state: PcbState::Ready,
            memory_space,
            children: Vec::new(),
        };
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
        scheduler_signal(self.parent, BlockReason::Wait);
    }
}

impl Drop for Pcb {
    fn drop(&mut self) {
        log!("pcb":"drop">"pid({})", self.pid);
    }
}

pub fn pcb_block_slot(pcb: Arc<Mutex<Pcb>>, reason: BlockReason) {
    log!("pcb":"slot">"pid({}) - Reason({:?})", pcb.lock().pid, reason);
    match reason {
        BlockReason::Wait => {
            scheduler_ready_pcb(pcb);
        }
    }
}
