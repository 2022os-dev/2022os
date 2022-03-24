pub trait SPI {
    fn configure(
        &self,
        /*
        work_mode: work_mode,
        frame_format: frame_format,
        data_bit_length: u8,
        endian: u32,
        instruction_length: u8,
        address_length: u8,
        wait_cycles: u8,
        instruction_address_trans_mode: aitm,
        tmod: tmod,
        */
    );
    fn set_clk_rate(&self, spi_clk: u32) -> u32;
    fn recv_data(&self, chip_select: u32, rx: &mut [u8]);
    fn send_data<X: Into<u32> + Copy>(&self, chip_select: u32, tx: &[X]);
    fn fill_data(&self, chip_select: u32, value: u32, tx_len: usize);
}
