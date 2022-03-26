pub const APP_NUM: usize = 4;

static HELLO_WORLD: &'static [u8] = include_bytes!("bin/hello_world");
static LOOP10: &'static [u8] = include_bytes!("bin/loop10");
static GET_PID: &'static [u8] = include_bytes!("bin/get_pid");
static SYS_WAIT4 : &'static [u8] = include_bytes!("bin/sys_wait4");

lazy_static! {
    pub static ref APP: [&'static [u8]; APP_NUM] = [LOOP10, HELLO_WORLD, GET_PID, SYS_WAIT4];
}
