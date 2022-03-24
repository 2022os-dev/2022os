OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x0;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
    }
    frames = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
