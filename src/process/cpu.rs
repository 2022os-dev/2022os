use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use riscv::register::mhartid;
use spin::Mutex;

use super::Pcb;
use crate::mm::address::PhysAddr;
use crate::mm::{PageNum, KALLOCATOR};
use crate::config::PAGE_SIZE;
use crate::asm;

#[derive(Clone)]
pub struct Cpu {
    pub hartid: usize,
    pub pcb: Option<Arc<Mutex<Pcb>>>,
    pub kernel_sp: usize,
}

lazy_static! {
    static ref HARTS: Mutex<Vec<Cpu>> = Mutex::new(Vec::new());
}

pub fn init_hart() {
    let sp: usize;
    unsafe { asm!("mv a0, sp", out("a0") sp) };
    HARTS.lock().push(Cpu {
        hartid: hartid(),
        pcb: None,
        kernel_sp: PhysAddr(sp).ceil().0
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

pub fn current_hart_leak() {
    for i in HARTS.lock().iter_mut() {
        if i.hartid == hartid() {
            i.pcb = None;
        }
    }

}

pub fn current_hart_run(pcb: Arc<Mutex<Pcb>>) {
    for i in HARTS.lock().iter_mut() {
        if i.hartid == hartid() {
            pcb.lock().trapframe().kernel_sp = i.kernel_sp;
            i.pcb = Some(pcb.clone());
            break;
        }
    }
}

pub fn hartid() -> usize {
    let ret: usize;
    unsafe {
        asm!("mv a0, tp", out("a0") ret);
    }
    ret
}
