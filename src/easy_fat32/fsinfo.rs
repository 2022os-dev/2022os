const LEADING_SIGNATURE: u32 = 0x41625252;
const STRUCT_SIGNATURE: u32 = 0x61417272;
const TRAILING_SIGNATURE: u32 = 0xaa550000;


// 注意，该数据结构从磁盘中提取出来后修改是不会同步到磁盘，也就意味着磁盘的不一定正确，如果日后此处必须与磁盘同步，则修改为直接在fsinfo所在块中进行操作

#[allow(unused)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct FsInfo {
    // 固定值0x41625252
    lead_sig: u32,
    // 保留使用
    reserved1: [u8;480],
    // 固定值0x61417272
    struct_sig: u32,
    // 当前分区free cluster个数,若此值为0xffffffff,说明free cluster个数未知
    free_clustor_num: u32,
    // 下一可用簇号
    first_free_clustor: u32,
    // 保留使用
    reserved2: [u8;12],
    // 固定值0xaa550000
    trail_sig: u32,
}

impl FsInfo {

    pub fn check_lead_sig(&self) -> bool {
        self.lead_sig == LEADING_SIGNATURE
    }

    pub fn check_struct_sig(&self) -> bool {
        self.struct_sig == STRUCT_SIGNATURE
    }

    pub fn check_trail_sig(&self) -> bool {
        self.trail_sig == TRAILING_SIGNATURE
    }

    pub fn get_lead_sig(&self) -> u32 {
        self.lead_sig 
    }

    pub fn get_struct_sig(&self) -> u32 {
        self.struct_sig 
    }

    pub fn get_trail_sig(&self) -> u32 {
        self.trail_sig 
    }

    pub fn get_free_cluster_num(&self) -> u32 {
        self.free_clustor_num
    }

    pub fn get_first_free_cluster(&self) -> u32 {
        self.first_free_clustor
    }

    pub fn set_free_cluster_num(&mut self, free_clustor_num: u32) {
        self.free_clustor_num = free_clustor_num
    }

    pub fn set_first_free_cluster(&mut self, first_free_clustor: u32) {
        self.first_free_clustor = first_free_clustor
    }
}