use spin::RwLock;
use alloc::boxed::Box;
use alloc::sync::Arc;

use super::dentry::Dentry;

use crate::sbi::*;

pub enum FileType {
    File,
    Directory,
}

pub enum FileOpenMode {
    Read,
    Write
}


#[derive(Debug)]
pub enum FileErr {
    NotWrite
}


lazy_static!{
    pub static ref STDIN: Arc<RwLock<Box<dyn _File + Send + Sync>>> = Arc::new(RwLock::new(Box::new(Console::new())));
    pub static ref STDOUT: Arc<RwLock<Box<dyn _File + Send + Sync>>> = Arc::new(RwLock::new(Box::new(Console::new())));
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
    line_pos: usize
}

impl Console {
    pub fn new() -> Self{
        Self {
            #[cfg(feature = "read_buffer")]
            line_buf: [0; INPUT_BUF_SIZE],
            #[cfg(feature = "read_buffer")]
            line_end: 0,
            #[cfg(feature = "read_buffer")]
            line_pos: 0
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

impl _File for Console {
    fn write(&mut self, buf: &[u8]) -> Result<(), FileErr> {
        unsafe {
            log!("user_log":>"{}", core::str::from_utf8_unchecked(buf));
        }
        Ok(())
    }
    fn read(&mut self, buf: &mut [u8]) -> Result<(), FileErr> {
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
            while i <  buf.len() {
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
        Ok(())
    }
}

pub trait _File {
    fn lseek(&mut self, offset: isize) -> Result<(), FileErr> {
        unimplemented!("lseek")
    }
    fn read(&mut self, buf: &mut [u8]) -> Result<(), FileErr> {
        unimplemented!("read")
    }
    fn write(&mut self, buf: &[u8]) -> Result<(), FileErr> {
        unimplemented!("write")
    }
    fn readdir(&mut self) -> Result<Dentry, FileErr> {
        unimplemented!("readdir")
    }

    fn open(&mut self, mode: FileOpenMode) -> Result<(), FileErr> {
        unimplemented!("open")
    }

    fn file_type(&self) -> FileType {
        unimplemented!("file_type")
    }
    fn open_mode(&self) -> FileOpenMode {
        unimplemented!("open_mode")
    }
    fn get_uid(&self) -> usize {
        unimplemented!("get_uid")
    }
    fn get_gid(&self) -> usize {
        unimplemented!("get_gid")
    }
    fn get_pos(&self) -> usize {
        unimplemented!("get_pos")
    }

}

/*
pub struct TestBlk {}
impl fatfs::IoBase for TestBlk {
    type Error = ();
}
impl fatfs::Read for TestBlk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }
}
impl fatfs::Write for TestBlk {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl fatfs::Seek for TestBlk {
    fn seek(&mut self, _: fatfs::SeekFrom) -> Result<u64, ()> {
        Ok(0)
    }
}
lazy_static!{
        static ref FAT: Arc<RwLock<fatfs::FileSystem<TestBlk, fatfs::DefaultTimeProvider, fatfs::LossyOemCpConverter>>> = Arc::new(RwLock::new(fatfs::FileSystem::new(TestBlk{}, fatfs::FsOptions::new()).unwrap()));
}


fn fpos_add(fpos: usize, offset: isize) -> usize {
    if offset < 0 {
        fpos - (-offset) as usize
    } else {
        fpos + offset as usize
    }
}

#[derive(Clone, Copy)]
pub struct RegularFileOp {}

impl FileOp for RegularFileOp {
    fn lseek(&self, fp: *mut File, offset: isize) -> Result<(), FileOpErr> {
        unsafe {
            (*fp).f_pos = fpos_add((*fp).f_pos, offset);
        }
        Ok(())
    }
    fn read(&self, fp: *mut File, buf: &mut [u8]) -> Result<(), FileOpErr> {
        unsafe {
            if let FileType::File = (*fp).f_type {
                let mut size = buf.len();
                if let Some(inode) = (*fp).f_inode.clone() {
                    if inode.read().i_size < (*fp).f_pos + size {
                        // Out of file's size
                        return Err(FileOpErr {});
                    }
                    let mut readed = 0;
                    while size > 0 {
                        let read_size = min(BLOCK_BUFFER_SIZE, size);
                        let block = inode.read().i_op.bmap(inode.clone(), (*fp).f_pos).unwrap();
                        let buffer = get_block(0, block);
                        let data = buffer.data();
                        core::ptr::copy(
                            data.as_ptr(),
                            buf.split_at_mut(readed).1.as_mut_ptr(),
                            read_size,
                        );
                        size -= read_size;
                        readed += read_size;
                        (*fp).f_pos += read_size;
                    }
                    return Ok(());
                }
                return Err(FileOpErr {});
            } else {
                return Err(FileOpErr {});
            }
        }
    }
    fn write(&self, fp: *mut File, buf: &[u8]) -> Result<(), FileOpErr> {
        println!("[kernel]: call write: {}", core::str::from_utf8(buf).unwrap());
        unsafe {
            if let FileType::File = (*fp).f_type {
                let mut size = buf.len();
                if let Some(inode) = (*fp).f_inode.clone() {
                    if inode.read().i_size < (*fp).f_pos + size {
                        // Out of file's size
                        return Err(FileOpErr {});
                    }
                    let mut writed = 0;
                    while size > 0 {
                        let write_size = min(BLOCK_BUFFER_SIZE, size);
                        let block = inode.write().i_op.bmap(inode.clone(), (*fp).f_pos).unwrap();
                        let buffer = get_block(0, block);
                        let mut data = buffer.data();
                        core::ptr::copy(
                            buf.split_at(writed).1.as_ptr(),
                            data.as_mut_ptr(),
                            write_size,
                        );
                        size -= write_size;
                        writed += write_size;
                        (*fp).f_pos += write_size;
                    }
                    Ok(())
                } else {
                    Err(FileOpErr {})
                }
            } else {
                Err(FileOpErr {})
            }
        }
    }
    fn readdir(&self, fp: *mut File) -> Result<Dentry, FileOpErr> {
        // Regular file don't support readdir
        Err(FileOpErr {})
    }

    fn open(&self, fp: *mut File, inode: Arc<RwLock<Inode>>) -> Result<(), FileOpErr> {
        unsafe {
            (*fp).f_inode = Some(inode);
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
pub struct ConsoleFileOp {}

impl FileOp for ConsoleFileOp {
    fn lseek(&self, _: *mut File, _: isize) -> Result<(), FileOpErr> {
        // Console file don't support lseek
        Err(FileOpErr{})
    }
    fn read(&self, fp: *mut File, buf: &mut [u8]) -> Result<(), FileOpErr> {
        // Fixme: add read
        unsafe {
            if let FileOpenMode::Write = (*fp).f_mode {
                // can't read
                return Err(FileOpErr{})
            }

        }
        Ok(())
    }
    fn write(&self, fp: *mut File, buf: &[u8]) -> Result<(), FileOpErr> {
        unsafe {
            if let FileOpenMode::Read = (*fp).f_mode {
                // can't write
                return Err(FileOpErr{})
            }

        }
        for i in buf.iter() {
            sbi_call(PUT_CHAR, [*i as usize, 0, 0]);
        }
        Ok(())
    }
    fn readdir(&self, fp: *mut File) -> Result<Dentry, FileOpErr> {
        // Regular file don't support readdir
        Err(FileOpErr {})
    }

    fn open(&self, fp: *mut File, inode: Arc<RwLock<Inode>>) -> Result<(), FileOpErr> {
        // Console file don't support open
        Err(FileOpErr {})
    }

}
*/