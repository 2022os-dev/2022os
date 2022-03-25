use crate::mm::MemorySpace;
use crate::mm::kalloc::*;
use crate::config::*;
use super::TrapFrame;

pub type Pid = usize;

#[derive(Clone, Copy)]
pub enum PcbState {
    UnInit,
    Ready,
    Running,
    Exit,
}

pub struct Pcb {
    pub pid: Option<Pid>,
    pub state: PcbState,
    pub memory_space: MemorySpace,
}

impl Pcb {
    // Fixme: Remember to release kernel stack and trapframe when process dead
    pub fn new(memory_space: MemorySpace) -> Self {
        let mut pcb = Self {
            pid: None,
            state: PcbState::Ready,
            memory_space,
        };
        pcb.memory_space.map_trampoline();
        let trapframe = KALLOCATOR.lock().kalloc();
        let trapframe = <*const TrapFrame>::from_bits(trapframe.offset_phys(0).0);
        pcb.memory_space.map_trapframe(trapframe);
        pcb.trapframe().from_memory_space(memory_space);

        let stack = KALLOCATOR.lock().kalloc();
        // Assume that all process's stack in a page
        pcb.trapframe().kernel_sp = stack.offset(PAGE_SIZE).0;
        // Fixme: every process may has a independent page table
        pcb.trapframe().kernel_satp = riscv::register::satp::read().bits();
        // Map trapframe
        pcb
    }

    pub fn state(&self) -> PcbState {
        self.state
    }

    pub fn set_state(&mut self, state: PcbState) -> PcbState {
        let old_state = self.state;
        self.state = state;
        old_state
    }

    pub fn trapframe(&mut self) -> &mut TrapFrame {
        let pte = self.memory_space.pgtbl.walk(MemorySpace::trapframe_page().offset(0), false);
        unsafe {
            <*mut TrapFrame>::from_bits(pte.ppn().offset(0).0).as_mut().unwrap()
        }
    }

    pub fn exit(&mut self) {
        // Fixme: 记录进程的段地址，直接释放特定的段而不用搜索整个地址空间
        self.memory_space.pgtbl.unmap_pages(0.into()..0x8000.into(), true);
        self.memory_space.pgtbl.unmap(MemorySpace::trapframe_page(), true);
        self.memory_space.pgtbl.unmap(MemorySpace::trampoline_page(), true);
        self.state = PcbState::Exit;
    }
}

impl Drop for Pcb {
    fn drop(&mut self) {

    }
}