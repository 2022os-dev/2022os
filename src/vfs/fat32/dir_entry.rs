use alloc::sync::Arc;
use alloc::string::String;
#[allow(unused)]
use alloc::vec::Vec;
use spin::RwLock;


#[allow(unused)]
use super::{
    BLOCK_SIZE,
    DEV,
    get_info_buffer,
    get_data_block_buffer,
    fat32_manager::Fat32Manager,
    fat::FAT,
    println,
    log,
};

#[allow(unused)]
const READ_AND_WRITE: u8 = 0b00000000;
#[allow(unused)]
const READ_ONLY: u8 = 0b00000001;
#[allow(unused)]
const HIDDEN: u8 = 0b00000010;
#[allow(unused)]
const SYSTEM: u8 = 0b00000100;
#[allow(unused)]
const LABLE: u8 = 0b00001000;
#[allow(unused)]
const SUB_DIRECTORY: u8 = 0b00010000;
#[allow(unused)]
const FILING: u8 = 0b00100000;
const LONG_DIR_ENTRY: u8 = 0b00001111;

type DataBlock = [u8; BLOCK_SIZE];

#[allow(unused)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct ShortDirEntry {
    // 文件名，若在使用中0x0位置为文件名第一个字符，若未使用则为0x00，若曾经被使用过但是现在已经被删除则为0xe5，有多余可以用0x20填充
    file_name: [u8;8],
    // 扩展名,如果是子目录，则将扩展名部分用“0x20”进行填充 
    extension_name: [u8;3],
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

pub fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

