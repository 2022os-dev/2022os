use fat32::file;
use fat32::dir;
use fat32::fat;

use alloc::sync::Arc;
use crate::vfs::*;

lazy_static!{
    static ref SDCARD: SDCard = SDCard {};
    static ref VOLUMN: fat32::volume::Volume<SDCard> = fat32::volume::Volume::new(*SDCARD);
    pub static ref FAT32ROOT: Inode = Arc::new(VOLUMN.root_dir());
}

#[derive(Copy, Clone)]
struct SDCard {}

impl block_device::BlockDevice for SDCard {
    type Error = ();
    fn read(&self, buf: &mut[u8], address: usize, _number_of_blocks: usize) -> Result<(), Self::Error> {
        let mut blk = address / 512;
        let mut len_left = buf.len();
        let mut blk_off = address % 512;
        let mut tmp_buf: [u8; 512] = [0; 512];
        let total_len = buf.len();

        while len_left > 0 {
            crate::blockdev::read_block(blk, &mut tmp_buf);
            let single_len = core::cmp::min(len_left, 512 - blk_off);
            buf[total_len - len_left..(total_len - len_left + single_len)]
                .copy_from_slice(&tmp_buf[blk_off..blk_off+single_len]);

            len_left -= single_len;
            blk += 1;
            blk_off = (blk_off + single_len) % 512;
        }
        Ok(())
    }
    fn write(&self, buf: &[u8], address: usize, _number_of_blocks: usize) -> Result<(), Self::Error> {
        let mut blk = address / 512;
        let mut len_left = buf.len();
        let mut blk_off = address % 512;
        let mut tmp_buf: [u8; 512] = [0; 512];
        let total_len = buf.len();

        while len_left > 0 {
            let single_len = core::cmp::min(len_left, 512 - blk_off);
            if single_len != 512 {
                crate::blockdev::read_block(blk, &mut tmp_buf);
            }
            tmp_buf[blk_off..blk_off+single_len]
                .copy_from_slice(&buf[total_len - len_left..(total_len - len_left + single_len)]);

            crate::blockdev::write_block(blk, &mut tmp_buf);

            len_left -= single_len;
            blk += 1;
            blk_off = (blk_off + single_len) % 512;
        }
        Ok(())
    }
}


#[allow(unused)]
pub fn sd_test() {
    let mut root = VOLUMN.root_dir();
    // root.create_dir("from_root_dir").unwrap();
    root.direntry_iter().map(|dent| {
        if dent.is_lfn() {
            let (buf, len) = dent.get_lfn().unwrap();
            println!("Dent: {}", unsafe { core::str::from_utf8_unchecked(&buf[..len]) });
        } else {
            let (buf, len) = dent.get_sfn().unwrap();
            println!("Dent: {}", unsafe { core::str::from_utf8_unchecked(&buf[..len]) });
        }
    }).count();
    /*
    ROOT.unlink_child("hello.txt", false).unwrap_or(0);
    let inode = ROOT.create("hello.txt", FileMode::all(), InodeType::File).unwrap();
    let mut write_buf: [u8; 1025] = [0; 1025];
    for i in 0..write_buf.len() {
        write_buf[i] = 'A' as u8 + (i % 26) as u8;
    }
    let write_len = inode.write_offset(0, &write_buf).unwrap();
    println!("write lenght {}", write_len);
    let mut buf: [u8; 1] = [69];
    println!("inode lenght {}", inode.len());
    for i in 0..inode.len() {
        inode.read_offset(i, &mut buf).unwrap();
        print!("{}", buf[0] as char);
    }
    */
}

impl From<fat32::dir::DirError> for FileErr {
    fn from(err: fat32::dir::DirError) -> FileErr {
        match err {
            fat32::dir::DirError::DirHasExist => {
                FileErr::InodeChildExist
            }
            fat32::dir::DirError::FileHasExist => {
                FileErr::InodeChildExist
            }
            fat32::dir::DirError::IllegalChar => {
                FileErr::InodeNotChild
            }
            _ => {
                FileErr::NotDefine
            }
        }
    }
}

