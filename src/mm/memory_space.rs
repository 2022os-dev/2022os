use super::address::*;
use super::PTEFlag;
use super::Pgtbl;
use super::KALLOCATOR;
use crate::config::*;
use crate::process::TrapFrame;
use crate::trap::{__alltraps, __restore};
use core::ops::Range;
use alloc::vec::Vec;
use xmas_elf::ElfFile;

pub struct MemorySpace {
    pub pgtbl: Pgtbl,
    entry: usize,
    // 保存elf中可加载的段
    segments: Vec<(VirtualAddr, VirtualAddr)>
}

impl MemorySpace {
    pub fn new(pgtbl: Option<Pgtbl>) -> Self {
        Self {
            pgtbl: pgtbl.unwrap_or(Pgtbl::new()),
            entry: 0,
            segments: Vec::new()
        }
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
        let pa = self.pgtbl.walk(src, false).ppn()
            .offset_phys(src.page_offset());
        pa.read(unsafe { core::slice::from_raw_parts_mut(dst.as_mut_ptr(), dst.len()) });
    }

    pub fn copy_to_user(&mut self, dst: VirtualAddr, src: &[u8]) {
        let mut dst = self
            .pgtbl
            .walk(dst, false)
            .ppn()
            .offset_phys(dst.page_offset());
        let dst = dst.as_slice_mut(src.len());
        dst.copy_from_slice(src);
    }

    pub fn copy(&self) -> Self {
        MemorySpace {
            pgtbl: self.pgtbl.copy(true),
            entry: self.entry,
            segments: self.segments.clone()
        }
    }

    pub fn get_stack_sp() -> VirtualAddr{
        VirtualAddr(HIGH_MEMORY_SPACE)
    }

    pub fn get_stack_start() -> VirtualAddr {
        VirtualAddr(HIGH_MEMORY_SPACE - USER_STACK_SIZE)
    }

    pub fn from_elf(data: &[u8]) -> Self {
        let mut space = Self {
            pgtbl: Pgtbl::new(),
            entry: 0,
            segments: Vec::new()
        };
        let elf = ElfFile::new(data).unwrap();
        let elf_header = elf.header;
        MemorySpace::validate_elf_header(elf_header);
        space.set_entry_point(elf_header.pt2.entry_point() as usize);
        space.map_elf_program_table(&elf);
        space.map_user_stack();
        space
    }

    pub fn entry(&self) -> usize {
        self.entry
    }

    pub fn segments(&self) -> &Vec<(VirtualAddr, VirtualAddr)> {
        &self.segments
    }

    pub fn unmap_segments(&mut self) {
        for (start, end) in self.segments.iter() {
            for i in start.floor().page()..end.ceil().page() {
                self.pgtbl.unmap(i.into(), true);
            }
        }
        self.segments.clear();
    }

    // Mapping api
    fn map_elf_program_table(&mut self, elf: &ElfFile) {
        log!(debug "Maping program section");
        let ph_count = elf.header.pt2.ph_count();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va = VirtualAddr(ph.virtual_addr() as usize);
                let end_va = VirtualAddr((ph.virtual_addr() + ph.mem_size()) as usize);
                let map_perm = MemorySpace::get_pte_flags_from_ph_flags(ph.flags(), PTEFlag::U);
                self.map_area_data_each_byte(
                    start_va..end_va,
                    map_perm | PTEFlag::V,
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
                self.segments.push((start_va, end_va));
            }
        }
    }

    fn map_user_stack(&mut self) {
        self.map_area_zero(
            Self::get_stack_start()..Self::get_stack_sp(),
            PTEFlag::U | PTEFlag::R | PTEFlag::W,
        );
    }

    pub fn unmap_user_stack(&mut self) {
        self.pgtbl.unmap_pages( Self::get_stack_start().floor()..Self::get_stack_sp().ceil(), true);
    }

    fn map_area_zero(&mut self, area: Range<VirtualAddr>, flags: PTEFlag) {
        // Fixme: enhance performance
        let (start, end) = (area.start, area.end);
        log!(debug "[kernel] Maping zero page 0x{:x} - 0x{:x}", start.0, end.0);
        let start = start.floor();
        let end = end.floor();
        for page in start.page()..end.page() {
            let page: PageNum = page.into();
            let pte = self.pgtbl.walk(page.offset(0), true);
            if !pte.is_valid() {
                let page = KALLOCATOR.lock().kalloc();
                pte.set_ppn(page);
                pte.set_flags(flags | PTEFlag::V);
            }
            PhysAddr::from(pte.ppn().offset(0)).write_bytes(0, PAGE_SIZE);
        }
    }

    fn map_area_data_each_byte(&mut self, area: Range<VirtualAddr>, flags: PTEFlag, data: &[u8]) {
        let start = area.start;
        let end = area.end;
        log!(debug "[kernel] Maping data page 0x{:x} - 0x{:x}, {:?}", start.0, end.0, flags);
        for va in start.0..end.0 {
            let pte = self.pgtbl.walk(VirtualAddr(va), true);
            if !pte.is_valid() {
                let page = KALLOCATOR.lock().kalloc();
                pte.set_ppn(page);
                pte.set_flags(flags | PTEFlag::V);
            }
            PhysAddr::from(pte.ppn().offset(va % PAGE_SIZE)).write_bytes(data[va - start.0], 1);
        }
    }

    pub fn map_trapframe(&mut self) {
        let mut trapframe = KALLOCATOR.lock().kalloc().offset_phys(0);
        let tf :&mut TrapFrame = trapframe.as_mut();
        tf.init(self);
        self.pgtbl.map(
            Self::trapframe_page(),
            trapframe.floor(),
            PTEFlag::R | PTEFlag::W | PTEFlag::V,
        );
    }

    pub fn unmap_trapframe(&mut self) {
        self.pgtbl.unmap(Self::trapframe_page(), true);
    }

    pub fn map_trampoline(&mut self) {
        let page = MemorySpace::trampoline_page();
        let pn = KALLOCATOR.lock().kalloc();
        self.pgtbl
            .map(page, pn, PTEFlag::R | PTEFlag::X | PTEFlag::V);
        pn.offset_phys(0).write(unsafe {
            core::slice::from_raw_parts(
                crate::trap::__alltraps as *const u8,
                crate::trap::trampoline as usize - crate::trap::__alltraps as usize,
            )
        });
    }

    pub fn unmap_trampoline(&mut self, do_free: bool) {
        self.pgtbl
            .unmap(Self::trampoline_page(), do_free);;
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
