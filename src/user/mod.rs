pub const APP_NUM: usize = 1;

static HELLO_BIN: &'static [u8] = include_bytes!("00hello_world");

lazy_static! {
    pub static ref APP: [&'static [u8]; APP_NUM] = [HELLO_BIN];
}
