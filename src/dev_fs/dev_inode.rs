//此变量需随DevInode中除data_block变量的其他变量所占存储空间的增大而减小，128 - 其他变量所占存储空间 = DevInode + 3，单位字节
const DIRECT_COUNT: u32 = 32;

//磁盘块大小，单位字节
const BLOCK_SIZE: u32 = 512;


const INDIRECT1_COUNT: usize = BLOCK_SIZE / 4;
//文件类型，目前支持普通文件和目录，以后可能会加入符号链接、设备、管道等
pub enum DiscInodeType {
    File ,
    Directory,
}

#[derive(Clone, Copy)]
pub struct Authority(u32);

#[repr(C)]
//注意，dev_inode数据结构大小被严格限制为小于等于128字节，使得每个块可以装下4个dev_inode,如图
///  0------------127-----------255-----------383------------511
///  | dev_inode1  | dev_inode2  | dev_inode3  | dev_inode4  |
///  |-------------------------------------------------------|
pub struct DevInode {
    //文件类型
    _type: DiscInodeType,
    //访问权限,第一位为可读，第二位为可写，第三位为可执行，其他保留
    mode: Authority,
    //文件大小，单位字节
    pub size: u32,
    //数据块指针数组，前DIRECT_COUNT位直接指向相应数据块的块号，倒数第三位为间接块1，使用一级间接索引，倒数第二位使用二级间接索引，最后一位使用三级间接索引
    //故文件最大容量为
    //DIRECT_COUNT * BLOCK_SIZE +
    // BLOCK_SIZE / 4 * BLOCK_SIZE + 
    // BLOCK_SIZE / 4 * BLOCK_SIZE / 4 * BLOCK_SIZE+
    // BLOCK_SIZE / 4 *BLOCK_SIZE / 4 * BLOCK_SIZE / 4 * BLOCK_SIZE字节
    pub data_block: [u32,DIRECT_COUNT+3],
    // //最后一次访问文件时间
    // pub a_time: u32,
    // //最后一次修改索引节点时间
    // pub c_time: u32,
    // // 最后一次修改文件时间
    // pub m_time: u32,
    // //硬链接计数器
    // pub links_count: u32,
}

impl DevInode {

    pub fn initialize(&mut self, type_: DiskInodeType) {
        
    }
    pub fn is_dir(&self) -> bool {
        
    }
    #[allow(unused)]
    pub fn is_file(&self) -> bool {
        
    }
    /// Return block number correspond to size.
    pub fn data_blocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }
    fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }
    /// Return number of blocks needed include indirect1/2.
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;
        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1;
        }
        // indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1;
            // sub indirect1
            total +=
                (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT;
        }
        total as u32
    }
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }
    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        
    }
    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Arc<dyn BlockDevice>,
    ) {
        
    }

    /// Clear size to zero and return blocks that should be deallocated.
    ///
    /// We will clear the block contents to zero later.
    pub fn clear_size(&mut self, block_device: &Arc<dyn BlockDevice>) -> Vec<u32> {
        
    }
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        
    }
    /// File size must be adjusted before.
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
    }

}


























const AUT_FLAG_R: usize = 0;
const AUT_FLAG_W: usize = 1;
const AUT_FLAG_X: usize = 2;

bitflags!{
    pub struct PTEFlag: usize {
        const R = 1 << AUT_FLAG_R ;
        const W = 1 << AUT_FLAG_W ;
        const X = 1 << AUT_FLAG_X ;
    }
}