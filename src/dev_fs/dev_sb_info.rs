

use alloc::sync::Arc;

use spin::Mutex;

//磁盘存储结构
///  |----------------|--------------|-------------|-------------|--------------|
///  | dev_superblock | inode_bitmap | data_bitmap | inode_block |  data_block  |
///  |--------------------------------------------------------------------------|
/// 
/// 
//inode存储在inode_block块中，inode大小被设定为块大小BLOCK_SIZE的四分之一，必须设定为整除BLOCK_SIZE
/// |-------------------------------------|
///  | inode1 | inode2 | inode3 | inode4 |
///  |-----------------------------------|


pub struct DevSbInfo {
    // // 空闲索引节点个数
    // pub free_inode_count: usize,
    // // 空闲数据块个数
    // pub free_data_block-count: usize,
    // // 文件系统总块数
    // pub block-count: usize,
    //索引节点位图
    pub inode_bitmap: Bitmap,
    //数据块位图
    pub data_bitmap: Bitmap,
    //索引节点开始块号
    pub inode_block: usize,
    // 数据块开始块号
    pub data_block: usize,
}

impl DevSbInfo {

    pub fn new (inode_bitmap_blocks: u32, total: u32) -> Arc<Mutex<self>> {
    //TODO
        //inode位图
        let inode_bitmap: Bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_num: usize = inode_bitmap.getTotal();
        //core::mem::size_of::<DiscInode>()为512/4字节，其必须整除BLOCK_SIZE！
        let inode_block_num: usize = (inode_num * core::mem::size_of::<DiscInode>() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let total_data_block_num = total - 1 - inode_block_num - inode_bitmap_blocks;
        //每BLOCK_BITS4096个数据块需要一个数据位图块，通过此结论计算数据位图块和数据块的数量
        let data_bitmap_blocks = (total_data_block_num + BLOCK_BITS) / (BLOCK_BITS + 1);
        let data_block_num: usize = total_data_block_num - data_bitmap_blocks;
        //数据块位图
        let data_bitmap: Bitmap = BitMap::new(1 + inode_bitmap_blocks, data_bitmap_blocks);

        //向前SUPERBLOCKBLOCKNUM块中写入superblock内容

        //初始化文件系统根目录，在第一个inode_block中

        let mut dfs = Self {
            // inode_bitmap.getTotal(),
            // data_bitmap.getTotal(),
            // total,
            inode_bitmap,
            data_bitmap,
            SUPERBLOCKBLOCKNUM + inode_bitmap_blocks + data_bitmap_blocks,
            SUPERBLOCKBLOCKNUM + inode_bitmap_blocks + data_bitmap_blocks + inode_block_num
        }

        Arc::new(Mutex::new(dfs))
    }

    pub fn createBySb() -> Arc<Mutex<Self>> {
    //TODO
        // 通过读取超级快内容来构造dev_sb_info
        
    }

    pub fn root_inode(efs: &Arc<Mutex<Self>>) -> Inode {
        
    }

    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        
    }

    pub fn get_data_block_id(&self, data_block_id: u32) -> u32 {
        
    }

    pub fn alloc_inode(&mut self) -> u32 {
        self.inode_bitmap.alloc(&self.block_device).unwrap() as u32
    }

    pub fn dealloc_inode(&mut self, inode_id: u32) {
        
    }

    /// Return a block ID not ID in the data area.
    pub fn alloc_data(&mut self) -> u32 {
        self.data_bitmap.alloc(&self.block_device).unwrap() as u32 + self.data_area_start_block
    }

    pub fn dealloc_data(&mut self, block_id: u32) {
        
    }
}

impl SuperBlockOp for DevSbInfo{
    //文件系统初始化
    fn fsinit(&self, sb: *mut SuperBlock) -> Result<(), SuperOpErr> {
        
        createBySb();
    }

    //分配一个与文件系统关联的Inode对象
    fn alloc_inode(&self, sb: &mut SuperBlock) -> Result<*mut Inode, SuperOpErr>{
        //用createInode相应代码
        alloc_inode();
    }

    //在内存与磁盘同时删除Inode节点，需要检查引用数是否归0
    fn delete_inode(&self, inode: &mut Inode) -> Result<(), SuperOpErr> {
        dealloc_inode();
    }

    
    fn alloc_block(&self, sb: &mut SuperBlock) -> Result<*mut Inode, SuperOpErr> {
        alloc_data();
    }

    
    fn dealloc_block(&self, inode: &mut Inode) -> Result<(), SuperOpErr> {
        dealloc_data();
    }

    //从磁盘读取inode数据，必须保证inode的i_ino被正确填写，通过i_ino读取inode
    fn read_inode(&self, inode: &mut Inode) -> Result<(), SuperOpErr> {
        //可参考read源代码
    }

    // 将Inode结构写回磁盘
    fn write_inode(&self, inode: &mut Inode) -> Result<(), SuperOpErr> {
        //可参考read源代码
    }

    // 将Inode引用数减1
    fn put_inode(&self , inode: &mut Inode) -> Result<(), SuperOpErr> {

    }

    // 与磁盘同步SuperBlock
    fn write_super(&self, sb: &mut SuperBlock) -> Result<(), SuperOpErr> {
        
    }
}