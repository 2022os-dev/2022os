use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;
use core::{cell::{RefCell, RefMut}, borrow::BorrowMut};
use spin::Mutex;
use spin::MutexGuard;

use super::inode::Ino;
use crate::config::BLOCK_BUFFER_SIZE;


pub struct Buffer {
    //设备标识
    pub dev: usize,
    //块号
    pub ino: Ino,
    //缓冲区
    data: Mutex<[u8; BLOCK_BUFFER_SIZE]>,
    //是否修改标识，若被修改，drop后将会写回磁盘
    modified: bool
}

impl Buffer {
    ///将磁盘块内容调入缓冲区
    pub fn create(dev: usize, ino: Ino) -> Self {
        let mut data = [0u8; BLOCK_BUFFER_SIZE];
        //代实现
        read_block(ino, &mut data);
        Self {
            dev,
            ino,
            data,
            modified: false,
        }
    }

    pub fn data(&self) -> MutexGuard<'_, [u8; BLOCK_BUFFER_SIZE]> {
        self.data.lock()
    }

    //获取不可变引用，用于读
    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let size = core::mem::size_of::<T>();
        assert!(offset + size <= BLOCK_BUFFER_SIZE);
        let data = self.data();
        let res = &data[offset] as *const _ as usize;
        unsafe { &*(res as *const T) }
    }
    //获取可变引用，用于写
    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let size = core::mem::size_of::<T>();
        assert!(offset + size <= BLOCK_BUFFER_SIZE);
        self.modified = false;
        let data = self.data();
        let res = &data[offset] as *mut _ as usize;
        unsafe { &*(res as *mut T) }
    }
}

impl Drop for BlockCache {
    //若被修改则写回磁盘
    fn drop(&mut self) {
        if self.modified {
            self.modified = false;
            //待实现
            self.block_device.write_block(self.ino, &self.data);
        }
    }
}

const COUNT_BUFFER: usize = 32;

pub struct BufferManager {
    queue: VecDeque<(usize, Arc<<BlockCache>>)>,
}

impl BufferManager {
    pub fn create() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn get_buffer(&mut self, ino: Ino, dev: usize) -> Arc<Buffer> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == ino) {
            Arc::clone(&pair.1)
        } else {
            if self.queue.len() == COUNT_BUFFER {
                if let Some((idx, _)) = self.queue.iter().enumerate().find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("No so much buffer!");
                }
            }
            // load block into mem and push back
            let buffer = Arc::new(Mutex::new(Buffer::create(ino, dev)));
            self.queue.push_back((block_id, buffer));
            buffer
        }
    }
}



lazy_static! {
    pub static ref BUFFER_MANAGER: Mutex<BufferManager> =
        Mutex::new(BufferManager::new());
}

pub fn get_buffer(
    ino: Ino,
    dev: usize,
) -> Arc<<BlockBuffer> {
    BUFFER_MANAGER
        .lock()
        .get_block_cache(ino, dev)
}
