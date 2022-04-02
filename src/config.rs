pub const PAGE_OFFSET_BIT: usize = 12;
pub const PAGE_SIZE: usize = 4096;
pub const SV39_VPN_BIT: usize = 9;
pub const PAGE_TABLE_LEVEL: usize = 3;

// 物理内存终止页
pub const PHYS_FRAME_END: usize = 0x80f00000;
// 用户栈映射的虚拟地址, 用户栈大小为一个页
pub const USER_STACK_PAGE: usize = 0x80000000 - PAGE_SIZE;
// 每个hart使用的栈大小
pub const BOOT_STACK_SIZE: usize = 2 * PAGE_SIZE;
// 定时器频率
#[cfg(feature = "board_unleashed")]
pub const RTCLK_FREQ: usize = 1000_000; // 1M Hz

pub const PTE_FLAG_SIZE: usize = 8;
pub const PTE_PPN_OFFSET: usize = 10;
