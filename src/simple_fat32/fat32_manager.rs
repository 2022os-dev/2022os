

pub struct Fat32Manager {
    // 每扇区字节数
    bytes_per_sector: u16,
    // 每簇扇区数
    sectors_per_cluster: u8,
    // 保留扇区数
    reserved_sector_num: u16,
    // 文件系统总扇区数
    total_sector: u32,
    // 根目录所在第一个簇的簇号,通常是0x02
    root_cluster_number: u32,
    // fsinfo(文件系统信息扇区)扇区号,为操作系统提供关于空簇总数及下一个可用簇信息
    fsinfo_sector_num: u16,
    // 设备号,待实现
    dev: u8,
    
    fat:Arc<RwLock<FAT>>,

}