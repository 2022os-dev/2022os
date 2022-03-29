use core::fmt::Write;

use crate::mm::*;
use crate::process::*;
use crate::sbi::sbi_legacy_call;
use spin::MutexGuard;

pub(super) fn sys_write(
    pcb: &mut MutexGuard<Pcb>,
    fd: usize,
    buf: VirtualAddr,
    len: usize,
) -> isize {
    let mut buffer = alloc::vec![0_u8; len];
    pcb.memory_space
        .copy_from_user(buf, buffer.as_mut_slice());
    const FD_STDOUT: usize = 1;
    match fd {
        FD_STDOUT => {
            let slice = buffer.as_slice();
            let string = core::str::from_utf8(slice).unwrap();
            log!("user_log":>"{}", string);
            len as isize
        }
        _ => {
            panic!("Unsupport syscall write fd {}", fd);
        }
    }
}
