#[allow(unused)]
use alloc::sync::Arc;
#[allow(unused)]
use spin::RwLock;

#[allow(unused)]
use super::{
    BLOCK_SIZE,
    DEV,
    get_info_buffer,
};

extern crate alloc;

const FAT32_ENTRY_SIZE: u32 = 4;
const FAT_ENTRY_PER_SECTOR: u32 = BLOCK_SIZE as u32 / FAT32_ENTRY_SIZE;
const FREE_CLUSTER_ENTRY: u32 = 0x00000000;
const BAD_CLUSTER: u32 = 0x0ffffff7;
const LAST_CLUSTER: u32 = 0x0fffffff;


#[allow(unused)]
#[derive(Clone, Copy)]
pub struct FAT {
    // fat表1所在块号
    fat1: u32,
    // fat表2所在块号
    fat2: u32,
}

impl FAT {

    pub fn new(fat1: u32, fat2: u32,) -> FAT {
        Self {
            fat1,
            fat2,
        }
    }

    pub fn get_position(self, cluster: u32,) -> (u32, u32, u32) {
        //需要-2吗
        (self.fat1 + cluster / FAT_ENTRY_PER_SECTOR, self.fat2 + cluster / FAT_ENTRY_PER_SECTOR, cluster % FAT_ENTRY_PER_SECTOR * FAT32_ENTRY_SIZE)
    }

    #[allow(unused)]
    // 注意，caller必须保证有足够的free_cluster!!!
    pub fn get_next_free_cluster(&self, current_cluster: u32, dev: u8) -> u32 {
        let mut current = current_cluster + 1;
        // 这个循环不可能走到尽头
        while current < (self.fat2 - self.fat1) * FAT_ENTRY_PER_SECTOR {
            let (fat1, fat2, off) = self.get_position(current);
            let res = get_info_buffer(fat1, dev).read().read(off as usize,|&fat32_entry: &u32| {
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

    #[allow(unused)]
    pub fn get_next_cluster(&self, current: u32, dev: u8) -> u32 {
        let (fat1, fat2, off) = self.get_position(current);
        let res = get_info_buffer(fat1, dev).read().read(off as usize,|&fat32_entry: &u32| {
            fat32_entry
        });
        if res == BAD_CLUSTER {
            println!(" {} is bad cluster!",current);
            0
        }
        else {
            res
        }                                   
    }

    #[allow(unused)]
    pub fn set_next_cluster(&self, current: u32, next: u32, dev: u8) {
        let (fat1, fat2, off) = self.get_position(current);
        let res = get_info_buffer(fat1, dev).write().modify(off as usize,|fat32_entry: &mut u32| {
            *fat32_entry = next
        }); 
        let res = get_info_buffer(fat2, dev).write().modify(off as usize,|fat32_entry: &mut u32| {
            *fat32_entry = next
        });                               
    }

    #[allow(unused)]
    pub fn get_cluster_num(&self, current: u32, dev: u8) -> u32 {
        let mut cnt = 0;
        let (fat1, fat2, off) = self.get_position(current);
        let mut next = get_info_buffer(fat1, dev).read().read(off as usize,|&fat32_entry: &u32| {
            fat32_entry
        });
        if next == FREE_CLUSTER_ENTRY {
            return 0;
        }
        let mut current = next;
        cnt += 1;
    
        while current != LAST_CLUSTER{
            let (fat1, fat2, off) = self.get_position(current);
            next = get_info_buffer(fat1, dev).read().read(off as usize,|&fat32_entry: &u32| {
                fat32_entry
            });
            current = next;
            cnt += 1;
        }
        cnt                           
    }

    #[allow(unused)]
    pub fn get_file_last_cluster(&self, current: u32, dev: u8) -> u32 {
        
        let (fat1, fat2, off) = self.get_position(current);
        let mut next = get_info_buffer(fat1, dev).read().read(off as usize,|&fat32_entry: &u32| {
            fat32_entry
        });
        
        if next == LAST_CLUSTER {
            return current;
        }
        let mut current = next;
        loop {
            
            let (fat1, fat2, off) = self.get_position(current);
            next = get_info_buffer(fat1, dev).read().read(off as usize,|&fat32_entry: &u32| {
                fat32_entry
            });
            if next == LAST_CLUSTER {
                return current;
            }
            current = next;
        }   
    }

}

