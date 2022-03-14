const DIRECT_COUNT: u32 = 32;
const BLOCK_SIZE: u32 = 512;
const INDIRECT1_COUNT: usize = BLOCK_SIZE / 4;

pub enum DiscInodeType {
    File ,
    Directory,
}

#[repr(C)]
pub struct DiscInode {
    dtype: DiscInodeType,
    pub size: u32,
    pub contentD: [u32,DIRECT_COUNT],
    pub contentI1: u32,
    pub contentI2: u32,
}

impl DiscInode {

    pub fn init(&self, dtype: DiscInodeType) {
        self.dtype = dtype,
        self.block = block,
        self.offset = offset,
        self.size = 0,
        self.contentD.iter_mut().for_each(|v| *v = 0);
        self.contentI1 = 0;
        self.contentI2 = 0;
    }

    pub fn isDirectory(&self) -> bool {
        self.dtype == DiscInodeType::Directory
    }

    pub fn dataBlockNum(size: u32) -> usize {
        (size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32
    }

    pub fn allBlocksNeed(size: u32) -> u32 {

        let blockNum = Self::dataBlockNum(size) as usize;
        let mut total = blockNum as usize;
        if blockNum > DIRECT_COUNT {
            total += 1;
        }
        if blockNum > INDIRECT1_COUNT + DIRECT_COUNT{
            
            total += (blockNum - INDIRECT1_COUNT - DIRECT_COUNT + INDIRECT1_COUNT - 1) / INDIRECT1_COUNT;
            total += 1;
        }
        total as u32
    }

    pub fn increase() {
        //TODO
    }

    pub fn delete() {
        //TODO
    }

    pub fn read(
        &self
        off: usize,
        buf: &mut [u8],
    ) {
        //TODO

    }

    pub fn write(
        &self
        off: usize,
        buf: &mut [u8],
    ) {
        //TODO
    }

    
}