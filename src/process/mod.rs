pub mod cpu;
mod trapframe;
pub mod pcb;

use crate::mm::memory_space::MemorySpace;

pub use trapframe::TrapFrame;
pub use pcb::{Pcb, PcbState, Pid};

#[no_mangle]
pub fn restore_trapframe(satp: usize) -> ! {
    let (_, _restore) = crate::mm::memory_space::MemorySpace::trampoline_entry();
    let restore = unsafe { core::mem::transmute::<usize, fn(usize, usize) -> !>(_restore) };
    let tf = MemorySpace::trapframe_page().offset(0).0;
    // ################# Test ########################
    let mut pgtbl = crate::mm::pgtbl::Pgtbl {
        root: (satp ^ 0x8000000000000000).into()
    };
    let pte = pgtbl.walk(crate::mm::memory_space::MemorySpace::trampoline_page().offset(0), false);
    if !pte.is_valid() {
        println!("unmap trampoline");
    }
    let pte = pgtbl.walk(crate::mm::memory_space::MemorySpace::trapframe_page().offset(0), false);
    if !pte.is_valid() {
        println!("unmap trapframe");
    }
    // ###############################################
    restore(tf, satp);
}
