use core::ops::Deref;
use registers::*;

pub use registers::Protocol as Protocol;
pub use registers::Mode as Mode;

pub use registers::Reset as Reset;

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
  #[doc = "0x40: Frame format register ()"]
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
    pub fn read(&self) -> T {
      let ptr: *const T = &self.value;
      unsafe { ptr.read_volatile() }
    }
    pub fn write(&self, val: T) {
      let ptr: *mut T = &self.value as *const _ as usize as *mut T;
      unsafe { ptr.write_volatile(val); }
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
    pub fn set_phase(&self, phasebit: bool) {
      let mut data = self.read();
      data = if phasebit {
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

  pub struct _CSID;
  pub type CSID = Reg<u32, _CSID>;
  impl Reset for CSID {
    fn reset(&self) {
      self.write(0u32);
    }
  }

  pub struct _CSDEF;
  pub type CSDEF = Reg<u32, _CSDEF>;
  impl Reset for CSDEF {
    fn reset(&self) {
      self.write(1u32);
    }
  }
  impl CSDEF {
    pub fn CS_active_low(&self) {
      self.write(1u32);
    }
    pub fn CS_active_high(&self) {
      self.write(0u32);
    }
  }

  pub struct _CSMODE;
  pub type CSMODE = Reg<u32, _CSMODE>;
  impl Reset for CSMODE {
    fn reset(&self) {
      self.write(0u32);
    }
  }
  #[derive(Copy, Clone)]
  pub enum Mode {
    AUTO,
    HOLD,
    OFF,
  }
  impl CSMODE {
    pub fn switch_csmode(&self, mode: Mode) {
      self.write(match mode {
        Mode::AUTO => 0u32,
        Mode::HOLD => 2u32,
        Mode::OFF  => 3u32,
      });
    }
  }

  pub struct _DELAY0;
  pub type DELAY0 = Reg<u32, _DELAY0>;
  impl Reset for DELAY0 {
    fn reset(&self) {
      self.write(0x00010001u32);
    }
  }
  impl DELAY0 {
    pub fn get_cssck(&self) -> u8 {
      let data = self.read();
      data as u8
    }
    pub fn set_cssck(&self, value: u8) {
      let mut data = self.read();
      data = (data & 0xffff0000u32) | value as u32;
      self.write(data);
    }

    pub fn get_sckcs(&self) -> u8 {
      let data = self.read();
      (data >> 16) as u8
    }
    pub fn set_sckcs(&self, value: u8) {
      let mut data = self.read();
      data = (data & 0x0000ffffu32) | ((value as u32) << 16);
      self.write(data);
    }
  }

  pub struct _DELAY1;
  pub type DELAY1 = Reg<u32, _DELAY1>;
  impl Reset for DELAY1 {
    fn reset(&self) {
      self.write(0x00000001u32);
    }
  }
  impl DELAY1 {
    pub fn get_intercs(&self) -> u8 {
      let data = self.read();
      data as u8
    }
    pub fn set_intercs(&self, value: u8) {
      let mut data = self.read();
      data = (data & 0xffff0000u32) | value as u32;
      self.write(data);
    }

    pub fn get_interxfr(&self) -> u8 {
      let data = self.read();
      (data >> 16) as u8
    }
    pub fn set_interxfr(&self, value: u8) {
      let mut data = self.read();
      data = (data & 0x0000ffffu32) | ((value as u32) << 16);
      self.write(data);
    }
  }

  pub struct _FMT;
  pub type FMT = Reg<u32, _FMT>;
  #[derive(Copy, Clone)]
  pub enum Protocol {
    Single,
    Dual,
    Quad,
  }
  impl Reset for FMT {
    fn reset(&self) {
      self.write(0x00080000u32);
    }
  }
  impl FMT {
    pub fn switch_protocol(&self, proto: Protocol) {
      let p = match proto {
        Protocol::Single => 0u32,
        Protocol::Dual   => 1u32,
        Protocol::Quad   => 2u32,
      };
      let r = self.read();
      self.write((r & (!0b011u32)) | p);
    }

    // TODO FIX BITWISE OPS
    pub fn set_endian(&self, msb: bool) {
      let end = if msb { 0b100u32 } else { 0u32 };
      let r = self.read();
      self.write((r & (!0b100u32)) | end);
    }

    pub fn set_direction(&self, tx: bool) {
      let dir = if tx { 0b1000u32 } else { 0u32 };
      let r = self.read();
      self.write((r & (!0b1000u32)) | dir);
    }

    pub fn set_len(&self, frame_size: u8) {
      let fs = (frame_size as u32 & 0x0fu32) << 16;
      let mask = 0xfu32 << 16;
      let r = self.read();
      self.write((r & !mask) | fs);
    }
  }

  pub struct _TXDATA;
  pub type TXDATA = Reg<u32, _TXDATA>;
  impl TXDATA {
    pub fn is_full(&self) -> bool {
      let r = self.read();
      (r & (1u32 << 31)) != 0u32
    }
  }

  pub struct _RXDATA;
  pub type RXDATA = Reg<u32, _RXDATA>;
  impl RXDATA {
    pub fn is_empty(&self) -> bool {
        let r = self.read();
        (r & (1u32 << 31)) != 0u32
    }
  }

  pub struct _TXMARK;
  pub type TXMARK = Reg<u32, _TXMARK>;
  impl Reset for TXMARK {
    fn reset(&self) {
      self.write(0u32);
    }
  }

  pub struct _RXMARK;
  pub type RXMARK = Reg<u32, _RXMARK>;
  impl Reset for RXMARK {
    fn reset(&self) {
      self.write(0u32);
    }
  }

  pub struct _FCTRL;
  pub type FCTRL = Reg<u32, _FCTRL>;
  impl Reset for FCTRL {
    fn reset(&self) {
      self.write(1u32);
    }
  }
  impl FCTRL {
    pub fn set_flash_mode(&self, mmio_enable: bool) {
      let v = if mmio_enable { 1u32 } else { 0u32 };
      self.write(v);
    }
  }

  pub struct _FFMT;
  pub type FFMT = Reg<u32, _FFMT>;
  impl Reset for FFMT {
    fn reset(&self) {
      self.write(0x00030007);
    }
  }
  impl FFMT {
    fn set_cmden(&self, en: bool) {
      let v = if en { 1u32 } else { 0u32 };
      let mask = 1u32;
      let r = self.read();
      self.write((r & !mask) | v);
    }

    fn set_addrlen(&self, len: u8) {
      let v = (len as u32 & 0x7u32) << 1;
      let mask = 0xeu32;
      let r = self.read();
      self.write((r & !mask) | v);
    }

    fn set_padcnt(&self, padcnt: u8) {
      let v = (padcnt as u32 & 0xfu32) << 4;
      let mask = 0xf0u32;
      let r = self.read();
      self.write((r & !mask) | v);
    }

    fn set_cmdproto(&self, proto: u8) {
      let v = (proto as u32 & 0x3u32) << 8;
      let mask = 0x300u32;
      let r = self.read();
      self.write((r & !mask) | v);
    }

    fn set_addrproto(&self, proto: u8) {
      let v = (proto as u32 & 0x3u32) << 10;
      let mask = 0xc00u32;
      let r = self.read();
      self.write((r & !mask) | v);
    }

    fn set_dataproto(&self, proto: u8) {
      let v = (proto as u32 & 0x3u32) << 12;
      let mask = 0x3000u32;
      let r = self.read();
      self.write((r & !mask) | v);
    }

    fn set_cmdcode(&self, code: u8) {
      let v = (code as u32) << 16;
      let mask = 0xfu32 << 16;
      let r = self.read();
      self.write((r & !mask) | v);
    }
    
    fn set_padcode(&self, code: u8) {
      let v = (code as u32) << 24;
      let mask = 0xfu32 << 24;
      let r = self.read();
      self.write((r & !mask) | v);
    }
  }

  pub struct _IE;
  pub type IE = Reg<u32, _IE>;
  impl Reset for IE {
    fn reset(&self) {
      self.write(0u32);
    }
  }
  impl IE {
    pub fn set_transmit_watermark(&self, enable: bool) {
      let en = if enable { 1u32 } else { 0u32 };
      let r = self.read();
      self.write((r & (!1u32)) | en);
    }

    pub fn set_receive_watermark(&self, enable: bool) {
      let en = if enable { 2u32 } else { 0u32 };
      let r = self.read();
      self.write((r & (!2u32)) | en);
    }
  }

  pub struct _IP;
  pub type IP = Reg<u32, _IP>;
  impl IP {
    pub fn transmit_pending(&self) -> bool {
      let r = self.read();
      (r & 1u32) != 0
    }

    pub fn receive_pending(&self) -> bool {
      let r = self.read();
      (r & 2u32) != 0
    }
  }
}



/*
fn main() {
  use registers::*;
  let a = [0u32; 0x50];
  let b = &a as *const _ as usize as *const RegisterBlock;
  let c: &RegisterBlock = unsafe { &*b };

  println!("{:#x}", c.fmt.read());
  c.fmt.switch_protocol(Protocol::Dual);
  println!("{:#x}", c.fmt.read());
}
*/
