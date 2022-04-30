// #![no_std]
extern crate alloc;

pub const BLOCK_SIZE: usize = 512;
pub const DEV: u8 = 1;

pub mod bcache;
pub mod bpb;
pub mod dir_entry;
pub mod fat;
pub mod fat32_manager;
pub mod fsinfo;
pub mod vfs;

pub use bcache::{block_cache_sync_all, get_data_block_buffer, get_info_buffer, set_start_sector};

use super::*;
pub use bcache::*;
pub use bpb::*;
pub use dir_entry::ShortDirEntry;
pub use dir_entry::*;
pub use fat::FAT;
pub use fat32_manager::Fat32Manager;
pub use fsinfo::*;
pub use vfs::VFSFile;
pub use vfs::FAT32_ROOT;
