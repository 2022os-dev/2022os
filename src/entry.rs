use core::arch::global_asm;

global_asm!(
    ".section .text.entry
    .globl _start
_start:
    addi a0, a0, 65
    li a7, 1
    ecall
    mv tp, a0
    la sp, boot_stack_top
    li a1, 16384
    mul a0, a0, a1
    sub sp, sp, a0
    j kernel_start

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 16384 * 4
    .globl boot_stack_top
boot_stack_top:"
);
