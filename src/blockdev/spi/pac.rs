use core::ops::Deref;
use registers::*;

#[derive(Copy, Clone)]
pub enum SPIDevice {
  QSPI0,
  QSPI1,
  QSPI2,
  Other(usize),
}

impl SPIDevice {
  fn base_addr(&self) -> *const RegisterBlock {
    let a = match self {
      SPIDevice::QSPI0      => 0x10040000usize,
      SPIDevice::QSPI1      => 0x10041000usize,
      SPIDevice::QSPI2      => 0x10050000usize,
      SPIDevice::Other(val) => val.clone(),
    };
    a as *const _
  }
}

impl Deref for SPIDevice {
  type Target = RegisterBlock;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.base_addr() }
  }
}

#[doc = "For more information, see: https://sifive.cdn.prismic.io/sifive/d3ed5cd0-6e74-46b2-a12d-72b06706513e_fu540-c000-manual-v1p4.pdf"]
#[repr(C)]
#[repr(packed)]
pub struct RegisterBlock {
  #[doc = "0x00: Serial clock divisor register"]
  pub sckdiv: SCKDIV,
  #[doc = "0x04: Serial clock mode register"]
  pub sckmode: SCKMODE,
  // unimplemented below
  #[doc = "0x08: Reserved"]
  _reserved0: RESERVED,
  #[doc = "0x0c: Reserved"]
  _reserved1: RESERVED,
  #[doc = "0x10: Chip select ID register"]
  pub csid: CSID,
  #[doc = "0x14: Chip select default register"]
  pub csdef: CSDEF,
  #[doc = "0x18: Chip Select mode register"]
  pub csmode: CSMODE,
  #[doc = "0x1C: Reserved"]
  _reserved2: RESERVED,
  #[doc = "0x20: Reserved"]
  _reserved3: RESERVED,
  #[doc = "0x24: Reserved"]
  _reserved4: RESERVED,
  #[doc = "0x28: Delay control 0 register"]
  pub delay0: DELAY0,
  #[doc = "0x2C: Delay control 1 register"]
  pub delay1: DELAY1,
  #[doc = "0x30: Reserved"]
  _reserved5: RESERVED,
  #[doc = "0x34: Reserved"]
  _reserved6: RESERVED,
  #[doc = "0x38: Reserved"]
  _reserved7: RESERVED,
  #[doc = "0x3C: Reserved"]
  _reserved8: RESERVED,
  #[doc = "0x40: Frame format register"]
  pub fmt: FMT,
  #[doc = "0x44: Reserved"]
  _reserved9: RESERVED,
  #[doc = "0x48: Tx FIFO data register"]
  pub txdata: TXDATA,
  #[doc = "0x4C: Rx FIFO data register"]
  pub rxdata: RXDATA,
  #[doc = "0x50: Tx FIFO watermark"]
  pub txmark: TXMARK,
  #[doc = "0x54: Rx FIFO watermark"]
  pub rxmark: RXMARK,
  #[doc = "0x58: Reserved"]
  _reserved10: RESERVED,
  #[doc = "0x5C: Reserved"]
  _reserved11: RESERVED,
  #[doc = "0x60: SPI flash interface control register"]
  pub fctrl: FCTRL,
  #[doc = "0x64: SPI flash instruction format register"]
  pub ffmt: FFMT,
  #[doc = "0x68: Reserved"]
  _reserved12: RESERVED,
  #[doc = "0x6C: Reserved"]
  _reserved13: RESERVED,
  #[doc = "0x70: SPI flash interrupt enable register"]
  pub ie: IE,
  #[doc = "0x74: SPI flash interrupt pending register"]
  pub ip: IP,
}

mod registers {

  use core::marker::PhantomData;

  pub trait Reset {
    fn reset(&self);
  }

  #[doc = "Universal register structure"]
  #[repr(C)]
  #[repr(packed)]
  pub struct Reg<T: Sized + Clone + Copy, U> {
    value: T,
    p: PhantomData<U>,
  }

  impl<T: Sized + Clone + Copy, U> Reg<T, U> {
    pub fn new(initval: T) -> Self {
      Self { value: initval, p: PhantomData{} }
    }
  }

  impl<T: Sized + Clone + Copy, U> Reg<T, U> {
    fn read(&self) -> T {
      self.value
    }
    fn write(&self, val: T) {
      let ptr = self as *const Self as usize as *mut Self;
      unsafe { (*ptr).value = val; }
    }
  }

  pub struct _RESERVED;
  pub type RESERVED = Reg<u32, _RESERVED>;

  pub struct _SCKDIV;
  pub type SCKDIV = Reg<u32, _SCKDIV>;
  impl Reset for SCKDIV {
    fn reset(&self) {
      self.write(3u32);
    }
  }

  pub struct _SCKMODE;
  pub type SCKMODE = Reg<u32, _SCKMODE>;
  impl Reset for SCKMODE {
    fn reset(&self) {
      self.write(0u32);
    }
  }
  impl SCKMODE {
    pub fn set_phase(&self, phabit: bool) {
      let mut data = self.read();
      data = if phabit {
        data | 0x1u32
      } else {
        data & 0xfffeu32
      };
      self.write(data);
    }
    pub fn set_polarity(&self, polbit: bool) {
      let mut data = self.read();
      data = if polbit {
        data | 0x2u32
      } else {
        data & 0xfffdu32
      };
      self.write(data);
    }
  }

  /* TODO: unimplemented below */
  pub struct _CSID;
  pub type CSID = Reg<u32, _SCKMODE>;

  pub struct _CSDEF;
  pub type CSDEF = Reg<u32, _CSDEF>;

  pub struct _CSMODE;
  pub type CSMODE = Reg<u32, _CSMODE>;

  pub struct _DELAY0;
  pub type DELAY0 = Reg<u32, _DELAY0>;

  pub struct _DELAY1;
  pub type DELAY1 = Reg<u32, _DELAY1>;

  pub struct _FMT;
  pub type FMT = Reg<u32, _FMT>;

  pub struct _TXDATA;
  pub type TXDATA = Reg<u32, _TXDATA>;

  pub struct _RXDATA;
  pub type RXDATA = Reg<u32, _RXDATA>;

  pub struct _TXMARK;
  pub type TXMARK = Reg<u32, _TXMARK>;

  pub struct _RXMARK;
  pub type RXMARK = Reg<u32, _RXMARK>;

  pub struct _FCTRL;
  pub type FCTRL = Reg<u32, _FCTRL>;

  pub struct _FFMT;
  pub type FFMT = Reg<u32, _FFMT>;

  pub struct _IE;
  pub type IE = Reg<u32, _IE>;

  pub struct _IP;
  pub type IP = Reg<u32, _IP>;
}

fn main() {
}
