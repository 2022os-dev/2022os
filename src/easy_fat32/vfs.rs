use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;

use super::{
    //未加入
    // println,
    get_info_buffer,
    get_data_block_buffer,
    fat32_manager::*,
    fsinfo::*,
    fat::*,
    dir_entry::*,
    Buffer,
    BufferManager,
};

const LONG_DIR_ENTRY: u8 = 0b00001111;
const SUB_DIRECTORY: u8 = 0b00010000;

#[derive(Clone)]
pub struct VFSFile {
    // 设备号，待实现
    dev: u8,
    // 文件系统
    fs: Arc<RwLock<Fat32Manager>>,
    // 短目录项所在块号，注意，自动加上start_sec，见fat32_manager，根目录这个东西没用，恒0
    sector: u32,
    // 短目录项所在块内偏移量，单位字节，必须被32整除，根目录这个东西没用，恒0
    offset: u32,
    // 二元组向量，包含所有长目录项所在块号与块内偏移
    long_dir_location: Vec<(u32, u32)>,
    // 文件名
    name: String,
    flag: u8,
    // 目录项个数，非目录文件置0，有用？
    // dirent_num: u32,
}


impl VFSFile {

    pub fn new(
        dev: u8, 
        fs: Arc<RwLock<Fat32Manager>>, 
        sector: u32, 
        offset: u32, 
        long_dir_location: Vec<(u32, u32)>, 
        name: String, 
        flag: u8,) ->Self {
            Self {
                dev, 
                fs, 
                sector, 
                offset, 
                long_dir_location, 
                name, 
                flag,
            }
        }


    // 读取磁盘中短目录项
    fn read_ShortDirEntry<V>(&self, f: impl FnOnce(&ShortDirEntry) -> V) -> V {
        //如果是根目录，即self.sector==0，单独处理
        if self.sector == 0 {
            let root_dirent = self.fs.read().get_root();
            let root_read = root_dirent.read();
            f(& root_read)
        }
        else {
            get_info_buffer(
                self.sector,
                self.dev
            ).read().read(self.offset as usize, f)
        }
        
    }

    // 修改磁盘中短目录项
    fn modify_ShortDirEntry<V>(&self, f: impl FnOnce(&mut ShortDirEntry) -> V) -> V {
        //如果是根目录，即self.sector==0，单独处理
        if self.sector == 0 {
            let root_dirent = self.fs.read().get_root();
            let mut root_write = root_dirent.write();
            f(&mut root_write)
        }
        else {
            get_info_buffer(
                self.sector,
                self.dev
            ).write().modify(self.offset as usize, f)
        }
        
    }

