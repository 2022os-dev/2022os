use alloc::sync::Arc;
use core::convert::TryFrom;
use core::mem::size_of;
use core::slice::from_raw_parts_mut;
use spin::RwLock;

use super::LinuxDirent;
use crate::sbi::*;
use super::Kstat;

pub enum InodeType {
    File,
    Directory,
    HardLink(Inode),
    SymbolLink,
}
bitflags! {
    // 表示openat(2) 中的flags
    pub struct OpenFlags: usize {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 6;
        const TRUNC = 1 << 10;
        const DIRECTROY = 0200000;
        const LARGEFILE  = 0100000;
        const CLOEXEC = 02000000;
    }
    // 表示openat(2) 中的mode_t
    pub struct FileMode: usize {
    }
}
impl OpenFlags {
    pub fn readable(&self) -> bool {
        self.bits() & 1 == 0 || self.contains(OpenFlags::RDWR)
    }
    pub fn writable(&self) -> bool {
        self.bits() & 1 == 1 || self.contains(OpenFlags::RDWR)
    }
}

#[derive(Debug)]
pub enum FileErr {
    // File没有读权限
    FileNotWrite,
    // File没有写权限
    FileNotRead,
    // 读到末尾
    FileEOF,
    // Fixme: 未定义的错误
    NotDefine,
    // Directory中找不到Child
    InodeNotChild,
    // 目录项被删除
    InodeDelete,
    // Inode不是Directory
    InodeNotDir,
    // 目录Inode中已经存在同名的child
    InodeChildExist,
    InodeEndOfDir,
    // Fd不正确
    FdInvalid,
    // Pipe需要等待另一端写入
    PipeReadWait,
    // Pipe需要等待另一端读出
    PipeWriteWait,
}

// File descriptor
pub type Fd = Arc<RwLock<File>>;
pub type Inode = Arc<dyn _Inode + Send + Sync + 'static>;

// File description
pub struct File {
    pos: usize,
    flags: OpenFlags,
    pub inode: Inode,
}

impl File {
    pub fn open(inode: Inode, flags: OpenFlags) -> Result<Fd, FileErr> {
        inode.file_open(flags);
        Ok(Arc::new(RwLock::new(Self {
            pos: 0,
            flags,
            inode,
        })))
    }

    pub fn fstat(&self, kstat: &mut Kstat) {
        self.inode.get_kstat(kstat);
    }

    pub fn lseek(&mut self, whence: usize, off: isize) -> Result<usize, FileErr> {
        if whence == 0 {
            // SEEK_SET
            if let Ok(off) = usize::try_from(off) {
                self.pos = off;
            } else {
                return Err(FileErr::NotDefine);
            }
        } else if whence == 1 {
            // SEEK_CUR
            if off > 0 {
                if let Some(i) = self.pos.checked_add(off as usize) {
                    self.pos = i;
                } else {
                    return Err(FileErr::NotDefine);
                }
            } else {
                if let Some(i) = self.pos.checked_sub((-off) as usize) {
                    self.pos = i;
                } else {
                    return Err(FileErr::NotDefine);
                }
            }
        } else if whence == 2 {
            // SEEK_END
            if off > 0 {
                if let Some(i) = self.inode.len().checked_add(off as usize) {
                    self.pos = i
                } else {
                    return Err(FileErr::NotDefine);
                }
            } else {
                if let Some(i) = self.inode.len().checked_sub((-off) as usize) {
                    self.pos = i
                } else {
                    return Err(FileErr::NotDefine);
                }
            }
        } else {
            return Err(FileErr::NotDefine);
        }
        Ok(self.pos)
    }

    // 成功则返回读写的字节数, 若读到目录结尾返回0
    pub fn get_dirents(&mut self, buf: &mut [u8]) -> Result<usize, FileErr> {
        if !self.flags().readable() {
            return Err(FileErr::FileNotRead);
        }
        let mut dirent = LinuxDirent::new();
        let nums = buf.len() / size_of::<LinuxDirent>();
        let buf = unsafe { from_raw_parts_mut(buf.as_mut_ptr() as *mut LinuxDirent, nums) };
        for i in 0..nums {
            match self.inode.get_dirent(self.pos, &mut dirent) {
                Ok(off) => {
                    buf[i].fill(&dirent);
                    self.pos += off;
                }
                Err(e) => {
                    if i == 0 {
                        return Err(e);
                    } else {
                        return Ok(i * size_of::<LinuxDirent>());
                    }
                }
            };
        }
        Ok(nums * size_of::<LinuxDirent>())
    }

    pub fn flags(&self) -> OpenFlags {
        self.flags
    }

