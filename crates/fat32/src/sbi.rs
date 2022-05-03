#![allow(unused)]
use core::arch::asm;
pub const SET_TIMER: usize = 0;
pub const PUT_CHAR: usize = 1;
pub const GET_CHAR: usize = 2;
pub const CLEAR_IPI: usize = 3;
pub const SEND_IPI: usize = 4;
pub const REMOTE_FENCE_I: usize = 5;
pub const REMOTE_SFENCE_VMA: usize = 6;
pub const REMOTE_SFENCE_ASID: usize = 7;
pub const SHUTDOWN: usize = 8;

pub fn sbi_call(eid: usize, fid: usize, mut args: [usize; 3]) -> (usize, usize) {
    unsafe {
        asm!("ecall", inout("x10") args[0],
            inout("x11") args[1],
            in("x12") args[2],
            in("x16") fid,
            in("x17") eid);
    };
    (args[0], args[1])
}

pub fn sbi_legacy_call(eid: usize, args: [usize; 3]) -> isize {
    sbi_call(eid, 0, args).0 as isize
}

pub fn sbi_send_ipi(mask: &usize) {
    sbi_legacy_call(SEND_IPI, [mask as *const _ as usize, 0, 0]);
}

pub fn shutdown() -> ! {
    sbi_legacy_call(SHUTDOWN, [0, 0, 0]);
    loop {}
}

pub fn sbi_hsm_hart_start(hartid: usize, start_addr: usize, opaque: usize) -> isize {
    sbi_call(0x48534d, 0, [hartid, start_addr, opaque]).0 as isize
}
