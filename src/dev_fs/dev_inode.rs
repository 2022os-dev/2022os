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
    //最后一次访问文件时间
    pub a_time: u32,
    //最后一次修改索引节点时间
    pub c_time: u32,
    // 最后一次修改文件时间
    pub m_time: u32,
    //硬链接计数器
    pub links_count: u32,
}

impl DevInode {

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

impl InodeOp for DevInode {

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