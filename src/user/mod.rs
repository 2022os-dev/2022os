use alloc::boxed::Box;

static HELLO_WORLD: &'static [u8] = include_bytes!("bin/hello_world");
static LOOP10: &'static [u8] = include_bytes!("bin/loop10");
static GET_PID: &'static [u8] = include_bytes!("bin/get_pid");
static SYS_WAIT4 : &'static [u8] = include_bytes!("bin/sys_wait4");
static SYS_BRK: &'static [u8] = include_bytes!("bin/sys_brk");

lazy_static! {
    pub static ref APP :Box<[&'static [u8]]> = Box::new([LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,LOOP10,]);
}
