//磁盘块大小，单位字节
const BLOCK_SIZE: u32 = 512;

//文件系统合法性验证魔数
const EFS_MAGIC: u32 = 0x3b800001;


//占据且只占据第一个磁盘块，图示见dev_sb_info.rs
pub struct DevSuperBlock{
    //文件系统合法性验证的魔数,参考rcore
    magic_num:u32,
    //文件系统总块数
    pub total_blocks:u32,
    //inode位图开始块号
    pub inode_bitmap:u32,
    //数据块位图开始块号
    pub data_bitmap:u32,
    //inode块号
    pub inode_block:u32,
    //数据块块号
    pub data_block:u32,
    //该文件系统块大小
    pub block_size,u32,
}

impl DevSuperBlock {
    // 初始化superblock
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
            block_size: BLOCK_SIZE,
        }
    }
    // 判断文件系统是否合法
    pub fn isValid(&self) -> bool {
        self.magic_num == EFS_MAGIC
    }
}
