pub const PAGE_OFFSET_BIT: usize = 12;
pub const PAGE_SIZE: usize = 4096;
pub const SV39_VPN_BIT: usize = 9;
pub const PAGE_TABLE_LEVEL: usize = 3;

// 物理内存终止页
pub const PHYS_FRAME_END: usize = 0x83f00000;
// 用户栈映射的虚拟地址, 用户栈大小为一个页, 暂时不支持修改
pub const USER_STACK_SIZE: usize = PAGE_SIZE;
pub const USER_STACK_PAGE: usize = 0x80000000 - USER_STACK_SIZE;
// 每个hart使用的栈大小
pub const BOOT_STACK_SIZE: usize = 2 * PAGE_SIZE;
// 定时器频率
#[cfg(feature = "board_unleashed")]
pub const RTCLK_FREQ: usize = 1000_000; // 1M Hz

pub const PTE_FLAG_SIZE: usize = 8;
pub const PTE_PPN_OFFSET: usize = 10;
// 每个进程最多能打开的文件
pub const MAX_FDS: usize = 1024;

// 文件目录最大长度
pub const PATH_LIMITS: usize = 512;

pub const PERI_START: usize = 0x10040000;
pub const PERI_END: usize = 0x10060000;