    // 修改磁盘中长目录项，读取根本用不到，所以暂时不实现
    fn modify_LongDirEntry<V>(&self, idx: usize, f: impl FnOnce(&mut LongDirEntry) -> V) -> V {
        get_info_buffer(
            self.long_dir_location[idx].0,
            self.dev
        ).write().modify(self.long_dir_location[idx].1 as usize, f)
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    // 判断是否是长目录项
    pub fn is_long_dir(&self) -> bool {
        self.flag == LONG_DIR_ENTRY
    }

    // 判断是否是目录项
    pub fn is_dir(&self) -> bool {
        self.flag & SUB_DIRECTORY == SUB_DIRECTORY
    }

    //和时间相关的方法暂时通通不实现
    pub fn get_creation_time(&self) {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.get_creation_time()
        })
    }


    pub fn get_last_modify_time(&self) {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.get_last_modify_time()
        })
    }

    pub fn get_last_access_time(&self) {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.get_last_access_time()
        })
    }

    pub fn get_start_cluster(&self) -> u32 {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.get_start_cluster()
        })
    }

    pub fn set_start_cluster(&self, cluster: u32) {
        self.modify_ShortDirEntry(|sd: &mut ShortDirEntry| {
            sd.set_start_cluster(cluster)
        })
    }

    pub fn get_file_length(&self,) -> u32 {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.get_file_length()
        })
    }

    pub fn set_file_length(&self, length: u32) {
        self.modify_ShortDirEntry(|sd: &mut ShortDirEntry| {
            sd.set_file_length(length)
        })
    }

    pub fn get_dir_length(&self) -> u32 {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.get_dir_length(
                self.dev,
                &self.fs.read().get_fat(), 
                &self.fs,)
        })
    }


    pub fn is_delete(&self) -> bool {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            sd.is_delete()
        })
    }

    pub fn set_delete(&mut self) {
        self.modify_ShortDirEntry(|sd: &mut ShortDirEntry| {
            sd.set_delete()
        })
    }


    // 获取该偏移量所在的块号与块内偏移
    pub fn get_sector_offset(&self, offset: u32,) -> (u32, u32) {
        self.read_ShortDirEntry(|sd: &ShortDirEntry| {
            let sectors_per_cluster: u32 = self.fs.read().get_sectors_per_cluster() as u32;
            let (current, sector_in_cluster, byte_offset) = 
            sd.get_offset_position(offset, sectors_per_cluster, self.dev, &self.fs.read().get_fat(),);
            return (sectors_per_cluster * (current - 2) + self.fs.read().get_first_sector() + sector_in_cluster, byte_offset);
        })
    }

    pub fn find_next_free_dirent(&mut self) -> Option<u32> {
        self.modify_ShortDirEntry(|short_ent: &mut ShortDirEntry|{
            short_ent.find_next_free_dirent(
                self.dev,
                &self.fs.read().get_fat(), 
                &self.fs,
            )
        })
    }


    pub fn read_at(&self, offset: u32, buf: &mut [u8]) -> u32{
        self.read_ShortDirEntry(|short_ent: &ShortDirEntry|{
            short_ent.read_at(
                offset, 
                buf, 
                self.dev,
                &self.fs.read().get_fat(), 
                &self.fs,
            )
        })
    }   

    pub fn increase(&self, need_bytes: u32) {
        self.modify_ShortDirEntry(|short_ent: &mut ShortDirEntry|{
            short_ent.increase(
                need_bytes,
                self.dev,
                &self.fs.read().get_fat(),
                &self.fs,  
            )
        })
    }

    pub fn write_at(&self, offset: u32, buf: & [u8]) -> u32 {
        //注意，目录项文件大小恒0，用get_dir_length()判断
        if self.get_file_length() < offset + buf.len() as u32 && self.get_dir_length() < offset + buf.len() as u32{
            self.increase(offset + buf.len() as u32 - self.get_file_length());
        }

        self.modify_ShortDirEntry(|short_ent: &mut ShortDirEntry|{
            short_ent.write_at(
                offset, 
                buf, 
                self.dev,
                &self.fs.read().get_fat(),
                &self.fs,  
            )
        })
    }
    
    // 在dir目录中找名字为name的短目录项，可能找不到
    pub fn find_short_name(&self, name: &str, dir: &ShortDirEntry) -> Option<VFSFile> {
        let mut entry = ShortDirEntry::empty();
        let mut offset: u32 = 0;
        loop {
            let length = dir.read_at(offset, entry.trans_to_mut_bytes(), self.dev, &self.fs.read().get_fat(), &self.fs);
            // 假如你读了个寂寞
            if length == 0 {
                return None
            }
            // 若目录项为空，则表示已经到底，该目录中不存在名为name的目录项
            else if entry.is_empty() {
                return None
            }
            // 啊哈哈哈找到了
            else if !entry.is_delete() && entry.get_name() == name {
                let (sector, offset) = self.get_sector_offset(offset);
                return Some(
                    VFSFile::new(
                        self.dev, 
                        // 克隆！！！
                        self.fs.clone(), 
                        sector, 
                        offset, 
                        Vec::new(), 
                        String::from(name),
                        entry.get_flag(),)
                )
            }
            // 该目录项不是要找的，则寻找下一个
            else {
                offset += 32;
            }
        }
    }

    // 在dir目录中找名字为name的长目录项
    pub fn find_long_name(&self, name: &str, dir: &ShortDirEntry) -> Option<VFSFile> {
        let mut entry = ShortDirEntry::empty();
        let mut long_entry = LongDirEntry::empty();
        let convert_short_name = Fat32Manager::long_name_to_short(name);
        let mut offset: u32 = 0;
        loop {
            let length = dir.read_at(offset, entry.trans_to_mut_bytes(), self.dev, &self.fs.read().get_fat(), &self.fs);
            // 假如你读了个寂寞
            if length == 0 {
                return None
            }
            // 若目录项为空，则表示已经到底，该目录中不存在短文件名为convert_short_name的目录项
            else if entry.is_empty() {
                return None
            }
            // 如果找到对应短目录项，从下往上找出所有被分割的长目录项，检验其校验和，如果全部正确则将偏移量加入向量并且返回
            else if !entry.is_delete() && entry.get_name() == convert_short_name {
                // 首先计算一共有多少个被分割的长目录项，新建向量
                let mut long_dir_location = Vec::new();
                let entry_num = (name.as_bytes().len() + 12) / 13;
                // 然后从offset开始，一次减32字节，读取对应长目录项的检验和，和短目录项检验和对比，一致则计算sector与off并且加入向量，不一致则返回none
                for i in (1..entry_num + 1) {
                    dir.read_at(offset - i as u32 * 32, long_entry.trans_to_mut_bytes(), self.dev, &self.fs.read().get_fat(), &self.fs);
                    // 检验和相等！！！
                    if long_entry.get_checksum() == entry.get_check_sum() {
                        long_dir_location.push(self.get_sector_offset(offset - i as u32 * 32));
                    }
                    // 否则返回空，因为出现被分割的长目录项检验和与短目录项不一致的情况
                    else {
                        return None
                    }
                }
                let (sector, off) = self.get_sector_offset(offset);
                return Some(
                    VFSFile::new(
                        self.dev, 
                        // 克隆！！！
                        self.fs.clone(), 
                        sector, 
                        off, 
                        long_dir_location, 
                        String::from(name), 
                        entry.get_flag(),)
                )
            }
            // 该目录项不是要找的，则寻找下一个
            else {
                offset += 32;
            }
        }
    }

    // 当前目录中找名字为name的目录项
    pub fn find_name(&self, name: &str,) -> Option<VFSFile> {
        // 检查是否是目录，以后可能用断言实现,或许不panic只println更好
        if !self.is_dir() {
            panic!("can not find file in no-directory");
        }

        let (file_name, extension_name) = Fat32Manager::split_name_extension(name);

        let file_name_byte_arr = file_name.as_bytes();
        let extension_name_byte_arr = extension_name.as_bytes();

        // if file_name_byte_arr.len() > 8 || extension_name_byte_arr.len() > 3 {
        //     self.find_long_name(name, *self.read_ShortDirEntry(|sd : &ShortDirEntry|{ sd }))
        // }
        // else {
        //     self.find_short_name(name, *self.read_ShortDirEntry(|sd : &ShortDirEntry|{ sd }))
        // } 
        self.read_ShortDirEntry(|short_ent:&ShortDirEntry|{
            if file_name_byte_arr.len() > 8 || extension_name_byte_arr.len() > 3 { //长文件名
                return self.find_long_name(name, short_ent)
            } else { // 短文件名
                return self.find_short_name(name, short_ent) 
            }
        })
    }

    // 通过路径path在当前目录中开始寻找该目录项
    pub fn find_name_by_path(&self, path: &str) -> Option<Arc<VFSFile>> {
        let pos :Vec<&str> = path.split("/").collect();
        if pos.len() == 0 {
            return Some( Arc::new(self.clone()) );
        }
        let mut current = self.clone();
        for i in (0 .. pos.len()) {
            if pos[i] == "" || pos[i] == "."{
                continue;
            }
            else if let Some(file) = current.find_name(pos[i]) {
                current = file;
            }
            else {
                println!("do not have this file");
                return None
            }
        }
        Some(Arc::new(current))
    }


    // 返回值注意，加arc要
    pub fn create(&mut self, name: &str, flag: u8) -> Option<VFSFile> {
        // 判断该文件是否合法
        if self.is_long_dir() {
            println!("illeagal dirent entry!");
            return None
        }
        // 判断该文件是否为目录
        if !self.is_dir() {
            println!("you can not create file in the no-dirent file!");
            return None
        }
        
        let (file_name, extension_name) = Fat32Manager::split_name_extension(name);
        let mut short_dir_entry;
        let mut long_dir_location = Vec::new();
        let mut short_sector: u32;
        let mut short_off: u32;

        if file_name.as_bytes().len() > 8 || extension_name.as_bytes().len() > 3 {
            
            let short_dir_entry_name = Fat32Manager::long_name_to_short(name);
            
            let (short_file_name, short_extension_name) = Fat32Manager::short_name_to_byte_arr(short_dir_entry_name.as_str());
            short_dir_entry = ShortDirEntry::new(&short_file_name, &short_extension_name, flag);
            let checksum = short_dir_entry.get_check_sum();
            let mut long_dir_entry_vec = Fat32Manager::split_long_name(name);
            let mut dir_offset;
            let len = long_dir_entry_vec.len() as u8;
            // 新建长目录项
            while long_dir_entry_vec.len() > 0 {
                let mut attr: u8 = long_dir_entry_vec.len() as u8;
                if attr == len {
                    attr |= 0x40;
                }
                let long_dir_entry_name = long_dir_entry_vec.pop().unwrap();
                let long_dir_entry = LongDirEntry::new(attr, long_dir_entry_name.as_str().as_bytes(), checksum);
                if let Some(offset) = self.find_next_free_dirent() {
                    dir_offset = offset;
                } 
                
                else {
                    return None
                } 
                self.write_at(dir_offset as u32, long_dir_entry.trans_to_bytes());
                let (sector, off) = self.get_sector_offset(dir_offset);
                long_dir_location.push((sector, off));
            }

            if let Some(offset) = self.find_next_free_dirent() {
                dir_offset = offset;
            } 
            else {
                return None
            } 
            (short_sector, short_off) = self.get_sector_offset(dir_offset);
            self.write_at(dir_offset as u32, short_dir_entry.trans_to_mut_bytes());
        }
        else {
            
            let (file_name_byte_arr, extension_name_byte_arr) = Fat32Manager::short_name_to_byte_arr(name);
            short_dir_entry = ShortDirEntry::new(&file_name_byte_arr, &extension_name_byte_arr, flag);
            let mut dir_offset;
            if let Some(offset) = self.find_next_free_dirent() {                            
                dir_offset = offset;
                
            } else {
                return None
            } 
            (short_sector, short_off) = self.get_sector_offset(dir_offset);
            
            self.write_at(dir_offset as u32, short_dir_entry.trans_to_mut_bytes());
        }

        let mut res = VFSFile::new(
            self.dev, 
            self.fs.clone(), 
            short_sector, 
            short_off, 
            long_dir_location, 
            //此参数传的是切片
            String::from(name), 
            flag,);
        // 如果是目录，则需要新建.和..目录项！！！
        if short_dir_entry.is_dir() {
            let (file_self, extension_self) = Fat32Manager::short_name_to_byte_arr(".");
            let (file_parent, extension_parent) = Fat32Manager::short_name_to_byte_arr("..");
            let mut self_dirent = ShortDirEntry::new(&file_self, &extension_self, SUB_DIRECTORY);
            let mut parent_dirent = ShortDirEntry::new(&file_parent, &extension_parent, SUB_DIRECTORY);
            let mut dir_offset;
            // 每新建一个文件都给其分配一个簇（这里写64，实际底层分配一个簇的字节）
            res.increase(64);
            self_dirent.set_start_cluster(res.get_start_cluster());
            parent_dirent.set_start_cluster(self.get_start_cluster());

            if let Some(offset) = res.find_next_free_dirent() {
                dir_offset = offset;
            } else {
                return None
            } 
            res.write_at(dir_offset as u32, self_dirent.trans_to_bytes());

            if let Some(offset) = res.find_next_free_dirent() {
                dir_offset = offset;
            } else {
                return None
            } 
            res.write_at(dir_offset as u32, parent_dirent.trans_to_bytes());
        }
        Some(res)

    }


    pub fn delete(&mut self) {
        if self.is_dir() {
            self.delete_dirent()
        }
        else {
            self.delete_file()
        }
    }
    
    //将自身文件内容删除(必须是普通文件)
    pub fn delete_file(&mut self) {
        if self.is_dir() {
            panic!("dirent can not call this method!");
        }

        self.fs.read().dealloc_cluster(self.get_start_cluster(), self.dev);

        self.set_file_length(0);

        self.set_start_cluster(0);

        self.set_delete();

        for i in (0..self.long_dir_location.len()) {
            self.modify_LongDirEntry(i, |ld : &mut LongDirEntry|{
                ld.set_delete();
            })
        }

    }

    //将自身文件内容删除(必须是目录文件),注意，删除目录文件时内部目录项所代表的所有文件都要被删除！！！(日后实现)
    pub fn delete_dirent(&mut self) {
        if !self.is_dir() {
            panic!("no-dirent can not call this method!");
        }

    }

    //罗列目录中所有文件
    pub fn ls(&self) {

    }
    //获取相关文件信息
    pub fn stat(&self) {

    }

}