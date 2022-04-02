pub mod syscall;
use core::arch::global_asm;

use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
};

use crate::mm::MemorySpace;
use crate::process::cpu::*;
use crate::task::*;

extern "C" {
    pub fn __alltraps();
    pub fn __restore(cx: usize, satp: usize);
    pub fn trampoline();
}

global_asm!(include_str!("traps.s"));

pub fn init() {
    unsafe {
        let (alltraps, _) = MemorySpace::trampoline_entry();
        stvec::write(alltraps, TrapMode::Direct);
    }
}

pub extern "C" fn trap_handler() {
    // Fixme: Don't skip the reference lifetime checker;
    current_pcb()
        .unwrap()
        .lock()
        .utimes_add(get_time() - current_hart_set_trap_times(get_time()));
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            syscall::syscall_handler();
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            panic!(
                "store fault sepc: 0x{:x}; stval 0x{:x}",
                current_pcb().unwrap().lock().trapframe()["sepc"],
                riscv::register::stval::read()
            );
            /*
            if let riscv::register::sstatus::SPP::Supervisor = cx.sstatus.spp() {
                panic!("PageFault in application, core dumped. sepc:0x{:x}", cx.sepc);
            }else {
                panic!("PageFault in application, core dumped. sepc:0x{:x}", cx.sepc);
            }
            */
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            panic!(
                "IllegalInstruction in application, core dumped. sepc:0x{:X}",
                current_pcb().unwrap().lock().trapframe()["sepc"]
            );
        }
        Trap::Exception(Exception::InstructionPageFault) => {
            use crate::mm::address::*;
            let mut i = PhysAddr(0xdc);
            let i: &mut usize = i.as_mut();
            println!("is {}", i);
            panic!(
                "InstructionPageFault, core dumped, sepc: 0x{:x}, scause:{:?}",
                current_pcb().unwrap().lock().trapframe()["sepc"],
                scause.cause()
            );
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            log!("trap":"time_interrupt">"");
            hart_set_next_trigger();
            scheduler_ready_pcb(current_hart().pcb.take().unwrap());
            schedule();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}:0x{:x}, stval = {:#x}!",
                scause.cause(),
                scause.bits(),
                stval
            );
        }
    };
}
