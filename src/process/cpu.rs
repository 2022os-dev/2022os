use alloc::vec::Vec;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use riscv::register::mhartid;
use spin::Mutex;

use crate::asm;
use super::Pcb;

#[derive(Clone)]
pub struct Cpu {
    pub hartid: usize,
    pub pcb: Option<Arc<Mutex<Pcb>>>,
}

lazy_static! {
    static ref HARTS: Mutex<Vec<Cpu>> = Mutex::new(Vec::new());
}

pub fn init_hart() {
    HARTS.lock().push(Cpu {
        hartid: hartid(),
        pcb: None,
    });
}

pub fn current_hart() -> Cpu {
    for i in HARTS.lock().iter() {
        if i.hartid == hartid() {
            return i.clone();
        }
    }
    panic!("uninit hartid {}", mhartid::read());
}

pub fn current_hart_run(pcb: Arc<Mutex<Pcb>>) {
    for i in HARTS.lock().iter_mut() {
        if i.hartid == hartid() {
            i.pcb = Some(pcb.clone());
            break;
        }
    }
}

pub fn hartid() -> usize {
    let ret: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) ret);
    }
    ret
}
