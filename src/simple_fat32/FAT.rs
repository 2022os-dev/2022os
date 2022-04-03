use alloc::sync::Arc;
use lazy_static::*;
use spin::RwLock;


use super::{
    set_start_sector,
    //未加入
    println,
    get_info_buffer,
    read,
    Buffer,
    BufferManager,
    FsInfo,
    ShortDirEntry,
    LongDirEntry,
    BPB,
    Fat32Manager,
};


const BLOCK_SIZE: u32 = 512;
const FAT32_ENTRY_SIZE: u32 = 4;
const FAT_ENTRY_PER_SECTOR: u32 = BLOCK_SIZE / FAT32_ENTRY_SIZE;
const FREE_CLUSTER_ENTRY: u32 = 0x00000000;
const BAD_CLUSTER: u32 = 0x0ffffff7;
const LAST_CLUSTER: u32 = 0x0fffffff;

pub struct FAT {
    // fat表1所在块
    fat1: u32,
    // fat表2所在块
    fat2: u32,
}

impl FAT {

    pub fn new(fat1: u32, fat2: u32,) -> self {
        self {
            fat1,
            fat2,
        }
    }

    pub fn get_position(self, cluster: u32,) -> (u32, u32, u32) {
        (self.fat1 + cluster / FAT_ENTRY_PER_SECTOR, self.fat2 + cluster / FAT_ENTRY_PER_SECTOR, cluster % FAT_ENTRY_PER_SECTOR * FAT32_ENTRY_SIZE)
    }

    // 注意，caller必须保证有足够的free_cluster!!!
    pub fn get_next_free_cluster(&self, current_cluster: u32, dev: u8) -> u32 {
        let current = current_cluster + 1;
        // 这个循环不可能走到尽头
        while current < (self.fat2 - self.fat1) * FAT_ENTRY_PER_SECTOR {
            let (fat1, fat2, off) = self.get_position(current);
            let res = get_info_buffer(fat1, dev).read().read(off,|&fat32_entry: u32| {
                fat32_entry
            });
            if res == FREE_CLUSTER_ENTRY {
                break;
            }
            else {
                current = current + 1;
            }
        }   
        current                                
    }

    pub fn get_next_cluster(&self, current_cluster: u32, dev: u8) -> u32 {
        let (fat1, fat2, off) = self.get_position(current);
        let res = get_info_buffer(fat1, dev).read().read(off,|&fat32_entry: u32| {
            fat32_entry
        });
        if res == BAD_CLUSTER {
            0
        }
        else {
            res
        }                                   
    }

    pub fn set_next_cluster(&self, current: u32, next: u32, dev: u8) {
        let (fat1, fat2, off) = self.get_position(current);
        let res = get_info_buffer(fat1, dev).write().modify(off,|&fat32_entry: u32| {
            fat32_entry = next
        }); 
        let res = get_info_buffer(fat2, dev).write().modify(off,|&fat32_entry: u32| {
            fat32_entry = next
        });                               
    }

    pub fn get_cluster_num(&self, current: u32, dev: u8) -> u32 {
        let (fat1, fat2, off) = self.get_position(current);
        let next = get_info_buffer(fat1, dev).read().read(off,|&fat32_entry: u32| {
            fat32_entry
        });
        let mut res: u32 = 1;
        while next != LAST_CLUSTER {
            let (fat1, fat2, off) = self.get_position(next);
            let next = get_info_buffer(fat1, dev).read().read(off,|&fat32_entry: u32| {
                fat32_entry
            });
            res += 1;
        }
        res                              
    }

    pub fn get_file_last_cluster(&self, current: u32, dev: u8) -> u32 {
        let (fat1, fat2, off) = self.get_position(current);
        let next = get_info_buffer(fat1, dev).read().read(off,|&fat32_entry: u32| {
            fat32_entry
        });
        while next != LAST_CLUSTER {
            let (fat1, fat2, off) = self.get_position(next);
            let next = get_info_buffer(fat1, dev).read().read(off,|&fat32_entry: u32| {
                fat32_entry
            });
        }
        next    
    }

}