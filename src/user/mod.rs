#![allow(unused)]
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub type INT = i32;

pub static CONTEST_TEST: &'static [u8] = include_bytes!("bin/contest_test");
pub static SHELL: &'static [u8] = include_bytes!("bin/shell");

#[cfg(feature = "memfs")]
lazy_static!{
pub static ref BRK: &'static[u8] = include_bytes!("bin/brk");
pub static ref CLOSE: &'static[u8] = include_bytes!("bin/close");
pub static ref EXECVE: &'static[u8] = include_bytes!("bin/execve");
pub static ref FSTAT: &'static[u8] = include_bytes!("bin/fstat");
pub static ref GETPID: &'static[u8] = include_bytes!("bin/getpid");
pub static ref MKDIR_: &'static[u8] = include_bytes!("bin/mkdir_");
pub static ref MOUNT: &'static[u8] = include_bytes!("bin/mount");
pub static ref OPENAT: &'static[u8] = include_bytes!("bin/openat");
pub static ref SLEEP: &'static[u8] = include_bytes!("bin/sleep");
pub static ref TEST_ECHO: &'static[u8] = include_bytes!("bin/test_echo");      
pub static ref TEXT_TXT: &'static[u8] = include_bytes!("bin/text.txt");
pub static ref UNAME: &'static[u8] = include_bytes!("bin/uname");
pub static ref WAITPID: &'static[u8] = include_bytes!("bin/waitpid");
pub static ref CHDIR: &'static[u8] = include_bytes!("bin/chdir");  
pub static ref DUP: &'static[u8] = include_bytes!("bin/dup");    
pub static ref EXIT: &'static[u8] = include_bytes!("bin/exit");
pub static ref GETCWD: &'static[u8] = include_bytes!("bin/getcwd");
pub static ref GETPPID: &'static[u8] = include_bytes!("bin/getppid");
pub static ref MMAP: &'static[u8] = include_bytes!("bin/mmap");
pub static ref MUNMAP: &'static[u8] = include_bytes!("bin/munmap");
pub static ref PIPE: &'static[u8] = include_bytes!("bin/pipe");
pub static ref TIMES: &'static[u8] = include_bytes!("bin/times");
pub static ref UNLINK: &'static[u8] = include_bytes!("bin/unlink");
pub static ref WRITE: &'static[u8] = include_bytes!("bin/write");
pub static ref CLONE: &'static[u8] = include_bytes!("bin/clone");
pub static ref DUP2: &'static[u8] = include_bytes!("bin/dup2");
pub static ref FORK: &'static[u8] = include_bytes!("bin/fork");
pub static ref GETDENTS: &'static[u8] = include_bytes!("bin/getdents");
pub static ref GETTIMEOFDAY: &'static[u8] = include_bytes!("bin/gettimeofday");
pub static ref OPEN: &'static[u8] = include_bytes!("bin/open");
pub static ref READ: &'static[u8] = include_bytes!("bin/read");
pub static ref UMOUNT: &'static[u8] = include_bytes!("bin/umount");
pub static ref WAIT: &'static[u8] = include_bytes!("bin/wait");
pub static ref YIELD: &'static[u8] = include_bytes!("bin/yield");
}

#[cfg(feature = "memfs")]
lazy_static!{
    pub static ref APPS: BTreeMap<&'static str, Box<&'static [u8]>> = {
        let mut map = BTreeMap::new();
        map.insert("brk", Box::new(*BRK));
        map.insert("close", Box::new(*CLOSE));
        map.insert("execve", Box::new(*EXECVE));
        map.insert("fstat", Box::new(*FSTAT));
        map.insert("getpid", Box::new(*GETPID));
        map.insert("mkdir_", Box::new(*MKDIR_));
        map.insert("mount", Box::new(*MOUNT));
        map.insert("openat", Box::new(*OPENAT));
        map.insert("sleep", Box::new(*SLEEP));
        map.insert("test_echo", Box::new(*TEST_ECHO));      
        map.insert("text.txt", Box::new(*TEXT_TXT));
        map.insert("uname", Box::new(*UNAME));
        map.insert("waitpid", Box::new(*WAITPID));
        map.insert("chdir", Box::new(*CHDIR));  
        map.insert("dup", Box::new(*DUP));    
        map.insert("exit", Box::new(*EXIT));
        map.insert("getcwd", Box::new(*GETCWD));
        map.insert("getppid", Box::new(*GETPPID));
        map.insert("mmap", Box::new(*MMAP));
        map.insert("munmap", Box::new(*MUNMAP));
        map.insert("pipe", Box::new(*PIPE));
        map.insert("times", Box::new(*TIMES));
        map.insert("unlink", Box::new(*UNLINK));
        map.insert("write", Box::new(*WRITE));
        map.insert("clone", Box::new(*CLONE));
        map.insert("dup2", Box::new(*DUP2));
        map.insert("fork", Box::new(*FORK));
        map.insert("getdents", Box::new(*GETDENTS));
        map.insert("gettimeofday", Box::new(*GETTIMEOFDAY));
        map.insert("open", Box::new(*OPEN));
        map.insert("read", Box::new(*READ));
        map.insert("umount", Box::new(*UMOUNT));
        map.insert("wait", Box::new(*WAIT));
        map.insert("yield", Box::new(*YIELD));
        map
    };
}