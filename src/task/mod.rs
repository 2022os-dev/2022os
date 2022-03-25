use spin::Mutex;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::mm::MemorySpace;
use crate::process::cpu::*;
use crate::process::{restore_trapframe, Pcb, PcbState};

lazy_static! {
    // 保存所有Ready进程
    pub static ref TASKLIST: Mutex<Vec<Arc<Mutex<Pcb>>>> = Mutex::new(Vec::new());
}

pub fn load_pcb(memory_space: MemorySpace) {
    // Fixme: when ran out of pcbs
    let pcb = Arc::new(Mutex::new(Pcb::new(memory_space)));
    TASKLIST.lock().push(pcb);
}

pub fn push_pcb(pcb: Arc<Mutex<Pcb>>) {
    TASKLIST.lock().push(pcb);
}

pub fn schedule_pcb() -> ! {
    // FCFS
    let mut tasklist = TASKLIST.lock();
    let pcb = tasklist.pop();

    if let Some(pcb) = pcb {
        pcb.lock().set_state(PcbState::Running);
        let satp = pcb.lock().trapframe()["satp"];        
        current_hart_run(pcb.clone());
        drop(tasklist);
        drop(pcb);
        restore_trapframe(satp);
    } else {
        log!(debug "No ready pcb");
    }
    loop {}
}

pub fn current_pcb() -> Option<Arc<Mutex<Pcb>>> {
    let cpu = current_hart();
    cpu.pcb
}