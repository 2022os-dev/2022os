// #![no_std]
extern crate alloc;

pub const BLOCK_SIZE : usize = 512;
pub const DEV : u8 = 1;

pub mod console;
pub mod sbi;
pub mod dir_entry;
pub mod bcache;
pub mod bpb;
pub mod fat;
pub mod fat32_manager;
pub mod fsinfo;
pub mod vfs;

pub use bcache::{get_data_block_buffer,get_info_buffer,block_cache_sync_all,set_start_sector};


pub use dir_entry::ShortDirEntry;
pub use dir_entry::*;
pub use bcache::*;
pub use bpb::*;
pub use fat::FAT;
pub use fat32_manager::Fat32Manager;
pub use fsinfo::*;
pub use vfs::VFSFile;
use super::*;