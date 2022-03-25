pub mod time;
use core::arch::global_asm;

use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

use crate::mm::MemorySpace;
use crate::process::PcbState;
use crate::task::*;

extern "C" {
    pub fn __alltraps();
}
extern "C" {
    pub fn __restore(cx: usize, satp: usize);
}
extern "C" {
    pub fn trampoline();
}

global_asm!(include_str!("traps.s"));
pub fn init() {
    unsafe {
        let (alltraps, _) = MemorySpace::trampoline_entry();
        stvec::write(alltraps, TrapMode::Direct);
    }
}

pub fn enable_timer_interupt() {
    unsafe {
        sie::set_stimer();
    }
    time::set_next_trigger();
}

#[no_mangle]
pub extern "C" fn trap_handler() -> ! {
    // Fixme: Don't skip the reference lifetime checker;
    let pcb = current_pcb();
    let mut pcblock = pcb.lock();
    let cx = pcblock.trapframe().clone();

    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            pcblock.trapframe()["sepc"] += 4;
            pcblock.trapframe()["a0"] = crate::syscall::syscall(&mut pcblock, cx["a7"], [cx["a0"], cx["a1"], cx["a2"]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            panic!(
                "store fault sepc: 0x{:x}; stval 0x{:x}",
                cx["sepc"],
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
                " IllegalInstruction in application, core dumped. sepc:0x{:X}",
                cx.sepc
            );
        }
        Trap::Exception(Exception::InstructionPageFault) => {
            panic!(
                " InstructionPageFault, core dumped, sepc: 0x{:x}, scause:{:?}",
                cx.sepc,
                scause.cause()
            );
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::trap::time::set_next_trigger();
            crate::syscall::syscall(&mut pcblock,
                crate::syscall::SYS_YIELD,
                [0, 0, 0],
            ) as usize;
        }
        _ => {
            panic!(
                "Unsupported trap {:?}:0x{:x}, stval = {:#x}!",
                scause.cause(),
                scause.bits(),
                stval
            );
        }
    }
    pcblock.set_state(PcbState::Ready);
    drop(pcblock);
    schedule_pcb();
}
