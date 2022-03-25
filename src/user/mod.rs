pub const APP_NUM: usize = 3;

static HELLO_WORLD: &'static [u8] = include_bytes!("bin/hello_world");
static LOOP10: &'static [u8] = include_bytes!("bin/loop10");
static GET_PID: &'static [u8] = include_bytes!("bin/get_pid");

lazy_static! {
    pub static ref APP: [&'static [u8]; APP_NUM] = [LOOP10, HELLO_WORLD, GET_PID];
}
