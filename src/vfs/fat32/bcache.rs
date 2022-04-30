// use std::fs::OpenOptions;
// use std::io::prelude::*;
// use std::fs::File;
// use std::io::SeekFrom;

const BLOCK_SIZE : usize = 512;
#[allow(unused)]
const DEV : u8 = 1;
const DATA_BLOCK_BUFFER_SIZE: u32 = 1024;
const INFO_BUFFER_SIZE: u32 = 20;
pub type Ino = u32; // 磁盘块号


use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use spin::RwLock;

#[allow(unused)]
pub struct Buffer {
    
    block_id: Ino,
    dev: u8,
    
    data: [u8; BLOCK_SIZE],
    modified: bool,
}

impl Buffer {
    pub fn new(block_id: Ino, dev: u8) -> Self {
        let mut data = [0; BLOCK_SIZE];
        read_block(block_id, &mut data);
        Self {
            block_id,
            dev,
            data,
            modified: false,
        }
    }

    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.data[offset] as *const _ as usize
    }

    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SIZE);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SIZE);
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }

    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            // 待实现
            write_block(self.block_id, &self.data);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.sync()
    }
}

pub struct BufferManager {
    queue: VecDeque<(u32, Arc<RwLock<Buffer>>)>,
    start_sector: u32,
    volume: u32,
}

impl BufferManager {
    pub fn new(volume: u32) -> Self {
        Self {
            queue: VecDeque::new(),
            start_sector: 0,
            volume,
        }
    }

    pub fn get_buffer(
        &mut self,
        block_id: Ino,
        dev: u8,
    ) -> Arc<RwLock<Buffer>> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // substitute
            if self.queue.len() == self.volume as usize{
                // from front to tail
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("Run out of Buffer!");
                }
            }
            // load block into mem and push back
            let buffer = Arc::new(RwLock::new(Buffer::new(
                block_id,
                dev,
            )));
            self.queue.push_back((block_id, Arc::clone(&buffer)));
            buffer
        }
    }

    // pub fn read_buffer(
    //     &mut self,
    //     block_id: Ino,
    //     dev: usize,
    // ) -> Option<Arc<RwLock<Buffer>>> {
    //     if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
    //         Option(Arc::clone(&pair.1))
    //     } else {
    //         None
    //     }
    // }

    #[allow(unused)]
    pub fn set_start_sector(&mut self , new_start_sector: u32) {
        self.start_sector = new_start_sector;
    }

    #[allow(unused)]
    pub fn get_start_sector(&self) -> u32{
        self.start_sector 
    }

    //测试
    // pub fn add(&mut self ,id: Ino) {
    //     self.queue.push_back((id, Arc::new(RwLock::new(Buffer::new(id, DEV)))));
    // }

}


lazy_static! {
    pub static ref DATA_BLOCK_BUFFER_MANAGER: RwLock<BufferManager> =
        RwLock::new(BufferManager::new(DATA_BLOCK_BUFFER_SIZE));
}

lazy_static! {
    pub static ref INFO_BUFFER_MANAGER: RwLock<BufferManager> =
        RwLock::new(BufferManager::new(INFO_BUFFER_SIZE));

}

// #[derive(PartialEq,Copy,Clone,Debug)]
// pub enum RwOption {
//     READ,
//     WRITE,
// }

pub fn get_data_block_buffer(
    block_id: Ino,
    dev: u8,
) -> Arc<RwLock<Buffer>> {
    let id = block_id + DATA_BLOCK_BUFFER_MANAGER.read().start_sector;
    // if op == RwOption::READ {
    //     DATA_BLOCK_BUFFER_MANAGER.write().get_buffer(id, dev);
    //     DATA_BLOCK_BUFFER_MANAGER.read().read_buffer(id, dev).unwrap()
    // }else {
    //     DATA_BLOCK_BUFFER_MANAGER.write().get_buffer(id, dev)
    // }
    DATA_BLOCK_BUFFER_MANAGER.write().get_buffer(id, dev)
}

pub fn get_info_buffer(
    block_id: Ino,
    dev: u8,
) -> Arc<RwLock<Buffer>> {
    let id = block_id + INFO_BUFFER_MANAGER.read().start_sector;
    // if op == RwOption::READ {
    //     INFO_BUFFER_MANAGER.write().get_buffer(id, dev);
    //     INFO_BUFFER_MANAGER.read().get_buffer(id, dev).unwrap()
    // }else {
    //     INFO_BUFFER_MANAGER.write().get_buffer(id, dev)
    // }
    
    INFO_BUFFER_MANAGER.write().get_buffer(id, dev)
}

pub fn set_start_sector(new_start_sector: u32) {
    DATA_BLOCK_BUFFER_MANAGER.write().start_sector = new_start_sector;
    INFO_BUFFER_MANAGER.write().start_sector = new_start_sector;
}


#[allow(unused)]
pub fn block_cache_sync_all() {
    let manager = DATA_BLOCK_BUFFER_MANAGER.write();
    for (_, cache) in manager.queue.iter() {
        cache.write().sync();
    }
    let manager = INFO_BUFFER_MANAGER.write();
    for (_, cache) in manager.queue.iter() {
        cache.write().sync();
    }
}

#[allow(unused)]
pub fn read_block(id: Ino, buf: &mut [u8]) {
    // let mut f = File::open("D:/gittest/disk").unwrap();
    // let off = id * 512;
    // f.seek(SeekFrom::Start(off as u64));
    // let n = f.read(&mut buf[..]).unwrap();
    // if n !=BLOCK_SIZE {
    //     panic!("do not read 512 bytes from disk!");
    // }
}

#[allow(unused)]
pub fn write_block(id: Ino, buf: &[u8]) {
    // let mut f = OpenOptions::new().read(true).write(true).open("D:/gittest/disk").unwrap();
    // let off = id * 512;
    // f.seek(SeekFrom::Start(off as u64));
    // let n = f.write(& buf[..]).unwrap();
    // if n !=BLOCK_SIZE {
    //     panic!("do not write 512 bytes from disk!");
    // }
}