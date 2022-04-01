#![allow(unused)]
use alloc::boxed::Box;

static HELLO_WORLD: &'static [u8] = include_bytes!("bin/hello_world");
static LOOP10: &'static [u8] = include_bytes!("bin/loop10");
static GET_PID: &'static [u8] = include_bytes!("bin/get_pid");
static SYS_WAIT4 : &'static [u8] = include_bytes!("bin/sys_wait4");
static SYS_BRK: &'static [u8] = include_bytes!("bin/sys_brk");
static SYS_KILL: &'static [u8] = include_bytes!("bin/sys_kill");
static FORKBOOM: &'static [u8] = include_bytes!("bin/forkboom");
static SIGNAL_CHLD: &'static [u8] = include_bytes!("bin/signal_chld");
static TIMES: &'static [u8] = include_bytes!("bin/times");
static NANOSLEEP : &'static [u8] = include_bytes!("bin/nanosleep");

lazy_static! {
    pub static ref APP :Box<[&'static [u8]]> = Box::new([SYS_BRK, NANOSLEEP, LOOP10, SYS_WAIT4, SYS_KILL, SIGNAL_CHLD, TIMES, HELLO_WORLD, GET_PID,SYS_BRK, NANOSLEEP, LOOP10, SYS_WAIT4, SYS_KILL, SIGNAL_CHLD, TIMES, HELLO_WORLD, GET_PID,SYS_BRK, NANOSLEEP, LOOP10, SYS_WAIT4, SYS_KILL, SIGNAL_CHLD, TIMES, HELLO_WORLD, GET_PID,SYS_BRK, NANOSLEEP, LOOP10, SYS_WAIT4, SYS_KILL, SIGNAL_CHLD, TIMES, HELLO_WORLD, GET_PID,SYS_BRK, NANOSLEEP, LOOP10, SYS_WAIT4, SYS_KILL, SIGNAL_CHLD, TIMES, HELLO_WORLD, GET_PID,]);
}
