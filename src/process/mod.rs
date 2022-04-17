pub mod cpu;
pub mod pcb;
pub mod signal;
mod trapframe;

use crate::mm::*;
use core::mem::transmute;
pub use pcb::{Pcb, PcbState, Pid};
pub use trapframe::TrapFrame;

pub fn restore_trapframe(tf: VirtualAddr) -> ! {
    let (_, _restore) = MemorySpace::trampoline_entry();
    let restore = unsafe { transmute::<usize, fn(usize) -> !>(_restore) };
    restore(tf.0);
}
