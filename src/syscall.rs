use spin::MutexGuard;
use spin::Mutex;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::process::pcb::alloc_pid;
use crate::process::{Pcb, PcbState};
use crate::mm::*;
use crate::task::scheduler_ready_pcb;

pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_YIELD: usize = 124;
pub const SYS_FORK: usize = 451;

pub fn syscall(pcb: &mut MutexGuard<Pcb>, id: usize, param: [usize; 3]) -> isize {
    match id {
        SYS_WRITE => sys_write(pcb, param[0], param[1] as *const u8, param[2]),
        SYS_EXIT => {
            sys_exit(pcb, param[0]);
            0
        },
        SYS_FORK => sys_fork(pcb),
        SYS_YIELD => sys_yield(pcb, param[0]),
        _ => {
            panic!("No Implement syscall: {}", id);
        }
    }
}

fn sys_write(pcb: &mut MutexGuard<Pcb>, fd: usize, buf: *const u8, len: usize) -> isize {
    let mut buffer = alloc::vec![0_u8; len];
    pcb.memory_space.copy_to_user(
        VirtualAddr(buf as usize),
        len,
        buffer.as_mut_slice(),
    );
    const FD_STDOUT: usize = 1;
    match fd {
        FD_STDOUT => {
            let slice = buffer.as_slice();
            let string = core::str::from_utf8(slice).unwrap();
            for c in string.chars() {
                crate::sbi::sbi_call(crate::sbi::PUT_CHAR, [c as usize, 0, 0]);
            }
            0
        }
        _ => {
            panic!("Unsupport syscall");
        }
    }
}

fn sys_exit(pcb: &mut MutexGuard<Pcb>, xstate: usize) {
    crate::println!("[kernel] Application exit with code {}", xstate);
    pcb.exit();
}

fn sys_yield(_: &mut MutexGuard<Pcb>, _: usize) -> isize {
    println!("[kernel] syscall Yield");
    0
}

fn sys_fork(pcb: &mut MutexGuard<Pcb>) -> isize {
    let mut child_ms = pcb.memory_space.copy();
    let pte = child_ms.pgtbl.walk(MemorySpace::trapframe_page().offset(0), false);
    if pte.is_valid() {
        if pte.is_leaf() {
            println!("Leaf");
        } else {
            println!("Valid");
        }
    }
    let pid = alloc_pid();
    let child = Arc::new(Mutex::new(Pcb {
        pid: pid,
        state: PcbState::Ready,
        memory_space: child_ms,
        children: Vec::new()
    }));
    child.lock().trapframe()["a0"] = 0;
    child.lock().trapframe()["satp"] = child_ms.pgtbl.root.page() | 0x8000000000000000 ;
    pcb.children.push(child.clone());
    scheduler_ready_pcb(child);
    pid as isize
}