impl ShortDirEntry {
    // 新建文件时时间暂时先通通置0，以后再实现时间
    pub fn new(file_name: &[u8], extension_name: &[u8], flag: u8) -> Self{
        let name:[u8;8] = clone_into_array(&file_name[0..8]);
        let extension_name:[u8;3] = clone_into_array(&extension_name[0..3]);
        Self {
            file_name: name,
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

    #[allow(unused)]
    pub fn empty() -> Self{
        Self {
            file_name: [0;8],
            extension_name: [0;3],
            flag: 0,
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

    #[allow(unused)]
    pub fn get_flag(&self) -> u8 {
        self.flag
    }

    #[allow(unused)]
    //和时间相关的方法暂时通通不实现
    pub fn get_creation_time(&self) {

    }

    #[allow(unused)]
    pub fn get_last_modify_time(&self) {
        
    }

    #[allow(unused)]
    pub fn get_last_access_time(&self) {

    }

    #[allow(unused)]
    // 获取文件名，不包括扩展名，caller调用前必须保证该文件没有被删除，否则返回的文件名不一定正确
    pub fn get_file_name(&self) -> String {
        let mut name = String::new();
        for c in self.file_name {
            if c == 0x20 {
                break;
            }
            else {
                name.push(c as char);
            }
        }
        name
    }

    #[allow(unused)]
    // 获取文件扩展名，caller调用前必须保证该文件有文件扩展名，否则返回扩展名可能是空
    pub fn get_extension_name(&self) -> String {
        let mut name = String::new();
        for c in self.extension_name {
            if c == 0x20 {
                break;
            }
            else {
                name.push(c as char);
            }
        }
        name
    }

    #[allow(unused)]
    // 获取文件名
    pub fn get_name(&self) -> String{
        let mut name = String::new();
        for c in self.file_name {
            if c == 0x20 {
                break;
            }
            else {
                name.push(c as char);
            }
        }
        if self.extension_name[0] != 0x20 {
            name.push('.');
        }
        for c in self.extension_name {
            if c == 0x20 {
                break;
            }
            else {
                name.push(c as char);
            }
        }
        name
    }

    #[allow(unused)]
    // 判断文件是否被删除
    pub fn is_delete(&self) -> bool {
        self.file_name[0] == 0xe5
    }

    #[allow(unused)]
    pub fn set_delete(&mut self) {
        self.file_name[0] = 0xe5
    }

    #[allow(unused)]
    // 判断目录项是否为空
    pub fn is_empty(&self) -> bool {
        self.file_name[0] ==0x00
    }


    #[allow(unused)]
    // 判断是否是长目录项
    pub fn is_long_dir(&self) -> bool {
        self.flag == LONG_DIR_ENTRY
    }

    #[allow(unused)]
    // 设置文件的起始簇
    pub fn set_start_cluster(&mut self, start: u32) {
        self.start_cluster_low = (start & 0x0000ffff) as u16;
        self.start_cluster_high = (start >>16) as u16;
    }

    #[allow(unused)]
    // 获取文件起始簇
    pub fn get_start_cluster(&self) -> u32 {
        let mut high: u32 = (self.start_cluster_high) as u32;
        high = high << 16;
        high + self.start_cluster_low as u32
    }

    #[allow(unused)]
    // 判断是否是目录项
    pub fn is_dir(&self) -> bool {
        self.flag & SUB_DIRECTORY == SUB_DIRECTORY
    }

    #[allow(unused)]
    // 设置文件长度，单位字节
    pub fn set_file_length(&mut self, file_length: u32) {
        self.file_length = file_length;
    }


    #[allow(unused)]
    // 设置文件长度，单位字节
    pub fn get_file_length(&self) -> u32 {
        self.file_length
    }

    #[allow(unused)]
    // 获取文件所占簇数
    pub fn get_cluster_num(&self, bytes_per_cluster: u32) -> u32 {
        let mut res: u32 = 0;
        res = (self.file_length + bytes_per_cluster -1) / bytes_per_cluster;
        res
    }

    #[allow(unused)]
    // 获取该文件偏移量所在簇号,簇内偏移量块号,块内偏移量,注意,该方法特别单纯,不会进行越界检查,caller必须负责,否则可能导致结果错误!!!
    pub fn get_offset_position(&self, offset: u32 ,sector_per_cluster: u32, dev: u8, fat: &Arc<RwLock<FAT>>,) -> (u32, u32, u32) {
        let cluster_num = offset / (sector_per_cluster * BLOCK_SIZE as u32);
        let cluster_offset = offset % (sector_per_cluster * BLOCK_SIZE as u32);
        let sector_in_cluster: u32 = cluster_offset / BLOCK_SIZE as u32;
        let byte_offset: u32 = cluster_offset % BLOCK_SIZE as u32;
        let mut current: u32 = self.get_start_cluster();
        
        let fat_read = fat.read();
        for i in (0..cluster_num) {
            let mut next = fat_read.get_next_cluster(current ,dev);
            current = next;
        }
        (current, sector_in_cluster, byte_offset)
    }

    #[allow(unused)]
    // 重头戏1,从该文件偏移量为offset开始处读取长度为buffer数组长度的数据到buffer数组
    pub fn read_at(&self, offset: u32, buffer: &mut [u8], dev :u8, fat: &Arc<RwLock<FAT>>, fat32_manager: &Arc<RwLock<Fat32Manager>>) -> u32{
        // 1.前期准备工作
        
        let fat_read = fat.read();
        let fat32_manager_read = fat32_manager.read();
        let bytes_per_sector = fat32_manager_read.get_bytes_per_sector();
        let sectors_per_cluster = fat32_manager_read.get_sectors_per_cluster();
        let first_sector = fat32_manager_read.get_first_sector();
        let total_read_length = buffer.len() as u32;
        let mut need_read_length = buffer.len() as u32;
        let mut have_read_length: u32 = 0;
        let mut file_length: u32;
        // 由于目录项file_length字段置0,所以需手动计算其大小
        if self.is_dir() {
            file_length = fat_read.get_cluster_num(self.get_start_cluster(), dev) * bytes_per_sector as u32 * sectors_per_cluster as u32;
        }
        else {
            file_length = self.file_length;
        }

        // 2.越界判断,若越界则返回读取到0字节
        if total_read_length + offset > file_length {
            return 0;
        }

        // 3.找到offset所在的簇号,簇内块号,块内字节偏移,确定块内开始字节与最后读取块内的结束字节,当前读取的簇号,当前读取的簇内块号
        let (current, sector_in_cluster, byte_offset) = self.get_offset_position(offset,sectors_per_cluster as u32, dev, &fat32_manager_read.get_fat());
        let mut sector_start = byte_offset;
        let mut sector_end: u32 = (BLOCK_SIZE as u32).min(byte_offset + need_read_length - have_read_length);
        let mut current_cluster = current;
        let mut current_sector = sector_in_cluster;
        // 4.重点!从第一个要读的簇内的偏移簇内块号开始读,直到need_read_length为0,有极大可能会读好几个簇,
        'counting_up: loop {
            while current_sector < sectors_per_cluster as u32{
                
                let dst = &mut buffer[have_read_length as usize..(have_read_length + sector_end - sector_start) as usize];
                if self.is_dir() {
                    get_info_buffer(
                        first_sector + (current_cluster - 2) * sectors_per_cluster as u32 + current_sector,
                        dev,
                    )
                    .read()
                    .read(0, |data_block: &DataBlock| {
                        let src = &data_block[sector_start as usize..sector_end as usize];
                        dst.copy_from_slice(src);
                    });
                    
                    have_read_length += sector_end - sector_start;
                    need_read_length -= sector_end - sector_start;
                    sector_start = 0;
                    sector_end = (BLOCK_SIZE as u32).min(total_read_length - have_read_length);
                    current_sector += 1;
                    
                } 
                else {
                    get_data_block_buffer(
                        first_sector + current_cluster * sectors_per_cluster as u32 + current_sector,
                        dev,
                    )
                    .read()
                    .read(0, |data_block: &DataBlock| {
                        let src = &data_block[sector_start as usize..sector_end as usize];
                        dst.copy_from_slice(src);
                    });
                    have_read_length += sector_end - sector_start;
                    need_read_length -= sector_end - sector_start;
                    sector_start = 0;
                    sector_end = (BLOCK_SIZE as u32).min(total_read_length - have_read_length);
                    current_sector += 1;
                }
                if need_read_length == 0 {
                    break 'counting_up;
                }
            }

            current_cluster = fat_read.get_next_cluster(current_cluster, dev);
            current_sector = 0;

        }
        have_read_length
    }


    #[allow(unused)]
    pub fn get_dir_length(&self, dev: u8, fat: &Arc<RwLock<FAT>>, fat32_manager: &Arc<RwLock<Fat32Manager>>) -> u32 {
        
        let fat32_manager_read = fat32_manager.read();
        let sector_per_cluster = fat32_manager_read.get_sectors_per_cluster();
        let bytes_per_sector = fat32_manager_read.get_bytes_per_sector();
        sector_per_cluster as u32 * bytes_per_sector as u32 * fat.read().get_cluster_num(self.get_start_cluster(),DEV)
    }

    #[allow(unused)]
    // 切记，使用增长目录大小时length长度恒为0，故只能获取簇个数来获取目录大小
    pub fn increase(&mut self, need_bytes: u32, dev :u8, fat: &Arc<RwLock<FAT>>, fat32_manager: &Arc<RwLock<Fat32Manager>>) {
        
        let fat32_manager_read = fat32_manager.read();
        let bytes_per_sector = fat32_manager_read.get_bytes_per_sector() as u32;
        let sectors_per_cluster = fat32_manager_read.get_sectors_per_cluster() as u32;
        let first = fat32_manager_read.alloc_cluster(((need_bytes + bytes_per_sector * sectors_per_cluster - 1) / (bytes_per_sector * sectors_per_cluster)) as usize, dev).unwrap();
        
        let fat_read = fat.read();
        //初始化时是0
        
        if self.get_start_cluster() == 0 {
            self.set_start_cluster(first);
        }
        else {
            let last_cluster = fat_read.get_file_last_cluster(self.get_start_cluster(), dev);
            println!("last {}",last_cluster);
            fat_read.set_next_cluster(last_cluster, first, dev);
        }
        if !self.is_dir() {
            self.file_length += need_bytes;
        }
    }

    #[allow(unused)]
    // 重头戏2,将buffer数组内容写入该文件偏移量为offset处,注意，此方法必须在调用之前进行越界检查，切记一定！！！
    pub fn write_at(&self, offset: u32, buffer: & [u8], dev :u8, fat: &Arc<RwLock<FAT>>, fat32_manager: &Arc<RwLock<Fat32Manager>>) -> u32{
        // 1.前期准备工作
      
        let fat_read = fat.read();
        
        let fat32_manager_read = fat32_manager.read();
        
        let bytes_per_sector = fat32_manager_read.get_bytes_per_sector() as u32;
        let sectors_per_cluster = fat32_manager_read.get_sectors_per_cluster() as u32;
        let first_sector = fat32_manager_read.get_first_sector();
        let total_write_length = buffer.len() as u32;
        let mut need_write_length = buffer.len() as u32;
        let mut have_write_length = 0;
        let mut file_length: u32;
        
        // 由于目录项file_length字段置0,所以需手动计算其大小
        if self.is_dir() {
            file_length = fat_read.get_cluster_num(self.get_start_cluster(), dev) * bytes_per_sector * sectors_per_cluster;
            
        }
        else {
            file_length = self.file_length;
            
        }

        // 2.越界判断,若要写的长度加偏移量大于文件长度,则返回0
        if total_write_length + offset > file_length {
            return 0;
        }
        
        // 3.找到offset所在的簇号,簇内块号,块内字节偏移,确定块内开始字节与最后写入块内的结束字节,当前写入的簇号,当前写入的簇内块号
        let (current, sector_in_cluster, byte_offset) = self.get_offset_position(offset,sectors_per_cluster, dev, &fat32_manager_read.get_fat(),);
        let mut sector_start = byte_offset;
        let mut sector_end = (BLOCK_SIZE as u32).min(byte_offset + total_write_length - have_write_length);
        let mut current_cluster = current;
        let mut current_sector = sector_in_cluster;

        // 4.重点!从第一个要写入的簇内的偏移簇内块号开始写入,直到need_read_length为0,有极大可能会写入好几个簇,
        
        'counting_up: loop {
            while current_sector < sectors_per_cluster{
                let src = &buffer[have_write_length as usize..(have_write_length + sector_end - sector_start) as usize];
                if self.is_dir() {
                    get_info_buffer(
                        first_sector + (current_cluster - 2) * sectors_per_cluster + current_sector,
                        dev,
                    )
                    .write()
                    .modify(0, |data_block: &mut DataBlock| {
                        let dst = &mut data_block[sector_start as usize..sector_end as usize];
                        dst.copy_from_slice(src);
                    });
                    have_write_length += sector_end - sector_start;
                    need_write_length -= sector_end - sector_start;
                    sector_start = 0;
                    sector_end = (BLOCK_SIZE as u32).min(total_write_length - have_write_length);
                    current_sector += 1;
                    
                } 
                else {
                    get_data_block_buffer(
                        first_sector + current_cluster * sectors_per_cluster + current_sector,
                        dev,
                    )
                    .write()
                    .modify(0, |data_block: &mut DataBlock| {
                        let dst = &mut data_block[sector_start as usize..sector_end as usize];
                        dst.copy_from_slice(src);
                    });
                    have_write_length += sector_end - sector_start;
                    need_write_length -= sector_end - sector_start;
                    sector_start = 0;
                    sector_end = (BLOCK_SIZE as u32).min(total_write_length - have_write_length);
                    current_sector += 1;
                }
                if need_write_length == 0 {
                    break 'counting_up;
                }
            }

            current_cluster = fat_read.get_next_cluster(current_cluster, dev);
            current_sector = 0;

        }
        have_write_length
    }

    #[allow(unused)]
    pub fn trans_to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as usize as *const u8,
                32,
            )
        }
    }

    #[allow(unused)]
    pub fn trans_to_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut _ as usize as *mut u8,
                32,
            )
        }
    }

    #[allow(unused)]
    pub fn get_check_sum(&self) -> u8 {
        let mut check_sum: u16 = 0;

        for i in (0..8) {
            if (check_sum & 1) != 0 {
                check_sum = 0x80 + (check_sum >> 1) + self.file_name[i] as u16;
            }
            else {
                check_sum = 0 + (check_sum >> 1) + self.file_name[i] as u16;
            }
        }
        for i in (8..11) {
            if (check_sum & 1) != 0 {
                check_sum = 0x80 + (check_sum >> 1) + self.extension_name[i - 8] as u16;
            }
            else {
                check_sum = 0 + (check_sum >> 1) + self.extension_name[i - 8] as u16;
            }
        }
        check_sum as u8
    }


    #[allow(unused)]
    pub fn find_next_free_dirent(&mut self, dev: u8, fat: &Arc<RwLock<FAT>>, fat32_manager: &Arc<RwLock<Fat32Manager>>) -> Option<u32> {
        
        if !self.is_dir() {
            println!("can not find dirent entry in no-dirent file");
            return None
        }
        let mut offset: u32 = 0;
        let mut short_dir_entry = ShortDirEntry::empty();
        let mut read_length = 0;
        
        read_length = self.read_at(offset, short_dir_entry.trans_to_mut_bytes(), dev, fat, fat32_manager,);
        
        loop {
            
            if read_length == 0 {
                // 读了个寂寞，证明目录大小为0或者给读到底都没有空闲目录项，赶紧给分配一个簇
                if let Some(cluster) = fat32_manager.write().alloc_cluster(1, dev) {
                    // 假如你的容量为0
                    let fat_write = fat.write();
                    let fat32_manager_read = fat32_manager.read();
                    if self.get_start_cluster() == 0 {
                        self.set_start_cluster(cluster);
                    }
                    else {
                        fat_write.set_next_cluster(fat_write.get_file_last_cluster(self.get_start_cluster(), dev), cluster, dev);
                    }
                    let cluster_num = fat_write.get_cluster_num(self.get_start_cluster(), dev) as u32;
    
                    //返回值，仔细研究
                    return Some((cluster_num - 1) * fat32_manager_read.get_bytes_per_sector() as u32* fat32_manager_read.get_sectors_per_cluster() as u32)
                }
                else{
                    println!("No free cluster!!!");
                    return None
                }
            }
            else {
                
                if !short_dir_entry.is_empty() {
                    offset += 32;
                    read_length = self.read_at(offset, short_dir_entry.trans_to_mut_bytes(), dev, fat, fat32_manager,);
                }
                else {
                    
                    return Some(offset)
                }
            }
        }
        
    }

    
 
}

