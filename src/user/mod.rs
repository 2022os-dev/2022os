#![allow(unused)]
use alloc::boxed::Box;

static HELLO_WORLD: &'static [u8] = include_bytes!("bin/hello_world");
static LOOP10: &'static [u8] = include_bytes!("bin/loop10");
static GET_PID: &'static [u8] = include_bytes!("bin/get_pid");
static SYS_WAIT4: &'static [u8] = include_bytes!("bin/sys_wait4");
static SYS_BRK: &'static [u8] = include_bytes!("bin/sys_brk");
static SYS_KILL: &'static [u8] = include_bytes!("bin/sys_kill");
static FORKBOOM: &'static [u8] = include_bytes!("bin/forkboom");
static SIGNAL_CHLD: &'static [u8] = include_bytes!("bin/signal_chld");
static TIMES: &'static [u8] = include_bytes!("bin/times");
static NANOSLEEP: &'static [u8] = include_bytes!("bin/nanosleep");
static READ: &'static [u8] = include_bytes!("bin/read");
static OPENAT: &'static [u8] = include_bytes!("bin/openat");
static PIPE: &'static [u8] = include_bytes!("bin/pipe");
static DUP : &'static [u8] = include_bytes!("bin/dup");
static MKDIRAT: &'static [u8] = include_bytes!("bin/mkdirat");
static CHDIR: &'static [u8] = include_bytes!("bin/chdir");
static GET_DIRENTS: &'static [u8] = include_bytes!("bin/get_dirents");
static SYS_CLONE: &'static [u8] = include_bytes!("bin/sys_clone");

lazy_static! {
    pub static ref APP: Box<[&'static [u8]]> = Box::new([
        SYS_CLONE
    ]);
}
