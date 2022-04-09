use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;


use super::{
    BLOCK_SIZE,
    DEV,
    get_info_buffer,
    get_data_block_buffer,
    block_cache_sync_all,
    set_start_sector,
    fsinfo::FsInfo, 
    bpb::BPB, 
    fat::FAT,
    dir_entry::ShortDirEntry,
    vfs::VFSFile,
    // println,
};



const FAT32_ENTRY_SIZE: u32 = 4;
const FAT_ENTRY_PER_SECTOR: u32 = BLOCK_SIZE as u32 / FAT32_ENTRY_SIZE;
const LAST_CLUSTER: u32 = 0x0fffffff;
const FREE_CLUSTER_ENTRY: u32 = 0x00000000;
const SUB_DIRECTORY: u8 = 0b00010000;

pub struct Fat32Manager {
    // 每扇区字节数
    bytes_per_sector: u16,
    // 每簇扇区数
    sectors_per_cluster: u8,
    // 文件系统总扇区数
    total_sector: u32,
    // 根目录
    root_dir: Arc<RwLock<ShortDirEntry>>,
    // fsinfo(文件系统信息扇区),为操作系统提供关于空簇总数及下一个可用簇信息
    fsinfo: Arc<RwLock<FsInfo>>,
    // 设备号,待实现
    dev: u8,
    //fat
    fat:Arc<RwLock<FAT>>,
    //第一簇（前两个簇不被使用！！！）的第一个块的块号
    first_sector: u32,
}

impl Fat32Manager {

    pub fn open_fat32(dev: u8) -> Arc<RwLock<Self>> {
        //从硬盘物理位置为0地方开始读，前512字节为主引导扇区mbr，mbr占用前446字节，另外64字节交给DPT，最后2字节55，aa是分区结束标志
        println!("start loading FAT32!");
        //读取相对扇区数,位于mbr扇区偏移量0x0176处，4字节
        let start_sec = get_info_buffer(
            0,
            dev,
        ).read().read(0x0176, |start_sector_arr: &[u8;4]| {
            //小端序转换成大端序！！！
            let mut i = 0;
            let mut start_sector = 0;
            while i < 4 {
                let mut add = start_sector_arr[i] as u32;
                let add = add<<(8*i);
                start_sector += add;
                i += 1;
            }
            start_sector
        });
        
        set_start_sector(start_sec as u32);

        // 下面读取dbr所在扇区数，读取bpb数据结构！！！此扇区距0号扇区偏移量为n，其中n值即上面的start_sec!!!
        let bpb = get_info_buffer(
            0,
            dev,
        ).read().read(0, |b: &BPB| {
            //小端序转换成大端序！！！
            *b
        });

        // 从bpb中提取出重要字段

        let bytes_per_sector: u16 = bpb.get_bytes_per_sector();
        println!("bytes_per_sector {}",bytes_per_sector);
        let sectors_per_cluster: u8 = bpb.get_sectors_per_cluster();
        println!("sectors_per_cluster {}",sectors_per_cluster);
        let total_sector: u32 = bpb.get_total_sector();
        println!("total_sector {}",total_sector);
        

        let fsinfo_sector_num: u16 = bpb.get_fsinfo_sector_num();
        println!("fsinfo_sector_num {}",fsinfo_sector_num);

        // 读取fsinfo数据结构！！！
        let fsinfo = get_info_buffer(
            fsinfo_sector_num as u32,
            dev,
        ).read().read(0, |fsi: &FsInfo| {
            *fsi
        });

        if fsinfo.check_lead_sig() == false {
            // 可能待实现
            // assert!(fsinfo.check_lead_sig())
            panic!("lead_sig of fsinfo is {},it should be 0x41625252!!!",fsinfo.get_lead_sig());
        }

        if fsinfo.check_struct_sig() == false {
            panic!("struct_sig of fsinfo is {},it should be 0x61417272!!!",fsinfo.get_struct_sig());
        }

        if fsinfo.check_trail_sig() == false {
            panic!("trail_sig of fsinfo is {},it should be 0xaa550000!!!",fsinfo.get_trail_sig());
        }

        
        let reserved_sector_num: u16 = bpb.get_reserved_sector_num();
        println!("reserved_sector_num {}",reserved_sector_num);
        let sectors_per_fat: u32 = bpb.get_sectors_per_fat();
        println!("sectors_per_fat {}",sectors_per_fat);
        let fat_num = bpb.get_fat_num();
        println!("fat_num {}",fat_num);

        // FAT数据结构！！！这里默认fat有2个
        let fat = FAT::new(reserved_sector_num as u32, sectors_per_fat + reserved_sector_num as u32,);

        let first_sector: u32 = reserved_sector_num as u32 + sectors_per_fat * fat_num as u32;
        // 注意，先写1，等下实现
        // ox2f（47）是/的ascii码，用这个来命名根目录
        let mut root_dir = ShortDirEntry::new(&[0x2F,0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20],
                                              &[0x20, 0x20, 0x20],
                                              0b00010000,);
        // let mut root_dir = ShortDirEntry::new(&[0x46,0x41, 0x54, 0x33, 0x32, 0x20, 0x20, 0x20],
        //     &[0x74, 0x78, 0x74],
        //     0b00010000,);
        
        
        let fat32_manager = Self {
            bytes_per_sector,
            sectors_per_cluster,
            total_sector,
            root_dir: Arc::new(RwLock::new(root_dir)),
            fsinfo: Arc::new(RwLock::new(fsinfo)),
            dev,
            fat: Arc::new(RwLock::new(fat)),
            first_sector,
        };

        Arc::new(RwLock::new(fat32_manager))
                    
    }

