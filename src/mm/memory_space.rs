use super::address::*;
use super::PTEFlag;
use super::KALLOCATOR;
use crate::config::*;
use crate::process::TrapFrame;
use crate::trap::{__alltraps, __restore};
use crate::vfs::*;
use alloc::collections::BTreeMap;
use alloc::vec;
use core::cmp::min;
use core::mem::size_of;
use core::mem::transmute;
use core::ops::Range;
use core::slice;

pub type Segments = BTreeMap<PageNum, (PageNum, PTEFlag)>;

// 表示进程的内存空间, 包括代码和数据段、一个用于上下文切换的trapframe页、用户栈、堆指针和堆内存
pub struct MemorySpace {
    // 进程的入口
    entry: usize,
    // 保存数据段和代码段、堆内存等映射信息
    pub segments: Segments,
    // 用于上下文切换的trapframe
    pub trapframe: PageNum,
    // 用户栈的物理页面，目前用户态的栈大小为一个页面
    pub user_stack: PageNum,
    // 进程的programe_break指针，用于分配堆内存
    prog_break: VirtualAddr,
    // 堆内存映射的最高的一个页面
    prog_high_page: PageNum,
}

impl MemorySpace {
    pub fn new() -> Self {
        let tf = KALLOCATOR.lock().kalloc();
        let stack = KALLOCATOR.lock().kalloc();
        Self {
            entry: 0,
            segments: BTreeMap::new(),
            trapframe: tf,
            user_stack: stack,
            prog_break: VirtualAddr(0),
            prog_high_page: PageNum(0),
        }
    }

    // 将programe_break指针向上移动inc，为用户态程序分配堆空间
    pub fn prog_sbrk(&mut self, inc: usize) -> VirtualAddr {
        // 首次调用sbrk，找到当前映射的最高页面
        if self.prog_break.0 == 0 {
            let maxvpage = self
                .segments
                .keys()
                .into_iter()
                .max_by(|lvp, rvp| lvp.0.cmp(&rvp.0));
            if let None = maxvpage {
                panic!("Can't found max vpage in segments");
            }
            // 将当前最高页面的高2个页面作为堆
            self.prog_break = (*maxvpage.unwrap() + 2).offset(0);
            self.prog_high_page = *maxvpage.unwrap() + 1;
        }
        // 返回之前的prog_break指针
        let retva = self.prog_break;
        while self.prog_break + inc > self.prog_high_page.offset(PAGE_SIZE) {
            if self.segments().contains_key(&(self.prog_high_page + 1)) {
                panic!(
                    "duplicated program break page 0x{:x}",
                    self.prog_high_page.offset(0).0
                );
            }
            self.segments.insert(
                self.prog_high_page + 1,
                (
                    KALLOCATOR.lock().kalloc(),
                    PTEFlag::R | PTEFlag::W | PTEFlag::U,
                ),
            );
            self.prog_high_page = self.prog_high_page + 1;
        }
        self.prog_break = self.prog_break + inc;
        retva
    }

    pub fn prog_brk(&mut self, va: VirtualAddr) -> Result<(), ()> {
        self.prog_break = va;
        Ok(())
    }

    pub fn trampoline_page() -> PageNum {
        PageNum::highest_page()
    }

    // Return (alltraps, restore)
    pub fn trampoline_entry() -> (usize, usize) {
        let alltraps = Self::trampoline_page().offset(0);
        let restore = alltraps + (__restore as usize - __alltraps as usize);
        (alltraps.0, restore.0)
    }

    pub fn trapframe(&mut self) -> &mut TrapFrame {
        let phys = self.trapframe.offset_phys(0).0;
        unsafe { <*mut TrapFrame>::from_bits(phys).as_mut().unwrap() }
    }

    pub fn get_stack_sp() -> VirtualAddr {
        VirtualAddr(USER_STACK_PAGE) + PAGE_SIZE
    }

    pub fn get_stack_start() -> VirtualAddr {
        VirtualAddr(USER_STACK_PAGE)
    }

