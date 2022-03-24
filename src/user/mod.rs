pub const APP_NUM: usize = 2;

static HELLO_WORLD: &'static [u8] = include_bytes!("hello_world");
static LOOP10: &'static [u8] = include_bytes!("loop10");

lazy_static! {
    pub static ref APP: [&'static [u8]; APP_NUM] = [LOOP10, HELLO_WORLD];
}
