use alloc::string::String;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::RwLock;
use super::*;

pub struct MemInode {
    inner: RwLock<InodeInner>
}

struct InodeInner {
    children: BTreeMap<String, Arc<MemInode>>,
    data: [u8; 512],
    used: bool,
    len: usize,
}

impl MemInode {
    fn new() -> Self {
        Self {
            inner: RwLock::new(InodeInner{
                children: BTreeMap::new(),
                data: [0; 512],
                used: false,
                len: 0,
            })
        }
    }
}


impl _Inode for MemInode {
    fn read_offset(&self, mut offset: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        log!("vfs":"mem_read">"({})", offset);
        let mut i = 0;
        while i < buf.len() {
            if offset >= 512 {
                break
            }
            buf[i] = self.inner.read().data[offset];
            i += 1;
            offset += 1;
        }
        Ok(i)
    }

    fn write_offset(&self, mut offset: usize, buf: &[u8]) -> Result<usize, FileErr> {
        log!("vfs":"mem_write">"({})", offset);
        let mut i = 0;
        while i < buf.len() {
            if offset >= 512 {
                break
            }
            self.inner.write().data[offset] = buf[i];
            i += 1;
            offset += 1;
        }
        if self.inner.read().len <= offset {
            self.inner.write().len = offset
        }
        Ok(i)
    }

    fn create(&self, subname: &str, _: FileMode, itype: InodeType) -> Result<Inode, FileErr> {
        log!("vfs":"mem_create">"({})", subname);
        match itype {
            InodeType::Directory |
            InodeType::File => {
                let inode = alloc_inode();
                if let Err(_) = inode {
                    return Err(FileErr::NotDefine)
                }
                self.inner.write().children.insert(String::from(subname), inode.clone().unwrap());
                Ok(inode.unwrap())
            }
            _ => {
                Err(FileErr::NotDefine)
            }
        }
    }

    fn open_child(&self, name: &str, flags: OpenFlags) -> Result<Fd, FileErr> {
        log!("vfs":"mem_open">"({})", name);
        if let Ok(file) = self.get_child(name).and_then(|inode| {
            File::open(inode, flags)
        }) {
            Ok(file)
        } else {
            Err(FileErr::NotDefine)
        }
    }

    fn get_child(&self, name: &str) -> Result<Inode, FileErr> {
        log!("vfs":"mem_getchild">"({})", name);
        if let Some(child) = self.inner.read().children.get(name) {
            Ok(child.clone())
        } else {
            Err(FileErr::InodeNotChild)
        }
    }

    fn len(&self) -> usize {
        self.inner.read().len
    }

}

lazy_static!{
    pub static ref ROOT: Inode = Arc::new(MemInode::new());
    pub static ref MEMINODES: RwLock<Vec<Arc<MemInode>>> = RwLock::new(Vec::new());
}

fn alloc_inode() -> Result<Arc<MemInode>, ()> {
    if MEMINODES.read().len() == 0 {
        memfs_init();
    }
    for i in 0..MEMINODES.read().len() {
        if MEMINODES.read()[i].inner.read().used == false {
            MEMINODES.read()[i].inner.write().used = true;
            return Ok(MEMINODES.read()[i].clone())
        }
    }
    Err(())
}

pub fn memfs_init() {
    for _ in 0..10 {
        MEMINODES.write().push(Arc::new(MemInode::new()));
    }
}