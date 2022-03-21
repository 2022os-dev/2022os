use alloc::sync::Arc;
//磁盘块大小，单位字节
const BLOCK_SIZE: u32 = 512;
//每个磁盘块比特位数
const BLOCK_BITS: usize = BLOCK_SIZE * 8;

pub struct Bitmap{
    pub start: usize,
    pub block_num: usize,
}

impl Bitmap{
    
    pub fn new(start: usize, block_num: usize) Self {
        Self {
            start,
            block_num,
        }
    }

    pub fn alloc(&self) -> Option<usize> {
        //TODO
        for allocB in 0..self.block_num {
            
        } 
    }

    pub fn dealloc(&self) -> Option<usize> {
        //TODO
    }

    pub fn getPosition(mut bit: usize) -> (usize, usize, usize) {
        let block_pos = bit / BLOCK_BITS;
        bit % = BLOCK_BITS;
        (block_pos, bit / 64, bit % 64)
    }

    pub fn getTotal(&self) {
        self.block_num * BLOCK_BITS
    }

}