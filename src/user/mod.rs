#![allow(unused)]
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub type INT = i32;

pub static CONTEST_TEST: &'static [u8] = include_bytes!("bin/contest_test");
pub static SHELL: &'static [u8] = include_bytes!("bin/shell");
pub static BRK: &'static[u8] = include_bytes!("bin/brk");
pub static CLOSE: &'static[u8] = include_bytes!("bin/close");
pub static EXECVE: &'static[u8] = include_bytes!("bin/execve");
pub static FSTAT: &'static[u8] = include_bytes!("bin/fstat");
pub static GETPID: &'static[u8] = include_bytes!("bin/getpid");
pub static MKDIR_: &'static[u8] = include_bytes!("bin/mkdir_");
pub static MOUNT: &'static[u8] = include_bytes!("bin/mount");
pub static OPENAT: &'static[u8] = include_bytes!("bin/openat");
pub static SLEEP: &'static[u8] = include_bytes!("bin/sleep");
pub static TEST_ECHO: &'static[u8] = include_bytes!("bin/test_echo");      
pub static TEXT_TXT: &'static[u8] = include_bytes!("bin/text.txt");
pub static UNAME: &'static[u8] = include_bytes!("bin/uname");
pub static WAITPID: &'static[u8] = include_bytes!("bin/waitpid");
pub static CHDIR: &'static[u8] = include_bytes!("bin/chdir");  
pub static DUP: &'static[u8] = include_bytes!("bin/dup");    
pub static EXIT: &'static[u8] = include_bytes!("bin/exit");
pub static GETCWD: &'static[u8] = include_bytes!("bin/getcwd");
pub static GETPPID: &'static[u8] = include_bytes!("bin/getppid");
pub static MMAP: &'static[u8] = include_bytes!("bin/mmap");
pub static MUNMAP: &'static[u8] = include_bytes!("bin/munmap");
pub static PIPE: &'static[u8] = include_bytes!("bin/pipe");
pub static TIMES: &'static[u8] = include_bytes!("bin/times");
pub static UNLINK: &'static[u8] = include_bytes!("bin/unlink");
pub static WRITE: &'static[u8] = include_bytes!("bin/write");
pub static CLONE: &'static[u8] = include_bytes!("bin/clone");
pub static DUP2: &'static[u8] = include_bytes!("bin/dup2");
pub static FORK: &'static[u8] = include_bytes!("bin/fork");
pub static GETDENTS: &'static[u8] = include_bytes!("bin/getdents");
pub static GETTIMEOFDAY: &'static[u8] = include_bytes!("bin/gettimeofday");
pub static OPEN: &'static[u8] = include_bytes!("bin/open");
pub static READ: &'static[u8] = include_bytes!("bin/read");
pub static UMOUNT: &'static[u8] = include_bytes!("bin/umount");
pub static WAIT: &'static[u8] = include_bytes!("bin/wait");
pub static YIELD: &'static[u8] = include_bytes!("bin/yield");

lazy_static!{
    pub static ref APPS: BTreeMap<&'static str, Box<&'static [u8]>> = {
        let mut map = BTreeMap::new();
        map.insert("brk", Box::new(BRK));
        map.insert("close", Box::new(CLOSE));
        map.insert("execve", Box::new(EXECVE));
        map.insert("fstat", Box::new(FSTAT));
        map.insert("getpid", Box::new(GETPID));
        map.insert("mkdir_", Box::new(MKDIR_));
        map.insert("mount", Box::new(MOUNT));
        map.insert("openat", Box::new(OPENAT));
        map.insert("sleep", Box::new(SLEEP));
        map.insert("test_echo", Box::new(TEST_ECHO));      
        map.insert("text.txt", Box::new(TEXT_TXT));
        map.insert("uname", Box::new(UNAME));
        map.insert("waitpid", Box::new(WAITPID));
        map.insert("chdir", Box::new(CHDIR));  
        map.insert("dup", Box::new(DUP));    
        map.insert("exit", Box::new(EXIT));
        map.insert("getcwd", Box::new(GETCWD));
        map.insert("getppid", Box::new(GETPPID));
        map.insert("mmap", Box::new(MMAP));
        map.insert("munmap", Box::new(MUNMAP));
        map.insert("pipe", Box::new(PIPE));
        map.insert("times", Box::new(TIMES));
        map.insert("unlink", Box::new(UNLINK));
        map.insert("write", Box::new(WRITE));
        map.insert("clone", Box::new(CLONE));
        map.insert("dup2", Box::new(DUP2));
        map.insert("fork", Box::new(FORK));
        map.insert("getdents", Box::new(GETDENTS));
        map.insert("gettimeofday", Box::new(GETTIMEOFDAY));
        map.insert("open", Box::new(OPEN));
        map.insert("read", Box::new(READ));
        map.insert("umount", Box::new(UMOUNT));
        map.insert("wait", Box::new(WAIT));
        map.insert("yield", Box::new(YIELD));
        map
    };
}