    /*
    pub fn copy_from_user(&mut self, src: VirtualAddr,  dst: &mut [u8]) {
        let pte = current_hart_pgtbl().walk(src, false);
        if !pte.is_valid() {
            panic!("")
        }
        log!(debug "Copy 0x{:x} from user 0x{:x} {:?}", src.0, pte.ppn().offset(src.page_offset()).0, pte.flags());
        let pa = PhysAddr(src.0);
        pa.read(unsafe { core::slice::from_raw_parts_mut(dst.as_mut_ptr(), dst.len()) });
    }

    pub fn copy_to_user(&mut self, dst: VirtualAddr, src: &[u8]) {
        let pte = current_hart_pgtbl().walk(dst, false);
        if !pte.is_valid() {
            panic!("")
        }
        log!(debug "Copy 0x{:x} to user 0x{:x} {:?}", dst.0, pte.ppn().offset(dst.page_offset()).0, pte.flags());
        let mut dst = PhysAddr(dst.0);
        let dst = dst.as_slice_mut(src.len());
        dst.copy_from_slice(src);
    }
    */

    // 完全复制一个内存空间，分配新的物理页面，将原页面的内容复制到新页面。用于fork
    pub fn copy(&self) -> Self {
        let mut mem = MemorySpace::new();
        mem.entry = self.entry;
        for (vpage, (page, flags)) in self.segments().iter() {
            let newpage = KALLOCATOR.lock().kalloc();
            let mut phys = newpage.offset_phys(0);
            phys.write(page.offset_phys(0).as_slice(PAGE_SIZE));
            mem.segments.insert(*vpage, (newpage, *flags));
        }
        let newpage = KALLOCATOR.lock().kalloc();
        let mut phys = newpage.offset_phys(0);
        phys.write(self.user_stack.offset_phys(0).as_slice(PAGE_SIZE));
        mem.user_stack = newpage;

        let newpage = KALLOCATOR.lock().kalloc();
        let mut phys = newpage.offset_phys(0);
        phys.write(self.trapframe.offset_phys(0).as_slice(PAGE_SIZE));
        mem.trapframe = newpage;
        mem
    }

    // 从elf中加载MemorySpace, ELF存储于data中
    pub fn from_elf_memory(data: &[u8]) -> Result<Self, ()> {
        let elf = elf_parser::Elf64::from_bytes(data);
        if elf.is_err() {
            return Err(())
        }
        let elf = elf.unwrap();
        let mut ms = Self::new();
        for phdr in elf.phdr_iter() {
            let start_va = VirtualAddr(phdr.p_vaddr as usize);
            let end_va = VirtualAddr((phdr.p_vaddr + phdr.p_memsz) as usize);
            let map_perm = MemorySpace::get_pte_flags_from_phdr_flags(phdr.p_flags) | PTEFlag::U;
            ms.add_area_data_each_byte(
                start_va..end_va,
                map_perm | PTEFlag::V,
                &data[phdr.p_offset as usize..(phdr.p_offset+phdr.p_filesz) as usize]
            );

        }
        ms.set_entry_point(elf.entry_point() as usize);
        let sp = Self::get_stack_sp().0;
        ms.trapframe().init(sp, elf.entry_point() as usize);
        return Ok(ms)
    }