    pub fn get_root_vfsfile(&self, fs: Arc<RwLock<Fat32Manager>>,) -> VFSFile {
        VFSFile::new(
            DEV, 
            fs, 
            0, 
            0, 
            Vec::new(), 
            String::from("/"), 
            SUB_DIRECTORY,)
    }

    pub fn get_root(&self) -> Arc<RwLock<ShortDirEntry>> {
        self.root_dir.clone()
    }



    // 分配need个簇，会进行越界检查
    pub fn alloc_cluster(&self ,need :usize, dev: u8) -> Option<u32>{
        let mut fsinfo_write = self.fsinfo.write();
        if fsinfo_write.get_free_cluster_num() < need as u32{
            println!("the need is more than free clusters!!!");
            None
        }
        else {  
            let mut fat_write = self.fat.write();
            
            let res = fsinfo_write.get_first_free_cluster();
            let mut current: u32 = fsinfo_write.get_first_free_cluster();
            // 使用之前必须清空
            self.clean_cluster(current, dev);
            let mut i = 1;
            
            while i < need {
                let mut next: u32 = fat_write.get_next_free_cluster(current, dev);
                fat_write.set_next_cluster(current, next, dev);
                current = next;
                self.clean_cluster(current, dev);
                i += 1;
            }
            fat_write.set_next_cluster(current, LAST_CLUSTER, dev);
            
            // 分配完之后修改fsinfo信息
            let free_cluster_num = fsinfo_write.get_free_cluster_num();
            
            fsinfo_write.set_free_cluster_num(free_cluster_num - need as u32);
            let next_free_cluster = fat_write.get_next_free_cluster(current, dev);
            
            fsinfo_write.set_first_free_cluster(next_free_cluster);
            
            Some(res)
        }
    }

    // 注意，调用此方法是从传入的start参数开始依次释放所有后面的簇，故一般要释放簇时只有当要删除整个文件时，暂时不支持在不删除文件的情况下动态减小该文件大小
    pub fn dealloc_cluster(&self ,start :u32, dev: u8) {
        let mut start = start;
        let mut fsinfo_write = self.fsinfo.write();
        let fat_write = self.fat.write();
        let current: u32 = fsinfo_write.get_first_free_cluster();
        if start < current {
            fsinfo_write.set_first_free_cluster(start);
        }
        loop {
            let next = fat_write.get_next_cluster(start, dev);
            fat_write.set_next_cluster(start, FREE_CLUSTER_ENTRY, dev);
            start = next;
            if start == LAST_CLUSTER {
                break;
            }
            
        }
    }

