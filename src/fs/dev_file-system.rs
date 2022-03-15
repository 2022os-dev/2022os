const SUPERBLOCKBLOCKNUM: usize = 1;

use alloc::sync::Arc;

use spin::Mutex;

//磁盘存储结构
///  |------------|--------------|-------------|-------------|--------------|
///  | superblock | inode_bitmap | data_bitmap | inode_block |  data_block  |
///  |----------------------------------------------------------------------|
/// 
/// 
//inode存储在inode_block块中，inode大小被设定为块大小BLOCK_SIZE的四分之一，必须设定为整除BLOCK_SIZE
/// |-------------------------------------|
///  | inode1 | inode2 | inode3 | inode4 |
///  |-----------------------------------|


pub struct DevFileSystem {
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    pub inode_block: usize,
    pub data_block: usize,
}

impl DevFileSystem {
    pub fn new (inode_bitmap_blocks: u32, total: u32) Arc<Mutex<self>> {
        let inode_bitmap: Bitmap = Bitmap::new(SUPERBLOCKBLOCKNUM, inode_bitmap_blocks as usize);
        let inode_num: usize = inode_bitmap.getTotal();
        //core::mem::size_of::<DiscInode>()为512/4字节，其必须整除BLOCK_SIZE！
        let inode_block_num: usize = (inode_num * core::mem::size_of::<DiscInode>() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let total_data_block_num = total - SUPERBLOCKBLOCKNUM - inode_block_num - inode_bitmap_blocks;
        //每BLOCK_BITS4096个数据块需要一个数据位图块，通过此结论计算数据位图块和数据块的数量
        let data_bitmap_blocks = (total_data_block_num + BLOCK_BITS) / (BLOCK_BITS + 1);
        let data_block_num: usize = total_data_block_num - data_bitmap_blocks;
        let data_bitmap: Bitmap = BitMap::new(SUPERBLOCKBLOCKNUM + inode_bitmap_blocks + let inode_block_num, data_bitmap_blocks);

        //向前SUPERBLOCKBLOCKNUM块中写入superblock内容

        //初始化文件系统根目录，在第一个inode_block中

        let mut dfs = Self {
            inode_bitmap,
            data_bitmap,
            SUPERBLOCKBLOCKNUM + inode_bitmap_blocks + data_bitmap_blocks,
            SUPERBLOCKBLOCKNUM + inode_bitmap_blocks + data_bitmap_blocks + inode_block_num
        }

        Arc::new(Mutex::new(dfs))
    }

    //创建索引节点，相当于创建文件或目录，调用inode相关函数
    pub fn createInode () {
        //TODO
    }
   
    //删除索引节点，相当于删除文件或目录，调用inode相关函数
    pub fn deleteInode () {
        //TODO
    }

    //读取索引节点代表的文件或者目录内容，调用inode和DirectoryEntry相关函数
    pub fn read () {
        //TODO
    }
    
    //往索引节点代表的文件或目录写入内容
    pub fn write () {
        //TODO
    }
    //获取索引节点位于的块号和块内偏移
    pub fn getInodePosition () -> (usize,usize) {
        //TODO
    }
}