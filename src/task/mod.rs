use crate::mm::MemorySpace;
use crate::process::cpu::*;
use crate::process::*;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

lazy_static! {
    // 保存所有可尝试调度的进程
    static ref READYTASKS: Mutex<Vec<Arc<Mutex<Pcb>>>> = Mutex::new(Vec::new());
}

pub fn scheduler_load_pcb(memory_space: MemorySpace) {
    let pcb = Arc::new(Mutex::new(Pcb::new(memory_space, 1, String::from("/"))));
    scheduler_ready_pcb(pcb);
}

pub fn scheduler_ready_pcb(pcb: Arc<Mutex<Pcb>>) {
    log!("scheduler":"Ready">"pid({})", pcb.lock().pid);
    #[cfg(feature = "FCFS")]
    READYTASKS.lock().push(pcb);
    #[cfg(not(feature = "FCFS"))]
    READYTASKS.lock().insert(0, pcb);
}

#[cfg(feature = "FCFS")]
pub fn scheduler_block_pcb(pcb: Arc<Mutex<Pcb>>) {
    log!("scheduler":"Blocking">"pid({})", pcb.lock().pid);
    READYTASKS.lock().insert(0, pcb);
}

pub fn schedule() -> ! {
    // FCFS
    log!("scheduler":>"Enter");
    loop {
        let pcb = READYTASKS.lock().pop();

        if let Some(pcb) = pcb {
            // assert!(!pcb.is_locked());
            let state = pcb.lock().state();
            match state {
                PcbState::Running => match pcb.lock().try_handle_signal() {
                    PcbState::Zombie(_) => {
                        continue;
                    }
                    PcbState::Running => {}
                    PcbState::SigHandling(_, _) => {}
                    _ => {
                        panic!("Invalid state");
                    }
                },
                PcbState::Blocking => {
                    let mut pcblock = pcb.lock();
                    if pcblock.non_block() {
                        log!("scheduler":"unblock">"");
                        pcblock.block_fn = None;
                        pcblock.set_state(PcbState::Running);
                    } else {
                        log!("scheduler":"block">"still blocking");
                    }
                    drop(pcblock);
                    scheduler_ready_pcb(pcb);
                    continue;
                }
                PcbState::SigHandling(_, _) => {}
                _ => {
                    panic!("invalid state pcb in tasks {:?}", state);
                }
            }
            current_hart_run(pcb);
        } else {
            drop(pcb);
            current_hart_leak();
            #[cfg(feature = "batch")]
            {
                // 查看是否已经释放所有pcb
                log!("scheduler":>"No ready Pcb");
                log!("pcb":"remain">"{}", unsafe {crate::process::pcb::DROPPCBS.lock()});
                loop {}
            }
        }
    }
}

pub fn current_pcb() -> Option<Arc<Mutex<Pcb>>> {
    let cpu = current_hart();
    cpu.pcb.clone()
}