    pub fn clean_cluster(&self, current_cluster: u32, dev: u8) {
        let start_sector : u32 = self.first_sector + (current_cluster - 2) * self.sectors_per_cluster as u32;
        let mut i: u32 = 0;
        while i < self.sectors_per_cluster as u32 {
            get_data_block_buffer(
                i + start_sector,
                dev,
            ).write().modify(0, |block: &mut [u8;BLOCK_SIZE]| {
                for j in 0..512 {
                    block[j] = 0;
                }  
            });
            i += 1;
        }
    }

    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    pub fn get_sectors_per_cluster(&self) -> u8 {
        self.sectors_per_cluster
    }

    pub fn get_fat(&self) -> Arc<RwLock<FAT>> {
        Arc::clone(&self.fat)
    }

    pub fn get_fsinfo(&self) -> Arc<RwLock<FsInfo>> {
        Arc::clone(&self.fsinfo)
    }

    pub fn get_first_sector(&self) -> u32 {
        self.first_sector
    }

    pub fn split_long_name(long_name :&str) -> Vec<String> {
        let name_byte = long_name.as_bytes();
        let mut name_vec:Vec<String> = Vec::new();
        let length = (long_name.len() + 12) / 13;
        for i in (0..length) {
            let mut name = String::new();
            for j in (i * 13..(i + 1) * 13) {
                if j < long_name.len() {
                    name.push(name_byte[j as usize] as char)
                }
                else if j == long_name.len() {
                    name.push(0x00 as char)
                }
                else {
                    name.push(0xff as char)
                }
                
            }
            name_vec.push(name.clone());
        }
        name_vec
    }

    pub fn split_name_extension<'a>(name: &'a str) -> (&'a str, &'a str) {
        let mut name_extension: Vec<&str> = name.split(".").collect();
        let file_name = name_extension[0];
        if name_extension.len() == 1 {
            name_extension.push("");
        } 
        let extension_name = name_extension[1];
        (file_name, extension_name)

    }

    //此方法需要更改，返回值为文件全名
    pub fn long_name_to_short(long_name :&str) -> String {
        // 取长文件名的前6个字符加上”~1”形成短文件名，扩展名不变。若一旦产生同名，后续处理暂时不实现
        let mut file_name = String::new();
        let (file, extension) = Fat32Manager::split_name_extension(long_name);
        let file_byte_arr = file.as_bytes();
        let extension_byte_arr = extension.as_bytes();
        for i in (0..6) {
            file_name.push(file_byte_arr[i] as char)
        }
        file_name.push('~');
        file_name.push('1');
        if extension_byte_arr.len() == 0 {
            file_name
        }
        else {
            file_name.push('.');
            for i in (0..3) {
                if i > extension_byte_arr.len() {
                    file_name.push(0x20 as char);
                }
                else {
                    file_name.push(extension_byte_arr[i] as char);
                }
            }
            file_name
        }
        
    }
    
    pub fn short_name_to_byte_arr(name: & str) -> ([u8;8], [u8;3]) {
        let (mut file_name, mut extension_name);
        if name == "." || name == ".." {
            file_name = name;
            extension_name = "";
        }
        else {
            (file_name, extension_name)= Fat32Manager::split_name_extension(name);
        }
        // println!("name {}",file_name);
        let file_name_arr = file_name.as_bytes();
        let extension_name_arr = extension_name.as_bytes();
        let mut file_arr: [u8;8] = [0;8];
        for i in 0..8 {
            if i < file_name_arr.len() {
                file_arr[i] = file_name_arr[i];
            }
            else {
                file_arr[i] = 0x20;
            }
        }
        
        let mut extension_arr: [u8;3] = [0;3];
        for i in 0..3 {
            if i < extension_name_arr.len() {
                extension_arr[i] = extension_name_arr[i];
            }
            else {
                extension_arr[i] = 0x20;
            }
        }

        return (file_arr, extension_arr);

    }
}