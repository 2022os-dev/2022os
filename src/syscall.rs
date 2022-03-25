use spin::MutexGuard;
use crate::process::Pcb;
use crate::mm::*;
use crate::task::*;

pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_YIELD: usize = 124;

pub fn syscall(pcb: &mut MutexGuard<Pcb>, id: usize, param: [usize; 3]) -> isize {
    match id {
        SYS_WRITE => sys_write(pcb, param[0], param[1] as *const u8, param[2]),
        SYS_EXIT => {
            sys_exit(pcb, param[0]);
        }
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

fn sys_exit(pcb: &mut MutexGuard<Pcb>, xstate: usize) -> ! {
    crate::println!("[kernel] Application exit with code {}", xstate);
    pcb.exit();
    schedule_pcb();
}

fn sys_yield(pcb: &mut MutexGuard<Pcb>, _: usize) -> isize {
    println!("[kernel] syscall Yield");
    0
}
