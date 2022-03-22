use alloc::sync::Arc;
//磁盘块大小，单位字节
const BLOCK_SIZE: u32 = 512;
//每个磁盘块比特位数
const BLOCK_BITS: usize = BLOCK_SIZE * 8;

type BitmapBlock = [u64; 64];

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

    pub fn alloc(&self, dev: usize) -> Option<usize> {
        //TODO
        for id in 0..self.block_num {
            //获取相应缓冲区块
            let buffer = get_buffer(id + self.start as usize, dev);
            let bitmap_block = buffer.get_mut<BitmapBlock>(0);
            let pos = if let Some((bits64_pos, inner_pos)) = bitmap_block.iter().enumerate()
            .find(|(_, bits64)| **bits64 != u64::MAX)
            .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
            {
                // modify cache
                bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                Some(id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
            } else {
                None
            }
            if pos.is_some() {
                return pos;
            }
        }
        None
    }

    pub fn dealloc(&self, dev: usize, bit: usize) -> Option<usize> {
        //TODO
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        let buffer = get_buffer(block_pos + self.start_block_id, dev));
        let bitmap_block = buffer.get_mut<BitmapBlock>(0);
        assert!(buffer[bits64_pos] & (1u64 << inner_pos) > 0);
        buffer[bits64_pos] -= 1u64 << inner_pos;
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