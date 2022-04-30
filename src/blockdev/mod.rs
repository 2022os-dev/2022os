mod spi_sd;

use alloc::sync::Arc;

pub trait BlockDevice : Send + Sync {
  fn read_block(&self, block_id: usize, buf: &mut [u8]);
  fn write_block(&self, block_id: usize, buf: &[u8]);
}

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<spi_sd::SDCardWrapper> = Arc::new(spi_sd::SDCardWrapper::new());
}

pub fn init_sdcard() {
  BLOCK_DEVICE.init();
}

pub fn read_block(block_id: usize, buf: &mut [u8]) {
  BLOCK_DEVICE.read_block(block_id, buf);
}
pub fn write_block(block_id: usize, buf: &[u8]) {
  BLOCK_DEVICE.write_block(block_id, buf);
}
