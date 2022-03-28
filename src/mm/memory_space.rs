use super::address::*;
use super::PTEFlag;
use super::KALLOCATOR;
use crate::config::*;
use crate::process::TrapFrame;
use crate::process::cpu::current_hart_pgtbl;
use crate::trap::{__alltraps, __restore};
use core::cmp::min;
use core::ops::Range;
use core::slice;
use alloc::collections::BTreeMap;
use xmas_elf::ElfFile;

pub type Segments = BTreeMap<PageNum, (PageNum, PTEFlag)>;

pub struct MemorySpace {
    pub entry: usize,
    // 保存elf中可加载的段
    pub segments: Segments,
    pub trapframe: PageNum,
    pub user_stack: PageNum,
    prog_break: VirtualAddr,
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
            prog_high_page: PageNum(0)
        }
    }

    pub fn prog_sbrk(&mut self, mut inc: usize) -> VirtualAddr {
        if self.prog_break.0 == 0 {
            let maxvpage = self.segments.keys().into_iter().max_by(|lvp, rvp| {
                lvp.0.cmp(&rvp.0)
            });
            if let None = maxvpage {
                panic!("Can't found max vpage in segments");
            }
            self.prog_break = (*maxvpage.unwrap() + 1).offset(0);
            self.prog_high_page = *maxvpage.unwrap();
        }
        let retva = self.prog_break;
        while self.prog_break + inc > self.prog_high_page.offset(PAGE_SIZE) {
            if self.segments().contains_key(&(self.prog_high_page + 1)) {
                panic!("duplicated program break page 0x{:x}", self.prog_high_page.offset(0).0);
            }
            self.segments.insert(self.prog_high_page + 1, (KALLOCATOR.lock().kalloc(), PTEFlag::R | PTEFlag::W | PTEFlag::U));
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

    pub fn trapframe_page() -> PageNum {
        Self::trampoline_page() - 1
    }

    // Return (alltraps, restore)
    pub fn trampoline_entry() -> (usize, usize) {
        let alltraps = Self::trampoline_page().offset(0);
        let restore = alltraps + (__restore as usize - __alltraps as usize);
        (alltraps.0, restore.0)
    }

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

    pub fn trapframe(&mut self) -> &mut TrapFrame {
        let phys = self.trapframe.offset_phys(0).0;
        unsafe {
            <*mut TrapFrame>::from_bits(phys).as_mut().unwrap()
        }
    }

    pub fn get_stack_sp(&self) -> VirtualAddr{
        VirtualAddr(USER_STACK) + PAGE_SIZE
    }

    pub fn get_stack_start(&self) -> VirtualAddr {
        VirtualAddr(USER_STACK)
    }

    pub fn from_elf(data: &[u8]) -> Self {
        let mut space = Self::new();
        let elf = ElfFile::new(data).unwrap();
        let elf_header = elf.header;
        MemorySpace::validate_elf_header(elf_header);
        space.set_entry_point(elf_header.pt2.entry_point() as usize);
        space.add_elf_program_table(&elf);
        let sp = space.get_stack_sp().0;
        let sepc = space.entry;
        space.trapframe().init(sp, sepc);
        space
    }

    pub fn entry(&self) -> usize {
        self.entry
    }

    pub fn segments(&self) -> &Segments {
        &self.segments
    }

    // Mapping api
    fn add_elf_program_table(&mut self, elf: &ElfFile) {
        log!(debug "Maping program section");
        let ph_count = elf.header.pt2.ph_count();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va = VirtualAddr(ph.virtual_addr() as usize);
                let end_va = VirtualAddr((ph.virtual_addr() + ph.mem_size()) as usize);
                let map_perm = MemorySpace::get_pte_flags_from_ph_flags(ph.flags(), PTEFlag::U);
                self.add_area_data_each_byte(
                    start_va..end_va,
                    map_perm | PTEFlag::V,
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
            }
        }
    }

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

    fn add_area_data_each_byte(&mut self, area: Range<VirtualAddr>, flags: PTEFlag, data: &[u8]) {
        let mut start = area.start;
        let end = area.end;
        let start_page = start.floor();
        let end_page = end.ceil();
        let total = data.len();
        let mut wroten = 0;
        log!(debug "[kernel] Maping data page 0x{:x} - 0x{:x}, {:?}", start.0, end.0, flags);
        for vpage in start_page.page()..end_page.page() {
            let vpage :PageNum = vpage.into();
            let page;
            if self.segments.contains_key(&vpage) {
                page = self.segments[&vpage].0;
                self.segments.get_mut(&vpage).unwrap().1 |= flags;
            } else {
                page = KALLOCATOR.lock().kalloc();
                self.segments.insert(vpage,(page, flags));
            }
            let size = min(PAGE_SIZE - start.page_offset(), total - wroten);
            log!(debug "maping data[{}]: 0x{:x} -> 0x{:x}", wroten, size, start.0);
            page.offset_phys(start.page_offset()).write(unsafe {
                slice::from_raw_parts(&data[wroten], size)
            });
            wroten += size;
            start = start + size;
        }
    }

    // Helper functions
    fn validate_elf_header(header: xmas_elf::header::Header) -> bool {
        let magic = header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        true
    }
    fn set_entry_point(&mut self, entry: usize) {
        self.entry = entry
    }
    fn get_pte_flags_from_ph_flags(flags: xmas_elf::program::Flags, init: PTEFlag) -> PTEFlag {
        let mut pte_flags = init;
        if flags.is_read() {
            pte_flags |= PTEFlag::R;
        }
        if flags.is_write() {
            pte_flags |= PTEFlag::W;
        }
        if flags.is_execute() {
            pte_flags |= PTEFlag::X;
        }
        pte_flags
    }
}

impl Drop for MemorySpace {
    fn drop(&mut self) {
        log!(debug "Freeing memory space");
        KALLOCATOR.lock().kfree(self.user_stack);
        KALLOCATOR.lock().kfree(self.trapframe);
        for (_, (page, _)) in self.segments.iter() {
            KALLOCATOR.lock().kfree(*page);
        }
    }
}
