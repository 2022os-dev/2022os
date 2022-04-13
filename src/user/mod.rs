#![allow(unused)]
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub static HELLO_WORLD: &'static [u8] = include_bytes!("bin/hello_world");
pub static LOOP10: &'static [u8] = include_bytes!("bin/loop10");
pub static GET_PID: &'static [u8] = include_bytes!("bin/get_pid");
pub static SYS_WAIT4: &'static [u8] = include_bytes!("bin/sys_wait4");
pub static SYS_BRK: &'static [u8] = include_bytes!("bin/sys_brk");
pub static SYS_KILL: &'static [u8] = include_bytes!("bin/sys_kill");
pub static FORKBOOM: &'static [u8] = include_bytes!("bin/forkboom");
pub static SIGNAL_CHLD: &'static [u8] = include_bytes!("bin/signal_chld");
pub static TIMES: &'static [u8] = include_bytes!("bin/times");
pub static NANOSLEEP: &'static [u8] = include_bytes!("bin/nanosleep");
pub static READ: &'static [u8] = include_bytes!("bin/read");
pub static OPENAT: &'static [u8] = include_bytes!("bin/openat");
pub static PIPE: &'static [u8] = include_bytes!("bin/pipe");
pub static DUP: &'static [u8] = include_bytes!("bin/dup");
pub static MKDIRAT: &'static [u8] = include_bytes!("bin/mkdirat");
pub static CHDIR: &'static [u8] = include_bytes!("bin/chdir");
pub static GET_DIRENTS: &'static [u8] = include_bytes!("bin/get_dirents");
pub static SYS_CLONE: &'static [u8] = include_bytes!("bin/sys_clone");
pub static EXECVE: &'static [u8] = include_bytes!("bin/execve");
pub static SHELL: &'static [u8] = include_bytes!("bin/shell");
pub static FILELINK: &'static [u8] = include_bytes!("bin/filelink");

#[cfg(feature = "batch")]
lazy_static! {
    pub static ref BATCH: Box<[&'static [u8]]> = Box::new([
        CHDIR,
        SYS_CLONE
    ]);
}

lazy_static! {
    pub static ref APP: BTreeMap<&'static str, Box<&'static [u8]>> = {
        let mut map = BTreeMap::new();
        map.insert("hello_world", Box::new(HELLO_WORLD));
        map.insert("loop10", Box::new(LOOP10));
        map.insert("get_pid", Box::new(GET_PID));
        map.insert("sys_wait4", Box::new(SYS_WAIT4));
        map.insert("sys_brk", Box::new(SYS_BRK));
        map.insert("sys_kill", Box::new(SYS_KILL));
        map.insert("forkboom", Box::new(FORKBOOM));
        map.insert("signal_chld", Box::new(SIGNAL_CHLD));
        map.insert("times", Box::new(TIMES));
        map.insert("nanosleep", Box::new(NANOSLEEP));
        map.insert("read", Box::new(READ));
        map.insert("openat", Box::new(OPENAT));
        map.insert("pipe", Box::new(PIPE));
        map.insert("dup", Box::new(DUP));
        map.insert("mkdirat", Box::new(MKDIRAT));
        map.insert("chdir", Box::new(CHDIR));
        map.insert("get_dirents", Box::new(GET_DIRENTS));
        map.insert("sys_clone", Box::new(SYS_CLONE));
        map.insert("execve", Box::new(EXECVE));
        map.insert("filelink", Box::new(FILELINK));
        map
    };
}