impl crate::vfs::_Inode for fat32::file::File<'_, SDCard> {
    fn read_offset(&self, offset: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        match self.read_off(offset, buf) {
            Err(fat32::file::FileError::WriteError) => {
                Err(FileErr::NotDefine)
            }
            Err(fat32::file::FileError::BufTooSmall) => {
                Err(FileErr::NotDefine)
            }
            Ok(len) => {
                Ok(len)
            }
        }
    }

    fn write_offset(&self, offset: usize, buf: &[u8]) -> Result<usize, FileErr> {
        let _self = unsafe { (self as *const Self as *mut Self).as_mut().unwrap()};
        match _self.write_off(offset, buf) {
            Err(fat32::file::FileError::WriteError) => {
                Err(FileErr::NotDefine)
            }
            Err(fat32::file::FileError::BufTooSmall) => {
                Err(FileErr::NotDefine)
            }
            Ok(len) => {
                Ok(len)
            }
        }
    }

    fn len(&self) -> usize {
        self.detail.length().unwrap()
    }
}

impl crate::vfs::_Inode for fat32::dir::Dir<'static, SDCard> {
    fn get_child(&self, name: &str) -> Result<Inode, FileErr> {
        match self.exist(name) {
            None => Err(FileErr::InodeNotChild),
            Some(di) => if di.is_file() {
                let fat = fat::FAT::new(di.cluster(),
                                   self.device,
                                   self.bpb.fat1());
                Ok(Arc::new(file::File::<SDCard> {
                    device: self.device,
                    bpb: self.bpb,
                    dir_cluster: self.detail.cluster(),
                    detail: di,
                    fat,
                }))
            } else if di.is_dir() {
                let fat = fat::FAT::new(di.cluster(),
                                   self.device,
                                   self.bpb.fat1());
                Ok(Arc::new(dir::Dir::<SDCard> {
                    device: self.device,
                    bpb: self.bpb,
                    detail: di,
                    fat,
                }))
            } else {
                Err(FileErr::InodeNotChild)
            }
        }
    }

    fn get_dirent(&self, offset: usize, dirent: &mut LinuxDirent) -> Result<usize, FileErr> {
        let dent_iter = self.direntry_iter();

        let dent = dent_iter.skip(offset).take(1).last();
        if let None = dent {
            return Err(FileErr::InodeEndOfDir)
        }
        let dent = dent.unwrap();
        dirent.d_ino = dent.cluster() as usize;
        dirent.d_type = if dent.is_dir() {
                DT_DIR
            } else if dent.is_file() {
                DT_REG
            } else {
                DT_UNKNOWN
            };
        use core::convert::TryFrom;
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
        if dent.is_lfn() {
            let (buf, len) = dent.get_lfn().unwrap();
            dirent.d_name[0..len].copy_from_slice(&buf[0..len]);
        } else {
            let (buf, len) = dent.get_sfn().unwrap();
            dirent.d_name[0..len].copy_from_slice(&buf[0..len]);
        }
        Ok(1)
    }

    fn create(&self, name: &str, _: FileMode, itype: InodeType) -> Result<Inode, FileErr> {
        let _self = unsafe { (self as *const Self as *mut Self).as_mut().unwrap()};
        match itype {
            InodeType::Directory => {
                _self.create_dir(name)?;
                _self.get_child(name)
            }
            InodeType::File => {
                _self.create_file(name)?;
                _self.get_child(name)
            }
            _ => {
                Err(FileErr::NotDefine)
            }
        }
    }

    fn unlink_child(&self, name: &str, rm_dir: bool) -> Result<usize, FileErr> {
        let _self = unsafe { (self as *const Self as *mut Self).as_mut().unwrap()};
        match _self.delete_file(name) {
            Err(fat32::dir::DirError::NoMatchFile) => {
            }
            Err(e) => {
                return Err(e.into())
            }
            Ok(_) => {
                return Ok(0)
            }
        }
        if rm_dir {
            _self.delete_dir(name)?;
            return Ok(0)
        } else {
            return Err(FileErr::NotDefine)
        }
    }

    fn len(&self) -> usize {
        0
    }
}
