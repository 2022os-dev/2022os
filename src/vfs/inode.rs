use core::sync::atomic::AtomicUsize;
use crate::alloc::boxed::Box;
use crate::spin::Mutex;

use super::super_block::SuperBlock;
use crate::alloc::string::String;
use super::TimeSpec;

pub type InodeMode = usize;
pub type InodeFlag = usize;

pub type Ino = usize; // 磁盘块号

pub enum InodeType {
    File,
    Directory
}

pub enum InodeOpErr {
}

pub trait InodeOp: Send + Sync {
    // 在dir对应的目录Inode下创建一个Inode
    fn create(&self, dir: &mut Inode, name: String)
              -> Result<*mut Inode, InodeOpErr>;
    // 在dir对应的目录Inode下搜索一个名为name的Inode
    fn lookup(&self, dir: &mut Inode, name: String)
              -> Result<*mut Inode, InodeOpErr>;
    // // 在dir对应的目录Inode下创建硬链接，硬链接的名称为name，指向old_inode对应的Inode。实际的实现可能为在dir下创建一个dentry，
    // // dentry的名称为name，dentry指向的ino为old_inode->i_ino
    // fn link(&self, oldinode: *mut Inode, dir: *mut Inode,
    //         name: String) -> Result<(), InodeOpErr>;
    // // 解除dir对应的目录Inode下name指向Inode的硬链接
    // fn unlink(&self, dir: *mut Inode, name: String)
    //          -> Result<(), InodeOpErr>;
    // // 在dir对应的目录Inode下创建符号链接，符号链接的名称是name，符号链接文件指向一个文件路径path，该路径可能为相对路径
    // fn symlink(&self, dir: *mut Inode, path: String,
    //            sym_name: String) -> Result<(), InodeOpErr>;
    // 在dir对应的目录Inode下创建一个目录Inode，可以使用create来实现
    fn mkdir(&self, dir: *mut Inode, name: String, flag: InodeFlag)
                -> Result<(), InodeOpErr>;
    // 在dir对应的目录Inode下删除一个名为name的目录，删除的策略由InodeFlag来指定
    fn rmdir(&self, dir: *mut Inode, name: String)
             -> Result<(),InodeOpErr>;
    // // 
    // fn mknod(&self, dir: *mut Inode, name: String,
    //          mode: InodeMode, rdev: usize) -> Result<(), InodeOpErr>;
    // 将old_inode目录Inode下的old_name对应的dentry移动到new_inode对应的new_namedentry中
    fn rename(&self, old_dir: *mut Inode, old_name: String,
              new_dir: *mut Inode, new_name: String)
              -> Result<(), InodeOpErr>;
    // // 将link对应的符号链接Inode指向的符号链接绝对地址返回
    // fn readlink(&self, link: *mut Inode)
    //             -> Result<String, ()>;
    // // 寻找符号链接inode指向的Inode，如果符号链接内的文件路径为相对路径，则从dir开始解析的，dir为link的父目录
    // fn follow_link(&self, dir: *mut Inode, inode: *mut Inode)
    //                 -> Result<*mut Inode, InodeOpErr>;
    // 改变文件大小，调用前先将Inode的i_size修改，将文件大小修改为i_size
    fn truncate(&self, inode: *mut Inode) -> Result<(), ()>;
    // 获取inode对应的文件偏移为offset字节的内容所在的块号
    fn bmap(&self, inode: *mut Inode, offset: usize)
            -> Result<Ino, InodeOpErr>;
}

pub struct Inode<'a> {
    i_ino: Ino,           // 索引节点号
    i_type: InodeType,      // Inode类型
    i_count: AtomicUsize,   // 引用计数器
    i_nlink: AtomicUsize,   // 硬链接数
    i_dirt: bool,           // 脏标志
    i_uid: usize,           // 所有者标识，目前不支持多用户，置0即可。
    i_gid: usize,           // 组标识符，目前不支持组，置0即可。
    i_size: usize,          // 文件字节数
    i_atime: TimeSpec,      // 上次访问时间
    i_mtime: TimeSpec,      // 上次写文件时间
    i_ctime: TimeSpec,      // 上次修改时间
    i_blksize: usize,       // 块的字节数
    i_blocks: usize,        // 文件的块数
    i_op: Box<&'a dyn InodeOp>, // Inode操作
    i_lock: Mutex<()>,      // 保护某些字段的自旋锁
    i_sb: *mut SuperBlock<'a>, // 超级块指针
}
