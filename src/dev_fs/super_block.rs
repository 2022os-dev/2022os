
const EFS_MAGIC: u32 = 0x3b800001;
pub struct SuperBlock{
    magic_num:u32,
    pub total_blocks:u32,
    pub inode_bitmap:u32,
    pub data_bitmap:u32,
    pub inode_block:u32,
    pub data_block:u32,
}

impl SuperBlock {
    pub fn init (
    &mut self
    total_blocks: u32,
    inode_bitmap: u32,
    data_bitmap: u32,
    inode_block: u32,
    data_block: u32,
    ) {
        *self=Self{
            magic_num: EFS_MAGIC,
            total_blocks,
            inode_bitmap,
            data_bitmap,
            inode_block,
            data_block,
        }
    }

    pub fn isValid(&self) -> bool {
        self.magic_num == EFS_MAGIC
    }
}
