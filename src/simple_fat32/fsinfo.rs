

pub struct FsInfo {
    // 固定值0x41625252
    lead_sig: u32,
    // 保留使用
    reserved1: [u8,480],
    // 固定值0x61417272
    struct_sig: u32,
    // 当前分区free cluster个数,若此值为0xffffffff,说明free cluster个数未知
    free_clustor_num: u32,
    // 下一可用簇号
    next_free_clustor: u32,
    // 保留使用
    reserved2: [u8,12],
    // 固定值0xaa550000
    trail_sig: u32,
}