    // 从elf中加载MemorySpace, ELF为Inode对于的文件
    // 按需读取文件，不需要将文件全部读入内存
    pub fn from_elf_inode(inode: Inode) -> Result<Self, FileErr> {
        let ehdr_size = size_of::<elf_parser::Elf64Ehdr>();
        let mut elf = vec![0; ehdr_size];
        if let Ok(_) = inode.read_offset(0, elf.as_mut_slice()) {
            let elf = elf_parser::Elf64::from_bytes(elf.as_slice());
            if let Err(e) = elf {
                println!("{:?}", e);
                return Err(FileErr::NotDefine)
            }
            let elf = elf.unwrap();
            let mut ms = Self::new();
            // map programe
            for i in 0..elf.phdr_num() {
                let inode_offset = elf.ehdr().e_phoff + i as u64 * elf.ehdr().e_phentsize as u64;
                let mut phdr = vec![0; size_of::<elf_parser::Elf64Phdr>()];
                inode.read_offset(inode_offset as usize, phdr.as_mut_slice())?;
                let phdr = unsafe {transmute::<*const u8, &elf_parser::Elf64Phdr>(phdr.as_ptr()) };
                // Not LOAD
                if phdr.p_type != 1 {
                    continue;
                }
                let mut data = vec![0; phdr.p_filesz as usize];
                inode.read_offset(phdr.p_offset as usize, data.as_mut_slice())?;
                let start_va = VirtualAddr(phdr.p_vaddr as usize);
                let end_va = VirtualAddr((phdr.p_vaddr + phdr.p_memsz) as usize);
                let map_perm = MemorySpace::get_pte_flags_from_phdr_flags(phdr.p_flags) | PTEFlag::U;
                ms.add_area_data_each_byte(
                    start_va..end_va,
                    map_perm | PTEFlag::V,
                    data.as_slice(),
                );
            }
            ms.set_entry_point(elf.entry_point() as usize);
            let sp = Self::get_stack_sp().0;
            ms.trapframe().init(sp, elf.entry_point() as usize);
            return Ok(ms)
        }
        Err(FileErr::NotDefine)
    }

    pub fn segments(&self) -> &Segments {
        &self.segments
    }

    /*
    // 映射一段0内存
    fn add_area_zero(&mut self, area: Range<VirtualAddr>, flags: PTEFlag) {
        // Fixme: enhance performance
        let (start, end) = (area.start, area.end);
        log!(debug "[kernel] Maping zero page 0x{:x} - 0x{:x}", start.0, end.0);
        let start = start.floor();
        let end = end.floor();
        for vpage in start.page()..end.page() {
            let vpage: PageNum = vpage.into();
            let page ;
            if self.segments.contains_key(&vpage) {
                panic!("duplicated segment 0x{:x}", vpage.page());
            } else {
                page = KALLOCATOR.lock().kalloc();
                self.segments.insert(vpage,(page, flags));
            }
            page.offset_phys(0).write_bytes(0, PAGE_SIZE);
        }
    }
    */

    // 将data中的数据映射到area
    fn add_area_data_each_byte(&mut self, area: Range<VirtualAddr>, flags: PTEFlag, data: &[u8]) {
        let mut start = area.start;
        let end = area.end;
        let start_page = start.floor();
        let end_page = end.ceil();
        let total = data.len();
        let mut wroten = 0;
        for vpage in start_page.page()..end_page.page() {
            let vpage: PageNum = vpage.into();
            let page;
            if self.segments.contains_key(&vpage) {
                // 多个段可能在同一页，将所有段的flags 或
                page = self.segments[&vpage].0;
                self.segments.get_mut(&vpage).unwrap().1 |= flags;
            } else {
                page = KALLOCATOR.lock().kalloc();
                self.segments.insert(vpage, (page, flags));
            }
            let size = min(PAGE_SIZE - start.page_offset(), total - wroten);
            if size == 0 {
                break;
            }
            page.offset_phys(start.page_offset())
                .write(unsafe { slice::from_raw_parts(&data[wroten], size) });
            wroten += size;
            start = start + size;
        }
    }

    // Helper functions
    fn set_entry_point(&mut self, entry: usize) {
        self.entry = entry
    }

    fn get_pte_flags_from_phdr_flags(flags: u32) -> PTEFlag {
        let mut pte = PTEFlag::empty();
        if flags & 0x4 != 0 {
            pte |= PTEFlag::R;
        }
        if flags & 0x2 != 0 {
            pte |= PTEFlag::W;
        }
        if flags & 0x1 != 0 {
            pte |= PTEFlag::X;
        }
        pte

    }
}

impl Drop for MemorySpace {
    fn drop(&mut self) {
        KALLOCATOR.lock().kfree(self.user_stack);
        KALLOCATOR.lock().kfree(self.trapframe);
        for (_, (page, _)) in self.segments.iter() {
            KALLOCATOR.lock().kfree(*page);
        }
    }
}
