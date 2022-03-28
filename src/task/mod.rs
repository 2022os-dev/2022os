use crate::mm::MemorySpace;
use crate::process::cpu::*;
use crate::process::pcb::*;
use crate::process::{restore_trapframe, Pcb, PcbState};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use core::assert;

lazy_static! {
    // 保存所有Ready进程
    static ref READYTASKS: Mutex<Vec<Arc<Mutex<Pcb>>>> = Mutex::new(Vec::new());
    // 保存所有Block进程
    static ref BLOCKTASKS: Mutex<Vec<Arc<Mutex<Pcb>>>> = Mutex::new(Vec::new());
}

pub fn scheduler_load_pcb(memory_space: MemorySpace) -> Arc<Mutex<Pcb>> {
    // Fixme: when ran out of pcbs
    let pcb = Arc::new(Mutex::new(Pcb::new(memory_space, 0, true, true)));
    scheduler_ready_pcb(pcb.clone());
    pcb
}

pub fn scheduler_block_pcb(pcb: Arc<Mutex<Pcb>>, reason: BlockReason) {
    pcb.lock().set_state(PcbState::Block(reason));
    BLOCKTASKS.lock().insert(0, pcb);
}

pub fn scheduler_ready_pcb(pcb: Arc<Mutex<Pcb>>) {
    pcb.lock().set_state(PcbState::Ready);
    READYTASKS.lock().insert(0, pcb);
}

pub fn scheduler_signal(pid: Pid, reason: BlockReason) {
    let mut blocktasks = BLOCKTASKS.lock();
    let findret = blocktasks.iter().enumerate().find(|(idx, pcb)| {
        let pcb = pcb.lock();
        if pcb.pid == pid {
            match pcb.state() {
                PcbState::Block(r) if r == reason => {
                    return true
                },
                _ => return false
            }
        }
        false
    });
    if let Some((idx, pcb)) = findret {
        let pcb = pcb.clone();
        blocktasks.remove(idx);
        drop(blocktasks);
        pcb_block_slot(pcb, reason);
    }
}

pub fn schedule() -> ! {
    // FCFS
    loop {
        let mut tasklist = READYTASKS.lock();
        let pcb = tasklist.pop();
        drop(tasklist);

        if let Some(pcb) = pcb {
            if pcb.is_locked() {
                panic!("pcb locked");
            }
            pcb.lock().set_state(PcbState::Running);
            let satp = pcb.lock().trapframe()["satp"];
            log!(debug "hart {} schedule {}", hartid(), pcb.lock().pid);
            current_hart_run(pcb.clone());
            drop(pcb);
            restore_trapframe(satp);
        } else {
            drop(pcb);
            current_hart_leak();
            // log!(debug "hart {} No ready pcb", hartid());
        }
    }
}

pub fn current_pcb() -> Option<Arc<Mutex<Pcb>>> {
    let cpu = current_hart();
    cpu.pcb.clone()
}
