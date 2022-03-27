use crate::asm;
use core::arch::global_asm;
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        asm!(
        "mv tp, a0",
        "la sp, boot_stack_top",
        "li a1, 16384",
        "mul a0, a0, a1",
        "sub sp, sp, a0",
        "j kernel_start", options(noreturn));
    }
}
global_asm!(
    ".section .bss.stack
    .globl boot_stack
boot_stack:
    .space 16384 * 4
    .globl boot_stack_top
boot_stack_top:"
);
