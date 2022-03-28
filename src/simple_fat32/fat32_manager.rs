use alloc::sync::Arc;
use lazy_static::*;
use spin::RwLock;


use super::{
    set_start_sector,
    //未加入
    println,
    get_info_buffer,
    FsInfo,
    ShortDirEntry,
    LongDirEntry,
    BPB,
    Buffer,
    BufferManager,
}

const BLOCK_SIZE: u32 = 512;
const FAT32_ENTRY_SIZE: u32 = 4;
const FAT_ENTRY_PER_SECTOR: u32 = BLOCK_SIZE / FAT32_ENTRY_SIZE;
const LAST_CLUSTER: u32 = 0x0fffffff;
const FREE_CLUSTER_ENTRY: u32 = 0x00000000;


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

    pub fn open_fat32(dev: u8) -> Arc<RwLock<self>> {
        //从硬盘物理位置为0地方开始读，前512字节为主引导扇区mbr，mbr占用前446字节，另外64字节交给DPT，最后2字节55，aa是分区结束标志
        println!("start loading FAT32!");
        //读取相对扇区数,位于mbr扇区偏移量0x0176处，4字节
        let start_sec = get_info_buffer(
            0,
            dev,
        ).read().read(0x0176, |start_sector_arr: &[u8,4]| {
            //小端序转换成大端序！！！
            let mut i = 0;
            let mut start_sector = 0;
            while i < 4 {
                let mut add = start_sector_arr[i] as u32;
                let add = add<<(8*i);
                start_sector += add;
            }
            start_sector
        })

        set_start_sector(start_sec as usize);

        // 下面读取dbr所在扇区数，读取bpb数据结构！！！此扇区距0号扇区偏移量为n，其中n值即上面的start_sec!!!
        let bpb = get_info_buffer(
            0,
            dev,
        ).read().read(0, |b: &BPB| {
            //小端序转换成大端序！！！
            *b
        })

        // 从bpb中提取出重要字段

        let bytes_per_sector: u16 = bpb.get_bytes_per_sector();
        let sectors_per_cluster: u8 = bpb.get_sectors_per_cluster();
        let total_sector: u32 = bpb.get_total_sector();
        

        let fsinfo_sector_num: u32 = bpb.get_fsinfo_sector_num();

        // 读取fsinfo数据结构！！！
        let fsinfo = get_info_buffer(
            fsinfo_sector_num,
            dev,
        ).read().read(0, |fsi: &FsInfo| {
            //小端序转换成大端序！！！
            *fsi
        })

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
        let sectors_per_fat: u16 = bpb.get_sectors_per_fat();
        let fat_num = bpb.get_fat_num();

        // FAT数据结构！！！
        let fat = FAT::new(reserved_sector_num, sectors_per_fat + reserved_sector_num ,);

        let first_sector: u32 = reserved_sector_num + sectors_per_fat * fat_num + sectors_per_cluster * 2;
        // 注意，先写1，等下实现
        let mut root_dir = ShortDirEntry::new(&[0x2F,0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20],
                                              &[0x20, 0x20, 0x20],
                                              1,);
        
        let fat32_manager = self {
            bytes_per_sector,
            sectors_per_cluster,
            total_sector,
            root_dir: Arc::new(RwLock::new(root_dir)),
            fsinfo: Arc::new(RwLock::new(fsinfo)),
            dev,
            fat: Arc::new(RwLock::new(fat)),
            first_sector,
        }
                    

    }

    pub fn alloc_cluster(&self ,need :usize, dev: u8) -> Option<u32>{
        let fsinfo = self.fsinfo.write();
        if fsinfo.get_free_cluster_num() < need {
            println!("the need is more than free clusters!!!");
            None
        }
        else {
            let fat = self.fat.write();
            let res = fsinfo.get_first_free_cluster();
            let mut current: u32 = fsinfo.get_first_free_cluster();
            self.clean_cluster(current, dev);
            let mut i = 1;
            while i < need {
                let mut next: u32 = fat.get_next_free_cluster(current, dev);
                fat.set_next_cluster(current, next, dev);
                current = next;
                self.clean_cluster(current, dev);
                i += 1;
            }
            fat.set_next_cluster(current, LAST_CLUSTER, dev);
            // 分配完之后修改fsinfo信息
            fsinfo.set_free_cluster_num(get_free_cluster_num() - need);
            fsinfo.set_first_free_cluster(fat.get_next_free_cluster(current, dev););
            Some(res);
        }
    }
    // 注意，调用此方法是从传入的start参数开始依次释放所有后面的簇，故一般要释放簇时只有当要删除整个文件时，暂时不支持在不删除文件的情况下动态减小该文件大小
    pub fn dealloc_cluster(&self ,start :u32, dev: u8) {
        let fsinfo = self.fsinfo.write();
        let fat = self.fat.write();
        let current: u32 = fsinfo.get_first_free_cluster();
        if start < current {
            fsinfo.set_first_free_cluster(start);
        }
        loop {
            let mut next = fat.get_next_cluster(start, dev);
            fat.set_next_cluster(start, FREE_CLUSTER_ENTRY, dev);
            start = next;
            if start == LAST_CLUSTER;
            break;
        }
    }

    pub fn clean_cluster(&self, current_cluster: u32, dev: u8) {
        let start_sector : u32 = self.first_sector + current_cluster * self.sectors_per_cluster;
        let mut i = 0;
        while i < self.sectors_per_cluster {
            get_data_block_buffer(
                i + start_sector,
                dev,
            ).write().modify(0, |block: &[u8;BLOCK_SIZE]| {
                for j in 0..512 {
                    block[j] = 0;
                }  
            })
        }
    }

    
}