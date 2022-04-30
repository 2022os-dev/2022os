use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
use spin::Mutex;

use super::*;

// 设备名
type Special = String;

// 文件系统名
type FsType = String;

// 文件路径名名
type Path = String;

const MAX_MOUNT_NUM: usize = 16;

pub type MountItem = (Special, FsType, FsType);

pub struct FileSystems {
    pub fs_queue: Vec<(Special, Path, FsType)>,
}

impl FileSystems {
    pub fn mount(&mut self, special: Special, dir: Path, fstype: FsType) -> isize {
        let len = self.fs_queue.len();

        if len >= MAX_MOUNT_NUM {
            println!("the mount queue is too long!");
            return -1;
        }

        //若dir以前被挂载过，则覆盖之，但不保存以前覆盖结果，即取消此挂载后不会像linux那般恢复之前挂载结果
        for i in 0..len {
            if self.fs_queue[i].1 == dir {
                self.fs_queue[i].0 = special;
                self.fs_queue[i].2 = fstype;

                for j in 0..self.fs_queue.len() {
                    println!(
                        "{} mount to {} , filesystem type is {}",
                        self.fs_queue[j].0, self.fs_queue[j].1, self.fs_queue[j].2
                    );
                }
                return 0;
            }
        }

        self.fs_queue.push((special, dir, fstype));

        for i in 0..self.fs_queue.len() {
            println!(
                "{} mount to {} , filesystem type is {}",
                self.fs_queue[i].0, self.fs_queue[i].1, self.fs_queue[i].2
            );
        }
        println!(" ");

        return 0;
    }

    pub fn umount(&mut self, special: Special) -> isize {
        let len = self.fs_queue.len();

        for i in 0..len {
            if self.fs_queue[i].0 == special {
                self.fs_queue.remove(i);
                for j in 0..self.fs_queue.len() {
                    println!(
                        "{} mount to {} , filesystem type is {}",
                        self.fs_queue[j].0, self.fs_queue[j].1, self.fs_queue[j].2
                    );
                }
                return 0;
            }
        }

        return -1;
    }
}

lazy_static! {
    pub static ref FS_QUEUE: Arc<Mutex<FileSystems>> = {
        let fs_system = FileSystems {
            fs_queue: Vec::new(),
        };
        Arc::new(Mutex::new(fs_system))
    };
}
