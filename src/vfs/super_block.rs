use crate::alloc::boxed::Box;
use crate::alloc::vec::Vec;
use core::any::Any;

use super::inode::{Inode, InodeOpErr};

pub type SuperOpErr = InodeOpErr;

pub trait SuperBlockOp: Send + Sync {
    fn fsinit(&self, sb: *mut SuperBlock) 
                    -> Result<(), SuperOpErr>;
    fn alloc_inode(&self, sb: &mut SuperBlock) 
                    -> Result<*mut Inode, SuperOpErr>;
    fn delete_inode(&self, inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    fn read_inode(&self, inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    fn write_inode(&self, inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    fn put_inode(&self , inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    fn put_super(&self, sb: &mut SuperBlock)
                    -> Result<(), SuperOpErr>;
    fn write_super(&self, sb: &mut SuperBlock)
                    -> Result<(), SuperOpErr>;
}

pub struct SuperBlock<'a> {
    s_dev: usize,           // 设备标识符
    s_blocksize: usize,     // 以字节为单位的块大小
    s_startblock: usize,    // 文件系统开始的块号
    s_dirt: bool,           // 修改标志
    s_maxbytes: usize,      // 文件最长长度
    s_op: Box<&'a dyn SuperBlockOp>,
    s_inodes: Vec<*mut Inode<'a>>,  // 文件系统所有Inode
    s_fs_info: Box<&'a dyn Any>,
    s_root: *mut Inode<'a>,         // 文件系统根目录的Inode
}
