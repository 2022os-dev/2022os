
#[allow(unused)]
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct BPB {
    // 3字节的文件跳转指令
    jump_instruction: [u8;3],
    // 文件系统标志与版本号
    flag_version: u64,
    // 每扇区字节数
    bytes_per_sector: u16,
    // 每簇扇区数
    sectors_per_cluster: u8,
    // 保留扇区数
    reserved_sector_num: u16,
    // fat表个数
    fat_num: u8,
    // fat32必须为0
    root_entries: u16,
    // fat32必须为0
    small_sector: u16,
    // 哪种存储介质，0xf8标准值，可移动存储介质
    media_descriptor: u8,
    // fat32必须为0
    sectors_per_fat_1216: u16,
    // 每磁道扇区数，只对于特殊形状存储介质有效
    sectors_per_track: u16,
    // 磁头数，只对于特殊形状存储介质有效
    number_of_head: u16,
    // ebr分区之前所隐藏的扇区数
    hidden_sector_num: u32,
    // 文件系统总扇区数
    total_sector: u32,


    
    // 每个fat表所占用扇区数
    sectors_per_fat: u32,
    // 标记
    extended_flag: u16,
    // fat32版本号
    fs_version: u16,
    // 根目录所在第一个簇的簇号,通常是0x02
    root_cluster_number: u32,
    // fsinfo(文件系统信息扇区)扇区号,为操作系统提供关于空簇总数及下一个可用簇信息
    fsinfo_sector_num: u16,
    // 备份引导扇区位置,总是位于文件系统的6号扇区
    backup_boot_sector: u16,
    // 保留,供以后扩展使用
    reserved: [u8;12],
}


impl BPB {

    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    pub fn get_sectors_per_cluster(&self) -> u8 {
        self.sectors_per_cluster
    }

    pub fn get_total_sector(&self) -> u32 {
        self.total_sector
    }

    pub fn get_sectors_per_fat(&self) -> u32 {
        self.sectors_per_fat
    }

    #[allow(unused)]
    pub fn get_root_cluster_number(&self) -> u32 {
        self.root_cluster_number
    }
    
    pub fn get_fsinfo_sector_num(&self) -> u16 {
        self.fsinfo_sector_num
    }

    pub fn get_reserved_sector_num(&self) -> u16 {
        self.reserved_sector_num
    }

    #[allow(unused)]
    pub fn get_fat_num(&self) -> u8 {
        self.fat_num
    }

}

#[allow(unused)]
//这东西用不到
pub struct ExtendBPB {
    physical_drive_number: u8,
    reserved: u8,
    // 扩展引导标志
    extended_boot_signature: u8,
    // 卷序列号,通常为随机值
    volume_serial_number: u32,
    // 卷标
    volume_lable: [u8;11],
    // 文件系统格式的ascii码,fat32
    system_id: u64,
}