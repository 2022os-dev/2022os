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
    fn create(&self, dir: &mut Inode, name: String)
              -> Result<*mut Inode, InodeOpErr>;
    fn lookup(&self, dir: &mut Inode, name: String)
              -> Result<*mut Inode, InodeOpErr>;
    fn link(&self, oldinode: *mut Inode, dir: *mut Inode,
            name: String) -> Result<(), InodeOpErr>;
    fn unlink(&self, dir: *mut Inode, name: String)
             -> Result<(), InodeOpErr>;
    fn symlink(&self, dir: *mut Inode, path: String,
               sym_name: String) -> Result<(), InodeOpErr>;
    fn mkdir(&self, dir: *mut Inode, name: String, flag: InodeFlag)
                -> Result<(), InodeOpErr>;
    fn rmdir(&self, dir: *mut Inode, name: String)
             -> Result<(),InodeOpErr>;
    fn mknod(&self, dir: *mut Inode, name: String,
             mode: InodeMode, rdev: usize) -> Result<(), InodeOpErr>;
    fn rename(&self, old_dir: *mut Inode, old_name: String,
              new_dir: *mut Inode, new_name: String)
              -> Result<(), InodeOpErr>;
    fn readlink(&self, link: *mut Inode)
                -> Result<String, ()>;
    fn follow_link(&self, dir: *mut Inode, inode: *mut Inode)
                    -> Result<*mut Inode, InodeOpErr>;
    fn truncate(&self, inode: *mut Inode) -> Result<(), ()>;
    fn bmap(&self, inode: *mut Inode, offset: usize)
            -> Result<Ino, InodeOpErr>;
}

pub struct Inode<'a> {
    i_ino: Ino,           // 索引节点号
    i_type: InodeType,      // Inode类型
    i_count: AtomicUsize,   // 引用计数器
    i_nlink: AtomicUsize,   // 硬链接数
    i_dirt: bool,           // 脏标志
    i_uid: usize,           // 所有者标识
    i_gid: usize,           // 组标识符
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
