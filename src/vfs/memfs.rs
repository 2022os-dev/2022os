use crate::config::PATH_LIMITS;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryFrom;
use spin::RwLock;

use super::*;

lazy_static! {
    pub static ref ROOT: Inode = Arc::new(MemRootInode::new());
    static ref MEMINODES: RwLock<Vec<Arc<MemInode>>> = RwLock::new(Vec::new());
}

struct MemInode {
    inner: RwLock<InodeInner>,
}

struct MemRootInode(MemInode);

struct InodeInner {
    // 设置一个名字方便调试
    name: String,
    children: BTreeMap<String, Inode>,
    data: [u8; 512],
    used: bool,
    len: usize,
}

impl MemInode {
    fn new() -> Self {
        Self {
            inner: RwLock::new(InodeInner {
                name: String::from(""),
                children: BTreeMap::new(),
                data: [0; 512],
                used: false,
                len: 0,
            }),
        }
    }
}

impl _Inode for MemInode {
    fn get_dirent(&self, offset: usize, dirent: &mut LinuxDirent) -> Result<usize, FileErr> {
        // memfs不存在".."和"."
        let inner = self.inner.write();

        let child = inner.children.iter().skip(offset).next();
        if let None = child {
            log!("vfs":"get_dirents">"inode end of dir: len({}), offset({})", inner.children.len(), offset);
            return Err(FileErr::InodeEndOfDir);
        }
        let (name, _) = child.unwrap();

        // memfs不设置inode号
        dirent.d_ino = 0;
        dirent.d_reclen = match u16::try_from(core::mem::size_of::<LinuxDirent>()) {
            Ok(size) => size,
            Err(_) => {
                log!("vfs":"get_dirents">"invalid reclen");
                return Err(FileErr::NotDefine);
            }
        };
        dirent.d_off = match isize::try_from(offset + 1) {
            Ok(size) => size,
            Err(_) => {
                log!("vfs":"get_dirents">"invalid d_off");
                return Err(FileErr::NotDefine);
            }
        };

        // 不区分文件夹和普通文件
        dirent.d_type = DT_REG;
        if name.len() > PATH_LIMITS {
            return Err(FileErr::NotDefine);
        }
        dirent.d_name[0..name.len()].copy_from_slice(name.as_bytes());
        Ok(1)
    }

    fn unlink_child(&self, name: &str, _: bool) -> Result<usize, FileErr> {
        let mut inner = self.inner.write();
        if inner.children.contains_key(&String::from(name)) {
            if inner.children.remove(&String::from(name)).is_some() {
                // Memfs 剩余的链接数为0
                return Ok(0);
            }
        }
        return Err(FileErr::InodeNotChild);
    }

    fn read_offset(&self, mut offset: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        log!("vfs":"mem_read">"offset ({})", offset);
        let mut i = 0;
        let inner = self.inner.read();
        while i < buf.len() {
            if offset >= 512 {
                break;
            }
            buf[i] = inner.data[offset];
            i += 1;
            offset += 1;
        }
        Ok(i)
    }

    fn write_offset(&self, mut offset: usize, buf: &[u8]) -> Result<usize, FileErr> {
        log!("vfs":"mem_write">"offset ({})", offset);
        let mut i = 0;
        let mut inner = self.inner.write();
        while i < buf.len() {
            if offset >= 512 {
                break;
            }
            inner.data[offset] = buf[i];
            i += 1;
            offset += 1;
        }
        if inner.len <= offset {
            inner.len = offset
        }
        Ok(i)
    }

    fn create(&self, subname: &str, _: FileMode, itype: InodeType) -> Result<Inode, FileErr> {
        let mut inner = self.inner.write();
        if subname.len() == 0 {
            // 文件名不正确
            log!("vfs":"mem_create""{}">"invalid name \"{}\"", inner.name, subname);
            return Err(FileErr::NotDefine);
        }
        match itype {
            InodeType::Directory | InodeType::File => {
                if let Some(_) = inner.children.get(&String::from(subname)) {
                    log!("vfs":"mem_create""{}">" exists {}", inner.name, subname);
                    return Err(FileErr::InodeChildExist);
                }
                let inode = alloc_inode();
                if let Err(_) = inode {
                    return Err(FileErr::NotDefine);
                }
                inode.clone().unwrap().inner.write().name = String::from(subname);
                inner
                    .children
                    .insert(String::from(subname), inode.clone().unwrap());
                log!("vfs":"mem_create""{}">"child name ({})", inner.name, subname);
                Ok(inode.unwrap())
            }
            // 硬链接
            InodeType::HardLink(inode) => {
                inner.children.insert(String::from(subname), inode.clone());
                Ok(inode)
            }
            _ => {
                log!("vfs":"mem_create""{}">"failed child name ({})",inner.name, subname);
                Err(FileErr::NotDefine)
            }
        }
    }

    fn get_child(&self, name: &str) -> Result<Inode, FileErr> {
        if let Some(child) = self.inner.read().children.get(name) {
            log!("vfs":"mem_getchild">"got child name ({})", name);
            Ok(child.clone())
        } else {
            log!("vfs":"mem_getchild">"not got child name ({})", name);
            Err(FileErr::InodeNotChild)
        }
    }

    fn len(&self) -> usize {
        self.inner.read().len
    }
}
impl MemRootInode {
    fn new() -> Self {
        Self(MemInode::new())
    }
}

impl _Inode for MemRootInode {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn unlink_child(&self, name: &str, rm_dir: bool) -> Result<usize, FileErr> {
        self.0.unlink_child(name, rm_dir)
    }

    fn get_dirent(&self, offset: usize, dirent: &mut LinuxDirent) -> Result<usize, FileErr> {
        self.0.get_dirent(offset, dirent)
    }

    fn read_offset(&self, offset: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        self.0.read_offset(offset, buf)
    }

    fn write_offset(&self, offset: usize, buf: &[u8]) -> Result<usize, FileErr> {
        self.0.write_offset(offset, buf)
    }
    fn create(&self, subname: &str, mode: FileMode, itype: InodeType) -> Result<Inode, FileErr> {
        self.0.create(subname, mode, itype)
    }
    fn get_child(&self, name: &str) -> Result<Inode, FileErr> {
        // 用于将用户态程序放到根目录下，方便execve系统调用测试
        if let Some(app) = crate::user::APP.get(name) {
            return Ok(Arc::new(ProgInode { data: app }));
        } else {
            self.0.get_child(name)
        }
    }
}

fn alloc_inode() -> Result<Arc<MemInode>, ()> {
    if MEMINODES.read().len() == 0 {
        memfs_init();
    }
    loop {
        for i in 0..MEMINODES.read().len() {
            if let Some(mut inode) = MEMINODES.read()[i].inner.try_write() {
                if inode.used == false {
                    inode.used = true;
                    return Ok(MEMINODES.read()[i].clone());
                }
            }
        }
    }
}

pub fn memfs_init() {
    for _ in 0..20 {
        MEMINODES.write().push(Arc::new(MemInode::new()));
    }
}

struct ProgInode {
    pub data: &'static [u8],
}

impl _Inode for ProgInode {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn read_offset(&self, offset: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        for i in 0..buf.len() {
            buf[i] = self.data[offset + i];
        }
        Ok(buf.len())
    }
}
