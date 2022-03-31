use crate::mm::MemorySpace;
use crate::process::cpu::*;
use crate::process::pcb::*;
use crate::process::{restore_trapframe, Pcb, PcbState};
use crate::process::signal::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use core::assert;

lazy_static! {
    // 保存所有可尝试调度的进程
    static ref READYTASKS: Mutex<Vec<Arc<Mutex<Pcb>>>> = Mutex::new(Vec::new());
}

pub fn scheduler_load_pcb(memory_space: MemorySpace) -> Arc<Mutex<Pcb>> {
    let pcb = Arc::new(Mutex::new(Pcb::new(memory_space, 0)));
    scheduler_ready_pcb(pcb.clone());
    pcb
}

/*
pub fn scheduler_block_pcb(pcb: Arc<Mutex<Pcb>>, reason: BlockReason) {
    log!("scheduler":"Block">"pid({})", pcb.lock().pid);
    pcb.lock().set_state(PcbState::Block(reason));
    BLOCKTASKS.lock().insert(0, pcb);
}
*/

pub fn scheduler_ready_pcb(pcb: Arc<Mutex<Pcb>>) {
    log!("scheduler":"Ready">"pid({})", pcb.lock().pid);
    READYTASKS.lock().insert(0, pcb);
}
/*
pub fn scheduler_signal(pid: Pid, reason: BlockReason) {
    log!("scheduler":"signal">"(pid({}), {:?})", pid, reason);
    let mut blocktasks = BLOCKTASKS.lock();
    let findret = blocktasks.iter().enumerate().find(|(_, pcb)| {
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
*/

pub fn schedule() -> ! {
    // FCFS
    log!("scheduler":>"Enter");
    loop {
        let mut tasklist = READYTASKS.lock();
        let pcb = tasklist.pop();
        drop(tasklist);

        if let Some(pcb) = pcb {
            // assert!(!pcb.is_locked());
            let state = pcb.lock().state();
            match state {
                PcbState::Ready => {
                    pcb.lock().try_handle_signal();
                    if let PcbState::Exit(_) = pcb.lock().state() {
                        // 进程退出了，重新调度
                        continue;
                    }
                }
                PcbState::Blocking(testfn) => {
                    if !testfn(pcb.clone()) {
                        scheduler_ready_pcb(pcb.clone());
                        continue;
                    }
                }
                _ => {
                    panic!("invalid state pcb in tasks {:?}", state);
                }
            }
            current_hart_run(pcb.clone());
            // current_hart_memset().pgtbl().print();
            let tf = current_pcb().unwrap().lock().memory_space.trapframe.offset(0);
            pcb.lock().set_state(PcbState::Running);
            drop(pcb);
            restore_trapframe(tf);
        } else {
            drop(pcb);
            current_hart_leak();
            log!("scheduler":>"No ready Pcb");
            // 查看是否已经释放所有pcb
            log!("pcb":"remain">"{}", unsafe {crate::process::pcb::DROPPCBS.lock()});
            loop {}
        }
    }
}

pub fn current_pcb() -> Option<Arc<Mutex<Pcb>>> {
    let cpu = current_hart();
    cpu.pcb.clone()
}
