use crate::config::*;
use core::convert::{AsMut, AsRef};
use core::ops::{Add, Sub};
use core::panic;
use core::slice;

/**
 * VirtualAddr: 虚拟地址
 */
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct VirtualAddr(pub usize);

impl VirtualAddr {
    pub fn floor(&self) -> PageNum {
        PageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PageNum {
        PageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 % PAGE_SIZE
    }
}

impl Add<usize> for VirtualAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for VirtualAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl From<PhysAddr> for VirtualAddr {
    fn from(pa: PhysAddr) -> Self {
        // 用户态与内核态使用同一个页表后可以直接转换
        Self(pa.0)
    }
}

impl From<PageNum> for VirtualAddr {
    fn from(page_num: PageNum) -> Self {
        if page_num.0 > (1 << PAGE_TABLE_LEVEL * SV39_VPN_BIT - 1) {
            return VirtualAddr(
                (page_num.0 | (usize::max_value()) << (PAGE_TABLE_LEVEL * SV39_VPN_BIT))
                    << PAGE_OFFSET_BIT,
            );
        }
        VirtualAddr(page_num.0 << PAGE_OFFSET_BIT)
    }
}

/**
 * PhysAddr: 物理地址
 */
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
    #![allow(unused)]
    pub fn floor(&self) -> PageNum {
        PageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PageNum {
        PageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 % PAGE_SIZE
    }
    pub fn write(&mut self, buf: &[u8]) {
        unsafe {
            (self.0 as *const u8 as *mut u8).copy_from(buf.as_ptr(), buf.len());
        }
    }
    pub fn write_bytes(&mut self, byte: u8, len: usize) {
        unsafe { (self.0 as *const u8 as *mut u8).write_bytes(byte, len) }
    }
    pub fn read(&self, buf: &mut [u8]) {
        unsafe {
            (self.0 as *const u8).copy_to(buf.as_mut_ptr(), buf.len());
        }
    }

    pub fn as_slice(&self, len: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0 as *const u8, len) }
    }

    pub fn as_slice_mut(&mut self, len: usize) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.0 as *mut u8, len) }
    }
}

impl<T> AsRef<T> for PhysAddr {
    fn as_ref(&self) -> &T {
        unsafe {
            <*const T>::from_bits(self.0)
                .as_ref()
                .unwrap_or_else(|| panic!("PhysAddr as ref invalid: 0x{:x}", self.0))
        }
    }
}

impl<T> AsMut<T> for PhysAddr {
    fn as_mut(&mut self) -> &mut T {
        unsafe {
            <*mut T>::from_bits(self.0)
                .as_mut()
                .unwrap_or_else(|| panic!("PhysAddr as mut invalid: 0x{:x}", self.0))
        }
    }
}

impl From<VirtualAddr> for PhysAddr {
    fn from(addr: VirtualAddr) -> Self {
        Self(addr.0)
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl<T: Sized> From<&T> for PhysAddr {
    fn from(reference: &T) -> Self {
        PhysAddr(<*const T>::to_bits(reference))
    }
}

impl From<PageNum> for PhysAddr {
    fn from(page_num: PageNum) -> Self {
        Self(page_num.offset(0).0)
    }
}

/**
 * PageNum: 页号
 */

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct PageNum(pub usize);

impl From<usize> for PageNum {
    fn from(u: usize) -> Self {
        PageNum(u)
    }
}

/// SV39:
/// ------------------------------
/// 0000... | vpn2 | vpn1 | vpn0 |
/// ------------------------------
impl PageNum {
    pub fn vpn_block_sv39(&self, level: usize) -> usize {
        if level >= PAGE_TABLE_LEVEL {
            panic!("Page Table Level larger than {}", PAGE_TABLE_LEVEL);
        }
        let vpn = self.0 >> (SV39_VPN_BIT * level);
        vpn & ((1 << SV39_VPN_BIT) - 1)
    }
    pub fn offset(&self, off: usize) -> VirtualAddr {
        VirtualAddr::from(self.clone()) + off
    }

    pub fn offset_phys(&self, off: usize) -> PhysAddr {
        PhysAddr::from(self.clone()) + off
    }

    pub fn page(&self) -> usize {
        self.0
    }
    pub const fn highest_page() -> Self {
        PageNum((1 << (PAGE_TABLE_LEVEL * SV39_VPN_BIT)) - 1)
    }
}

impl Add<usize> for PageNum {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        (self.0 + rhs).into()
    }
}
impl Sub<usize> for PageNum {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        (self.0 - rhs).into()
    }
}
