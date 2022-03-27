use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

use super::Pcb;
use crate::mm::address::PhysAddr;
use crate::mm::pgtbl::Pgtbl;
use crate::mm::activate_vm;
use crate::mm::memory_space::MemorySpace;
use crate::asm;

// 最多支持4核
static mut _HARTS :[Hart; 5] = [Hart::default(), Hart::default(),Hart::default(),Hart::default(),Hart::default(),];

pub struct Hart {
    pub hartid: usize,
    pub pcb: Option<Arc<Mutex<Pcb>>>,
    pub kernel_sp: usize,
    pub mem_space: Option<MemorySpace>
}

impl const Default for Hart {
    fn default() -> Self {
        Self {
            hartid: 0,
            pcb: None,
            kernel_sp: 0,
            mem_space: None
        }
    }
}

lazy_static! {
    static ref HARTS: Mutex<Vec<Hart>> = Mutex::new(Vec::new());
}

pub fn init_hart(pgtbl: &Pgtbl) {
    let sp: usize;
    unsafe { asm!("mv a0, sp", out("a0") sp) };
    let sp :PhysAddr = PhysAddr(sp).ceil().into();
    unsafe {
        _HARTS[hartid()].hartid = hartid();
        _HARTS[hartid()].kernel_sp = sp.0;
        _HARTS[hartid()].mem_space = Some(MemorySpace {
            pgtbl: pgtbl.copy(false),
            entry: 0,
            segments: Vec::new()
        });
        activate_vm(_HARTS[hartid()].mem_space.as_ref().unwrap().pgtbl.root.page());
    };
}

pub fn current_hart() -> &'static mut Hart {
    unsafe {
        &mut _HARTS[hartid()]
    }
}

pub fn current_hart_leak() {
    unsafe {
        _HARTS[hartid()].pcb = None;
    }
}

pub fn current_hart_run(pcb: Arc<Mutex<Pcb>>) {
    unsafe {
        pcb.lock().trapframe().kernel_sp = _HARTS[hartid()].kernel_sp;
        pcb.lock().trapframe().kernel_satp = _HARTS[hartid()].mem_space.as_ref().unwrap().pgtbl.get_satp();
        _HARTS[hartid()].pcb = Some(pcb);
    }
}

pub fn hartid() -> usize {
    let ret: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) ret);
    }
    ret
}
