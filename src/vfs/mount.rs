use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;


// 设备名
pub type Special = String;

// 文件系统名
pub type FsType = String;

const MAX_MOUNT_NUM: usize = 16;

pub struct FileSystems {
    fs_queue: Vec<(Special, String, FsType)>
}

impl FileSystems {
    pub fn mount(mut self, special: Special, dir: String, fstype: FsType, flags: u32, data: *const u8) -> isize {
        if self.fs_queue.len() >= MAX_MOUNT_NUM {
            log!("the mount queue is too long!");
            return -1;
        }

        let len = self.fs_queue.len();
        //若dir以前被挂载过，则覆盖之，但不保存以前覆盖结果，即取消此挂载后不会像linux那般恢复之前挂载结果
        for i in 0..len {
            if self.fs_queue[i].1 == dir {
                self.fs_queue[i].0 = special;
                self.fs_queue[i].2 = fstype;
                return 0
            }
        }
        
        self.fs_queue.push((special, dir, fstype));
        return 0;
        
    }

    pub fn umount(mut self, special: Special, flags: u32,) -> isize {
        let len = self.fs_queue.len();
        
        for i in 0..len {
            if self.fs_queue[i].0 == special {
                self.fs_queue.remove(i);
                return 0
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