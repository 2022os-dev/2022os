pub const APP_NUM: usize = 1;

static USER_BIN: &'static [u8] = include_bytes!("user_app");

lazy_static! {
    pub static ref APP: [&'static [u8]; APP_NUM] = [USER_BIN];
}
