const BLOCK_SIZE : usize = 512;
const DEV : usize = 1;
const DATA_BLOCK_BUFFER_SIZE: usize = 1024;
const INFO_BUFFER_SIZE: usize = 20;
pub type Ino = usize; // 磁盘块号

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use spin::RwLock;

pub struct Buffer {
    block_id: Ino,
    dev: usize,
    data: [u8; BLOCK_SZ],
    modified: bool,
}

impl Buffer {
    pub fn new(block_id: Ino, dev: usize) -> Self {
        let mut data = [0u8; BLOCK_SZ];
        //待实现
        block_device.read_block(block_id, &mut data);
        Self {
            block_id,
            dev,
            data,
            modified: false,
        }
    }

    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
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
            self.block_device.write_block(self.block_id, &self.cache);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.sync()
    }
}

pub struct BufferManager {
    queue: VecDeque<(usize, Arc<RwLock<Buffer>>)>,
    start_sector: usize,
    length: usize,
}

impl BufferManager {
    pub fn new(length: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            start_sector: 0,
            length,
        }
    }

    pub fn get_buffer(
        &mut self,
        block_id: Ino,
        dev: usize,
    ) -> Arc<RwLock<Buffer>> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // substitute
            if self.queue.len() == self.length {
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
            let buffer = Arc::new(Mutex::new(Buffer::new(
                block_id,
                dev,
            )));
            self.queue.push_back((block_id, Arc::clone(&buffer)));
            buffers
        }
    }

    pub fn set_start_sector(&self , new_start_sector: usize) {
        self.start_sector = new_start_sector;
    }

    pub fn get_start_sector(&self , new_start_sector: usize) {
        self.start_sector 
    }
}


lazy_static! {
    pub static ref DATA_BLOCK_BUFFER_MANAGER: RwLock<BufferManager> =
        Mutex::new(BufferManager::new(DATA_BLOCK_BUFFER_SIZE));
}

lazy_static! {
    pub static ref INFO_BUFFER_MANAGER: RwLock<BufferManager> =
        Mutex::new(BufferManager::new(INFO_BUFFER_SIZE));
}

#[derive(PartialEq,Copy,Clone,Debug)]
pub enum RwOption {
    READ,
    WRITE,
}

pub fn get_data_block_buffer(
    block_id: Ino,
    dev: usize,
    op: RwOption
) -> Arc<RwLock<Buffer>> {
    if op == RwOption::READ {
        DATA_BLOCK_BUFFER_MANAGER.read().get_buffer(block_id + self.start_sector, dev)
    }else {
        DATA_BLOCK_BUFFER_MANAGER.write().get_buffer(block_id + self.start_sector, dev)
    }
}

pub fn get_info_buffer(
    block_id: Ino,
    dev: usize,
    op: RwOption
) -> Arc<RwLock<Buffer>> {
    if op == RwOption::READ {
        INFO_BUFFER_MANAGER.read().get_buffer(block_id + self.start_sector, dev)
    }else {
        INFO_BUFFER_MANAGER.write().get_buffer(block_id + self.start_sector, dev)
    }
}

pub fn set_start_sector(new_start_sector: usize) {
    DATA_BLOCK_BUFFER_MANAGER.start_sector = new_start_sector;
    INFO_BUFFER_MANAGER.start_sector = new_start_sector;
}


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