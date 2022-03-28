pub const PAGE_OFFSET_BIT: usize = 12;
pub const PAGE_SIZE: usize = 4096;
pub const SV39_VPN_BIT: usize = 9;
pub const PAGE_TABLE_LEVEL: usize = 3;

pub const PHYS_FRAME_END: usize = 0x80f00000;
pub const USER_STACK: usize = 0x80000000 - PAGE_SIZE;
pub const BOOT_STACK_SIZE: usize = 2 * PAGE_SIZE;

pub const PTE_FLAG_SIZE: usize = 8;
pub const PTE_PPN_OFFSET: usize = 10;