#[allow(unused)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct LongDirEntry {
    // 长文件名目录项序列号,从1开始,若是最后一个长文件名目录项,则将其序号|=0x40,若被删除,则设置为0xe5
    flag: u8,
    // 长文件名的1~5个字符,使用unicode码若结束但还有未使用的字节,则先填充2字节00,再填充0xff
    name1: [u8;10],
    // 长目录项属性标志,一定是0x0f
    dir_flag: u8,
    // 保留
    reserved: u8,
    // 校验和,若应一个长文件名需要几个目录项存储,则它们具有相同校验和
    check_sum: u8,
    // 长文件名6~11个字符,未使用的字节用0xff填充
    pub name2: [u8;12],
    // 保留
    start_cluster: u16,
    // 长文件名12~13个字符,未使用的字节用0xff填充
    name3: [u8;4],
}

impl LongDirEntry {
    #[allow(unused)]
    pub fn new(flag: u8, name: &[u8], check_sum: u8) -> Self {
        let mut name1: [u8;10] = [0;10];
        let mut name2: [u8;12] = [0;12];
        let mut name3: [u8;4] = [0;4];
        let mut flags = true;
        for i in (0..5) {
            if flags {
                if name[i] == 0 {
                    flags = false;
                }
                name1[i<<1] = name[i];
            }
            else{
                name1[i<<1] = 0xFF;
                name1[(i<<1)+1] = 0xFF;
            }
            
        }
        for i in (5..11) {
            if flags {
                if name[i] == 0 {
                    flags = false;
                }
                name2[(i - 5)<<1] = name[i];
            }
            else{
                name2[(i - 5)<<1] = 0xFF;
                name2[((i - 5)<<1)+1] = 0xFF;
            }
        }
        for i in (11..13) {
            if flags {
                if name[i] == 0 {
                    flags = false;
                }
                name3[(i - 11)<<1] = name[i];
            }
            else{
                name3[(i - 11)<<1] = 0xFF;
                name3[((i - 11)<<1)+1] = 0xFF;
            }
        }
        // 暂时置0，后续修改
        Self {
            flag,
            name1,
            dir_flag: 0,
            reserved: 0,
            check_sum,
            name2,
            start_cluster: 0,
            name3,
        }
    }

