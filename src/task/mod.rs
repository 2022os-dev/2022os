use spin::Mutex;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::mm::MemorySpace;
use crate::process::cpu::*;
use crate::process::{restore_trapframe, Pcb, PcbState};

lazy_static! {
    pub static ref TASKLIST: Mutex<Vec<Arc<Mutex<Pcb>>>> = Mutex::new(Vec::new());
}

pub fn load_pcb(memory_space: MemorySpace) {
    // Fixme: when ran out of pcbs
    let pcb = Arc::new(Mutex::new(Pcb::new(memory_space)));
    TASKLIST.lock().push(pcb);
}

pub fn schedule_pcb() -> ! {
    // FCFS
    let mut pcb = None;
    for i in TASKLIST.lock().iter() {
        if let PcbState::Ready = i.lock().state() {
            pcb = Some(i.clone());
            current_hart_run(i.clone());
            break;
        }
    }

    if let Some(pcb) = pcb {
        pcb.lock().set_state(PcbState::Running);
        let satp = pcb.lock().trapframe()["satp"];        
        drop(pcb);
        restore_trapframe(satp);
    } else {
        log!(debug "No ready pcb");
    }
    loop {}
}

pub fn current_pcb() -> Arc<Mutex<Pcb>> {
    let cpu = current_hart();
    cpu.pcb.unwrap()
}