use alloc::sync::Arc;
use core::{cell::{RefCell, RefMut}, borrow::BorrowMut};
use spin::Mutex;
use spin::MutexGuard;

use super::inode::Ino;
use crate::config::BLOCK_BUFFER_SIZE;

pub struct Buffer {
    pub dev: usize,
    pub ino: Ino,
    data: Mutex<[u8; BLOCK_BUFFER_SIZE]>
}

impl Buffer {
    pub fn data(&self) -> MutexGuard<'_, [u8; BLOCK_BUFFER_SIZE]> {
        self.data.lock()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // TODO: 引用数清零, 写回磁盘
        // TODO: 识别出写过的块，只对写过的块进行写回，有效提高效率
    }
}

pub fn get_block(dev: usize, ino: Ino) -> Arc<Buffer> {
    // TODO: read block
    // Feature: 可以使用DMA的方式读取磁盘块，使当前进程进入阻塞态，
    //          等IO完成后再回到当前函数返回Buffer
    Arc::new(Buffer {
        dev: 0,
        ino: 0,
        data: Mutex::new([0; BLOCK_BUFFER_SIZE])
    })
}
