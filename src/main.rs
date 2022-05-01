// [no_std] Don't use standard library
#![no_std]
// [no_main] Tell compiler we don't need initialization before main() #![no_main]
#![no_main]
#![feature(naked_functions)]
// [global_asm] allow include an assemble file
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(ptr_to_from_bits)]
#![feature(const_trait_impl)]

use crate::{
    clock::clock_init,
    process::cpu::{hart_enable_timer_interrupt, init_hart},
};
use core::arch::asm;

#[macro_use]
mod lang_items;
mod sbi;

#[macro_use]
mod console;

mod blockdev;
mod entry;
mod heap;
mod link_syms;
mod mm;
mod process;
mod task;
mod trap;
mod user;

mod clock;
mod config;
mod vfs;

#[macro_use]
extern crate lazy_static;
extern crate alloc;
extern crate buddy_system_allocator;
extern crate spin;
#[macro_use]
extern crate bitflags;
extern crate elf_parser;

use mm::*;
use process::cpu::hartid;
use task::*;


/// Clear .bss section
fn clear_bss() {
    (link_syms::sbss as usize..link_syms::ebss as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// 记录启动核
static mut BOOTHART: isize = -1;

// [no_mangle] Turn off Rust's name mangling
#[no_mangle]
extern "C" fn kernel_start() {
    log!("hart":"Booting">"");
    if unsafe { BOOTHART } == -1 {
        unsafe {
            BOOTHART = hartid() as isize;
        };

        clear_bss();

        // 需要在开启虚拟内存之前初始化时钟，
        // 因为内核不会映射时钟配置寄存器
        #[cfg(feature = "init_clock")]
        clock_init();

        heap::init();

        mm::init();

        init_hart();

        blockdev::init_sdcard();



        // Load shell
        #[cfg(not(feature = "batch"))]
        scheduler_load_pcb(MemorySpace::from_elf_memory(user::SHELL).unwrap());

        #[cfg(feature = "batch")]
        for i in user::BATCH.iter() {
            scheduler_load_pcb(MemorySpace::from_elf_memory(i).unwrap());
        }

        #[cfg(feature = "multicore")]
        for i in 1..=4 {
            if hartid() != i {
                sbi::sbi_hsm_hart_start(i, crate::link_syms::skernel as usize, 0);
            }
        }
    } else {
        init_hart();
    }
    trap::init();
    hart_enable_timer_interrupt();
    schedule();
}
extern crate fat32;
extern crate block_device;

#[derive(Copy, Clone)]
struct SDCard {}

impl block_device::BlockDevice for SDCard {
    type Error = ();
    fn read(&self, buf: &mut[u8], address: usize, _number_of_blocks: usize) -> Result<(), Self::Error> {
        blockdev::read_block(address, buf);
        Ok(())
    }
    fn write(&self, buf: &[u8], address: usize, _number_of_blocks: usize) -> Result<(), Self::Error> {
        blockdev::write_block(address, buf);
        Ok(())
    }
}

use alloc::sync::Arc;
lazy_static!{
    static ref SDCARD: SDCard = SDCard {};
    static ref VOLUMN: fat32::volume::Volume<SDCard> = fat32::volume::Volume::new(*SDCARD);
    pub static ref FAT32ROOT: Inode = Arc::new(VOLUMN.root_dir());
}

#[allow(unused)]
fn sd_test() {
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
}

use crate::vfs::*;

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
                FileErr::FdInvalid
            }
            _ => {
                FileErr::NotDefine
            }
        }
    }
}

impl crate::vfs::_Inode for fat32::dir::Dir<'static, SDCard> {
    fn get_child(&self, name: &str) -> Result<Inode, FileErr> {
        let file = self.open_file(name)?;
        Ok(Arc::new(file.clone()))
    }

    fn get_dirent(&self, offset: usize, dirent: &mut LinuxDirent) -> Result<usize, FileErr> {
        /*
        let dir_iter = self.direntry_iter();
        if let None = dir_iter {
            return Err(FileErr::NotDefine)
        }
        let dir_iter = dir_iter.unwrap();
        let ent = dir_iter.skip(offset).take(1).last();
        if let None = ent {
            return Err(FileErr::InodeEndOfDir)
        }
        let ent = ent.unwrap();
        match fat32::tool::sfn_or_lfn(name) {
            fat32::entry::NameType::LFN => {

            }
            fat32::entry::NameType::SFN => {
            }
        }
        */
        Err(FileErr::NotDefine)
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