    #[allow(unused)]
    pub fn empty() -> Self {
        Self{
            flag: 0,      
            name1: [0;10], 
            dir_flag: 0,  
            reserved: 0,       
            check_sum: 0,
            name2: [0;12],  
            start_cluster: 0, 
            name3: [0;4],  
        }
    }

    #[allow(unused)]
    // 判断文件是否被删除
    pub fn is_delete(&self) -> bool {
        self.flag == 0xe5
    }

    #[allow(unused)]
    pub fn is_long_dir(&self) -> bool {
        self.flag == LONG_DIR_ENTRY
    }

    #[allow(unused)]
    pub fn set_delete(&mut self) {
        self.flag = 0xe5
    }

    #[allow(unused)]
    // 判断目录项是否为空
    pub fn is_empty(&self) -> bool {
        self.flag ==0b00
    }

    #[allow(unused)]
    pub fn is_last(&self) -> bool {
        self.flag & 0b01000000 == 1
    }

    #[allow(unused)]
    pub fn set_last(&mut self) {
        self.flag |= 0b01000000
    }

    #[allow(unused)]
    pub fn get_serial(&self) -> u8 {
        self.flag & 0b00011111
    }

    #[allow(unused)]
    pub fn get_checksum(&self) -> u8 {
        self.check_sum
    }

    #[allow(unused)]
    pub fn get_name(&self) -> String {
        let mut name = String::new();
        for i in (0..5) {
            if self.name1[i<<1] == 0 {
                return name;
            }
            else {
                name.push(self.name1[i<<1] as char);
            }
        }
        for i in (5..11) {
            if self.name2[(i - 5)<<1] == 0 {
                return name;
            }
            else {
                name.push(self.name2[(i - 5)<<1] as char);
            }
        }
        for i in (11..13) {
            if self.name3[(i - 11)<<1] == 0 {
                return name;
            }
            else {
                name.push(self.name3[(i - 11)<<1] as char);
            }
        }
        name
    }

    #[allow(unused)]
    pub fn trans_to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as usize as *const u8,
                32,
            )
        }
    }

    #[allow(unused)]
    pub fn trans_to_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut _ as usize as *mut u8,
                32,
            )
        }
    }
}


