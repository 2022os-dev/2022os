mod spi_sd;

pub trait BlockDevice : Send + Sync {
  fn read_block(&self, block_id: usize, buf: &mut [u8]);
  fn write_block(&self, block_id: usize, buf: &[u8]);
}
