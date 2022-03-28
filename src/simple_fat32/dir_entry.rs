

pub struct ShortDirEntry {
    // 文件名，若在使用中0x0位置为文件名第一个字符，若未使用则为0x00，若曾经被使用过但是现在已经被删除则为0xe5，有多余可以用0x20填充
    file_name: [u8,8],
    // 扩展名
    extension_name: [u8,3],
    // 属性字节，在短文件中不能取值为0x0f，若取此值则为长文件
    flag: u8,
    // 系统保留，(这个位默认为0,只有短文件名时才有用.当为0x00时为文件名全大写,当为0x08时为文件名全小写;0x10时扩展名全大写,0x00扩展名全小写;
    // 当为0x18时为文件名全小写,扩展名全大写)
    reserved: u8,
    // 创建时间的10毫秒位
    creation_time_mm: u8,
    // 文件创建时间，被划分成三个部分
    // 0~4为秒，以2秒为单位
    // 5~10为分，有效值为0~59
    // 11~15为时,有效值为0~23
    creation_time: u16,
    // 文件创建日期
    // 0~4为日，有效值为1~31
    // 5~10为分，有效值为0~59
    // 9~15为年，有效值为0~127，相对于1980年数值
    creation_date: u16,
    // 最近访问日期
    last_access_time: u16,
    // 文件起始簇号的高16位
    start_cluster_high: u16,
    // 文件最近修改时间
    last_modified_time: u16,
    // 文件最近修改日期
    last_modified_date: u16,
    // 文件起始簇号的低16位
    start_cluster_low: u16,
    // 文件长度,单位为字节,子目录项全置0
    file_length: u32,
}

impl ShortDirEntry {
    pub fn new(file_name: [u8,8], extension_name: [u8,3], flag: u8) {
        let name:[u8;8] = clone_into_array(&name_[0..8]);
        let extension_name:[u8;3] = clone_into_array(&extension_name[0..3]);
        self {
            file_name,
            extension_name,
            flag,
            reserved: 0,    
            creation_time_mm: 0,   
            creation_time: 0,    
            creation_date: 0,    
            last_access_time: 0,    
            start_cluster_high: 0,    
            last_modified_time: 0, 
            last_modified_date: 0,    
            start_cluster_low: 0,   
            file_length: 0,
        }
    }
}




pub struct LongDirEntry {
    // 长文件名目录项序列号,从1开始,若是最后一个长文件名目录项,则将其序号|=0x40,若被删除,则设置为0xe5
    flag: u8,
    // 长文件名的1~5个字符,使用unicode码若结束但还有未使用的字节,则先填充2字节00,再填充0xff
    name1: [u8,10],
    // 长目录项属性标志,一定是0x0f
    dir_flag: u8,
    // 保留
    reserved: u8,
    // 校验和,若应一个长文件名需要几个目录项存储,则它们具有相同校验和
    check_sum: u8,
    // 长文件名6~11个字符,未使用的字节用0xff填充
    name2: [u8,12],
    // 保留
    start_cluster: u16,
    // 长文件名12~13个字符,未使用的字节用0xff填充
    name3: [u8,4],
}