use alloc::sync::Arc;

const BLOCK_BITS: usize = 64 * 64;

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

    pub fn allocBlock(&self) -> Option<usize> {
        //TODO
        for allocB in 0..self.block_num {
            
        } 
    }

    pub fn deallocBlock(&self) -> Option<usize> {
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