#!derive[Copy, Clone]
pub enum SPIDevice {
  QSPI0,
  QSPI1,
  QSPI2,
  Other(usize),
}

fn spi_base_addr(dev: SPIDevice) -> usize {
  match dev {
    SPIDevice::QSPI0      => 0x10400000usize,
    SPIDevice::QSPI1      => 0x0usize,
    SPIDevice::QSPI2      => 0x0usize,
    SPIDevice::Other(val) => val,
  }
}

fn spi_raw_write(dev: SPIDevice, reg: usize, val: u32) {
  let ptr: *mut u32 = (spi_base_addr(dev) + reg) as *mut u32;
  unsafe { *ptr = val; }
}

fn spi_raw_read(dev: SPIDevice, reg: usize) -> u32 {
  let ptr: *mut u32 = (spi_base_addr(dev) + reg) as *mut u32;
  unsafe { *ptr }
}

