use core::arch::global_asm;

global_asm!(
    ".section .bss.stack
    .globl boot_stack
boot_stack:
    .space 8192 * 5
    .globl boot_stack_top
boot_stack_top:

    .section .text.entry
    .globl _start
_start:
    mv tp, a0
    # addi a0, a0, 48
    # li a7, 1
    # ecall
    la sp, boot_stack_top
    # 8192 = 2 ^ 13
    li a1, 13
    sll a0, tp, a1
    # sp = boot_stack_top - 8192 * a0
    sub sp, sp, a0
    j kernel_start"
);
