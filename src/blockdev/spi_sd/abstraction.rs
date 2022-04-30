use super::pac as pac;
use pac::*;

const TLCLK_FREQ: u32 = 500000000;

pub struct SPIImpl {
  spi: pac::SPIDevice,
}

pub trait SPIActions {
  fn init(&self);
  fn configure(
    &self,
    // work_mode: work_mode,  // changes clock mode (sckmode) support master mode only
    use_lines: u8,  // SPI data line width, 1,2,4 allowed
    data_bit_length: u8,  // bits per word, basically 8
    msb_first: bool,  // endianness
  );
  fn switch_cs(&self, enable: bool, csid: u32);
  fn set_clk_rate(&self, spi_clk: u32) -> u32;
  fn recv_data(&self, chip_select: u32, rx: &mut [u8]);
  fn send_data(&self, chip_select: u32, tx: &[u8]);
  // fn fill_data(&self, chip_select: u32, value: u32, tx_len: usize);
}

impl SPIImpl {
  pub fn new(spi: pac::SPIDevice) -> Self {
    Self { spi }
  }
}


impl SPIImpl {
  fn tx_enque(&self, data: u8) {
    if self.spi.txdata.is_full() {
      println!("[kernel] spi warning: overwritting existing data to transmit");
    }
    self.spi.txdata.write(data as u32);
  }

  fn rx_deque(&self) -> u8 {
    /*if self.spi.rxdata.is_empty() {
      println!("spi warning: attempting to read empty fifo");
    }*/
    let result: u32 = self.spi.rxdata.read();
    if (result & (1u32<<31)) != 0 {
      println!("[kernel] spi warning: attempting to read empty fifo");
    }
    result as u8
  }

  fn rx_wait(&self) {
    while !self.spi.ip.receive_pending() {
      // loop
    }
  }

  fn tx_wait(&self) {
    while !self.spi.ip.transmit_pending() {
      // loop
    }
  }
}

impl SPIActions for SPIImpl {
  // This function references spi-sifive.c:sifive_spi_init()
  fn init(&self) { 
    let spi = self.spi;
    
    //  Watermark interrupts are disabled by default
    spi.ie.set_transmit_watermark(false);
    spi.ie.set_receive_watermark(false);
    
    // Default watermark FIFO threshold values
    spi.txmark.write(1u32);
    spi.rxmark.write(0u32);

    // Set CS/SCK Delays and Inactive Time to defaults
    spi.delay0.set_cssck(1);
    spi.delay0.set_sckcs(1);
    spi.delay1.set_intercs(1);
    spi.delay1.set_interxfr(0);

    // Exit specialized memory-mapped SPI flash mode
    spi.fctrl.set_flash_mode(false);
  }

  fn configure(&self, use_lines: u8, data_bit_length: u8, msb_first: bool) {
    let spi = self.spi;
    // bit per word
    spi.fmt.set_len(data_bit_length);
    // switch protocol (QSPI, DSPI, SPI)
    let fmt_proto = match use_lines {
      4u8 => Protocol::Quad,
      2u8 => Protocol::Dual,
      _   => Protocol::Single,
    };
    spi.fmt.switch_protocol(fmt_proto);
    // endianness
    spi.fmt.set_endian(msb_first);
    // clock mode: from sifive_spi_prepare_message
    spi.sckmode.reset();
  }

  fn switch_cs(&self, enable: bool, csid: u32) {
    // manual cs
    self.spi.csmode.switch_csmode(if enable { Mode::HOLD } else { Mode::OFF } );
    self.spi.csid.write(csid);
  }

  fn set_clk_rate(&self, spi_clk: u32) -> u32 {
    // calculate clock rate
    let div = TLCLK_FREQ / 2u32 / spi_clk - 1u32;
    self.spi.sckdiv.write(div);
    TLCLK_FREQ / 2 / div
  }

  fn recv_data(&self, chip_select: u32, rx_buf: &mut [u8]) {
    // direction
    self.spi.fmt.set_direction(false);
    // select the correct device
    self.spi.csid.write(chip_select);

    let len = rx_buf.len();
    let mut remaining = len;

    while remaining != 0usize {
      // words need to be transferred in a single round
      let n_words = if 8usize < remaining { 8 } else { remaining };
      // enqueue n_words junk for transmission
      for i in 0..n_words {
        self.tx_enque(0xffu8);
      }
      // set watermark
      self.spi.rxmark.write(n_words as u32 - 1);
      // wait for spi
      // TODO implement yielding in wait
      self.rx_wait();
      // read out all data from rx fifo
      for i in 0..n_words {
        rx_buf[len - remaining] = self.rx_deque();
        remaining = remaining - 1;
      }
    }
  }

  fn send_data(&self, chip_select: u32, tx_buf: &[u8]) {
    // direction
    self.spi.fmt.set_direction(true);
    // select the correct device
    self.spi.csid.write(chip_select);
    
    let len = tx_buf.len();
    let mut remaining = len;
    while remaining != 0usize {
      // words need to be transferred in a single round
      let n_words = if 8usize < remaining { 8 } else { remaining };
      // set watermark
      self.spi.txmark.write(1);
      // wait for spi
      // TODO implement yielding in wait
      self.tx_wait();
      // enque spi
      for _ in 0..n_words {
        self.tx_enque(tx_buf[len - remaining]);
        remaining = remaining - 1;
      }
    }
  }
}
