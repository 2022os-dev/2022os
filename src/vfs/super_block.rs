use crate::alloc::boxed::Box;
use crate::alloc::vec::Vec;
use core::any::Any;

use super::inode::{Inode, InodeOpErr};

pub type SuperOpErr = InodeOpErr;

pub trait SuperBlockOp: Send + Sync {
    //文件系统初始化
    fn fsinit(&self, sb: *mut SuperBlock) 
                    -> Result<(), SuperOpErr>;
    //分配一个与文件系统关联的Inode对象
    fn alloc_inode(&self, sb: &mut SuperBlock) 
                    -> Result<*mut Inode, SuperOpErr>;
    //在内存与磁盘同时删除Inode节点，需要检查引用数是否归0
    fn delete_inode(&self, inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    //从磁盘读取inode数据，必须保证inode的i_ino被正确填写，通过i_ino读取inode
    fn read_inode(&self, inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    // 将Inode结构写回磁盘
    fn write_inode(&self, inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    // 将Inode引用数减1
    fn put_inode(&self , inode: &mut Inode)
                    -> Result<(), SuperOpErr>;
    //文件系统被取消挂载，可以暂时不做实现，因为目前只有根文件系统，不会取消
    fn put_super(&self, sb: &mut SuperBlock)
                    -> Result<(), SuperOpErr>;
    // 与磁盘同步SuperBlock
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
