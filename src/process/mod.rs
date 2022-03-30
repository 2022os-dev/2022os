pub mod cpu;
pub mod pcb;
pub mod signal;
mod trapframe;

pub use pcb::{Pcb, PcbState, Pid};
pub use trapframe::TrapFrame;
use crate::mm::*;
use core::mem::transmute;

pub fn restore_trapframe(tf: VirtualAddr) -> ! {
    let (_, _restore) = MemorySpace::trampoline_entry();
    let restore = unsafe { transmute::<usize, fn(usize) -> !>(_restore) };
    log!(debug "restore trapframe 0x{:x}", tf.0);
    restore(tf.0);
}
