pub mod cpu;
pub mod pcb;
mod trapframe;

pub use pcb::{Pcb, PcbState, Pid};
pub use trapframe::TrapFrame;
use crate::mm::*;

pub fn restore_trapframe(satp: usize) -> ! {
    let (_, _restore) = crate::mm::memory_space::MemorySpace::trampoline_entry();
    let restore = unsafe { core::mem::transmute::<usize, fn(usize, usize) -> !>(_restore) };
    let tf = MemorySpace::trapframe_page().offset(0).0;
    // ################# Test ########################
    let mut pgtbl = crate::mm::pgtbl::Pgtbl {
        root: (satp ^ 0x8000000000000000).into(),
    };
    let pte = pgtbl.walk(
        VirtualAddr(_restore),
        false,
    );
    if !pte.is_valid() {
        println!("restore_trapframe: unmap trampoline");
    }
    let pte = pgtbl.walk(
        crate::mm::memory_space::MemorySpace::trapframe_page().offset(0),
        false,
    );
    if !pte.is_valid() {
        println!("restore_trapframe: unmap trapframe");
    }
    // ###############################################
    restore(tf, satp);
}
