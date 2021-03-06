/**
 * Feature: 为分配的页面使用引用计数指针，避免内存泄漏
 */
use super::address::*;
use crate::config::PAGE_SIZE;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref KALLOCATOR: Mutex<Kallocator> = Mutex::new(Kallocator::default());
}

pub struct Kallocator(usize);

impl Default for Kallocator {
    fn default() -> Self {
        Self(0)
    }
}

use core::ops::Range;
impl Kallocator {
    pub fn init(&mut self, pages: Range<PageNum>) {
        log!("kalloc":"init">"0x{:x} - 0x{:x}", pages.start.page(), pages.end.page());
        self.0 = pages.start.page();
        for i in pages.start.page()..pages.end.page() {
            let mut pa: PhysAddr = Into::<PageNum>::into(i).into();
            let pa: &mut usize = pa.as_mut();
            *pa = i + 1;
        }
        let mut pa: PhysAddr = (pages.end - 1).into();
        let pa: &mut usize = pa.as_mut();
        *pa = 0;
    }

    pub fn kalloc(&mut self) -> PageNum {
        if self.0 == 0 {
            panic!("run out of memory");
        }
        let pa: PhysAddr = Into::<PageNum>::into(self.0).into();
        let pa: &usize = pa.as_ref();
        let ret: PageNum = self.0.into();
        self.0 = *pa;
        // REMOVE
        if self.0 == 0 {
            log!("kalloc":"kalloc""warn">"the last page 0x{:x}", ret.page());
        }
        // clear page
        Into::<PhysAddr>::into(ret).write_bytes(0, PAGE_SIZE);
        ret
    }

    pub fn kfree(&mut self, page: PageNum) {
        log!("kalloc":"kfree">"0x{:x}", page.page());
        *(page.offset_phys(0).as_mut()) = self.0;
        self.0 = page.page();
    }
}
