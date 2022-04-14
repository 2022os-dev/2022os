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

#[cfg(feature = "gitee_test")]
pub static GITEE_TEST_ECHO: &'static [u8] = include_bytes!("bin/test_echo");
#[cfg(feature = "gitee_test")]
pub static GITEE_BRK: &'static [u8] = include_bytes!("bin/gitee_brk");
#[cfg(feature = "gitee_test")]
pub static GITEE_CHDIR: &'static [u8] = include_bytes!("bin/gitee_chdir");
#[cfg(feature = "gitee_test")]
pub static GITEE_CLONE: &'static [u8] = include_bytes!("bin/gitee_clone");
#[cfg(feature = "gitee_test")]
pub static GITEE_CLOSE: &'static [u8] = include_bytes!("bin/gitee_close");
#[cfg(feature = "gitee_test")]
pub static GITEE_DUP: &'static [u8] = include_bytes!("bin/gitee_dup");
#[cfg(feature = "gitee_test")]
pub static GITEE_DUP2: &'static [u8] = include_bytes!("bin/gitee_dup2");
#[cfg(feature = "gitee_test")]
pub static GITEE_EXECVE: &'static [u8] = include_bytes!("bin/gitee_execve");
#[cfg(feature = "gitee_test")]
pub static GITEE_EXIT: &'static [u8] = include_bytes!("bin/gitee_exit");
#[cfg(feature = "gitee_test")]
pub static GITEE_FORK: &'static [u8] = include_bytes!("bin/gitee_fork");
#[cfg(feature = "gitee_test")]
pub static GITEE_FSTAT: &'static [u8] = include_bytes!("bin/gitee_fstat");
#[cfg(feature = "gitee_test")]
pub static GITEE_GETCWD: &'static [u8] = include_bytes!("bin/gitee_getcwd");
#[cfg(feature = "gitee_test")]
pub static GITEE_GETDENTS: &'static [u8] = include_bytes!("bin/gitee_getdents");
#[cfg(feature = "gitee_test")]
pub static GITEE_GETPID: &'static [u8] = include_bytes!("bin/gitee_getpid");
#[cfg(feature = "gitee_test")]
pub static GITEE_GETPPID: &'static [u8] = include_bytes!("bin/gitee_getppid");
#[cfg(feature = "gitee_test")]
pub static GITEE_GETTIMEOFDAY: &'static [u8] = include_bytes!("bin/gitee_gettimeofday");
#[cfg(feature = "gitee_test")]
pub static GETEE_MKDIR_: &'static [u8] = include_bytes!("bin/gitee_mkdir_");

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

        #[cfg(feature = "gitee_test")]
        map.insert("gitee_brk", Box::new(GITEE_BRK));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_chdir", Box::new(GITEE_CHDIR));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_clone", Box::new(GITEE_CLONE));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_close", Box::new(GITEE_CLOSE));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_dup", Box::new(GITEE_DUP));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_dup2", Box::new(GITEE_DUP2));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_execve", Box::new(GITEE_EXECVE));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_exit", Box::new(GITEE_EXIT));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_fork", Box::new(GITEE_FORK));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_fstat", Box::new(GITEE_FSTAT));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_getcwd", Box::new(GITEE_GETCWD));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_getdents", Box::new(GITEE_GETDENTS));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_getpid", Box::new(GITEE_GETPID));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_getppid", Box::new(GITEE_GETPPID));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_gettimeofday", Box::new(GITEE_GETTIMEOFDAY));
        #[cfg(feature = "gitee_test")]
        map.insert("gitee_mkdir_", Box::new(GETEE_MKDIR_));
        #[cfg(feature = "gitee_test")]
        // test_echo 被gitee_execve执行
        map.insert("test_echo", Box::new(GITEE_TEST_ECHO));
        map
    };
}
