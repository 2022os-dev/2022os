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
    let mut buf: PhysAddr = buf.into();
    let buf: &[u8] = buf.as_slice(len);
    const FD_STDOUT: usize = 1;
    match fd {
        FD_STDOUT => {
            let string = core::str::from_utf8(buf).unwrap();
            log!("user_log":>"{}", string);
            len as isize
        }
        _ => {
            panic!("Unsupport syscall write fd {}", fd);
        }
    }
}