    pub fn get_inode(&self) -> Inode {
        self.inode.clone()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileErr> {
        if !self.flags.readable() {
            return Err(FileErr::FileNotRead);
        }
        if self.pos >= self.inode.len() {
            return Err(FileErr::FileEOF);
        }
        self.inode.read_offset(self.pos, buf).and_then(|size| {
            self.pos += size;
            Ok(size)
        })
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, FileErr> {
        if !self.flags.writable() {
            return Err(FileErr::FileNotWrite);
        }
        self.inode.write_offset(self.pos, buf).and_then(|size| {
            self.pos += size;
            Ok(size)
        })
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // 文件关闭时通知Inode
        self.inode.file_close(self);
    }
}
pub trait _Inode {


    fn get_kstat(&self, kstat: &mut Kstat) {
        log!("vfs":"inode">"get_kstat");
    }
    // 如果Inode不是目录，返回Err(FileErr::NotDir)
    fn get_child(&self, _: &str) -> Result<Inode, FileErr> {
        Err(FileErr::NotDefine)
    }

    // 获取一个目录项, offset用于供inode判断读取哪个dirent 返回需要File更新的offset量
    //     读到目录结尾返回InodeEndOfDir
    fn get_dirent(&self, _: usize, _: &mut LinuxDirent) -> Result<usize, FileErr> {
        Err(FileErr::NotDefine)
    }

    // 在当前目录创建一个文件，文件类型由InodeType指定
    fn create(&self, _: &str, _: FileMode, _: InodeType) -> Result<Inode, FileErr> {
        Err(FileErr::NotDefine)
    }

    // 从Inode的某个偏移量读出
    fn read_offset(&self, _: usize, _: &mut [u8]) -> Result<usize, FileErr> {
        Err(FileErr::NotDefine)
    }

    // 在Inode的某个偏移量写入
    fn write_offset(&self, _: usize, _: &[u8]) -> Result<usize, FileErr> {
        Err(FileErr::NotDefine)
    }

    // Inode表示的文件都长度, 必须实现，用于read检测EOF
    fn len(&self) -> usize;

    // File打开时通知Inode，可以方便Inode记录引用
    fn file_open(&self, _: OpenFlags) {
        log!("vfs":"inode">"file open");
    }

    // File关闭时通知Inode
    fn file_close(&self, _: &File) {
        log!("vfs":"inode">"file close");
    }

    fn get_uid(&self) -> usize {
        0
    }

    fn get_gid(&self) -> usize {
        0
    }

    // 判断Inode是否准备好读，由于协助异步IO
    fn read_ready(&self) -> bool {
        unimplemented!("ready_read")
    }

    // 判断Inode是否准备好，由于协助异步IO
    fn write_ready(&self) -> bool {
        unimplemented!("write_read")
    }
}

lazy_static! {
    pub static ref CONSOLE: Inode = Arc::new(Console::new());
    pub static ref STDIN: Fd = File::open(CONSOLE.clone(), OpenFlags::RDONLY).unwrap();
    pub static ref STDOUT: Fd = File::open(CONSOLE.clone(), OpenFlags::WRONLY).unwrap();
}

#[cfg(feature = "read_buffer")]
const INPUT_BUF_SIZE: usize = 512;
pub struct Console {
    // 在内核设置行缓存区
    #[cfg(feature = "read_buffer")]
    line_buf: [u8; INPUT_BUF_SIZE],
    #[cfg(feature = "read_buffer")]
    line_end: usize,
    #[cfg(feature = "read_buffer")]
    line_pos: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "read_buffer")]
            line_buf: [0; INPUT_BUF_SIZE],
            #[cfg(feature = "read_buffer")]
            line_end: 0,
            #[cfg(feature = "read_buffer")]
            line_pos: 0,
        }
    }

    #[cfg(feature = "read_buffer")]
    fn read_line(&mut self) {
        let mut ch;
        self.line_end = 0;
        self.line_pos = 0;
        while self.line_end < INPUT_BUF_SIZE {
            ch = sbi_legacy_call(GET_CHAR, [0, 0, 0]);
            if ch < 0 {
                // 阻塞读入
                continue;
            }
            self.line_buf[self.line_end] = ch as u8;
            self.line_end += 1;
            // 回车
            if ch == 13 {
                #[cfg(feature = "input_echo")]
                print!("\n");
                break;
            }
            // 回显
            #[cfg(feature = "input_echo")]
            print!("{}", ch as u8 as char);
        }
    }
}

impl _Inode for Console {
    fn len(&self) -> usize {
        usize::MAX
    }
    fn write_offset(&self, _: usize, buf: &[u8]) -> Result<usize, FileErr> {
        unsafe {
            #[cfg(not(feature = "batch"))]
            print!("{}", core::str::from_utf8_unchecked(buf));
            #[cfg(feature = "batch")]
            log!("user_log":>"{}", core::str::from_utf8_unchecked(buf));
        }
        Ok(buf.len())
    }
    fn read_offset(&self, _: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        let mut i = 0;
        while i < buf.len() {
            #[cfg(feature = "read_buffer")]
            {
                if self.line_pos >= self.line_end {
                    self.read_line();
                }
                while i < buf.len() && self.line_pos < self.line_end {
                    buf[i] = self.line_buf[self.line_pos];
                    self.line_pos += 1;
                    i += 1;
                }
            }
            #[cfg(not(feature = "read_buffer"))]
            {
                let mut ch;
                while i < buf.len() {
                    ch = sbi_legacy_call(GET_CHAR, [0, 0, 0]);
                    if ch < 0 {
                        // 阻塞读入
                        continue;
                    }
                    buf[i] = ch as u8;
                    // 回显
                    #[cfg(feature = "input_echo")]
                    if ch == 13 {
                        // 回车
                        print!("\n");
                    } else {
                        print!("{}", ch as u8 as char);
                    }
                    i += 1;
                }
            }
        }
        Ok(buf.len())
    }
}
