use super::signal::*;
use super::TrapFrame;
use crate::config::*;
use crate::mm::MemorySpace;
use crate::mm::PageNum;
use crate::vfs::*;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use spin::Mutex;

pub type Pid = usize;

lazy_static! {
    // 1 用作不存在的根Pcb
    static ref PIDALLOCATOR: AtomicUsize = AtomicUsize::new(2);
}

pub fn alloc_pid() -> usize {
    PIDALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

// Note: 使用Atomic类型会出错
// 统计所有Pcb是否释放，检测引用计数
#[cfg(feature = "pcb")]
pub static mut DROPPCBS: Mutex<usize> = Mutex::new(0);

use core::ops::Fn;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PcbState {
    Running,
    Zombie(isize),
    // 保持信号处理前的trapframe和signal mask
    SigHandling(PageNum, Signal),
    // Blocking(fn(Arc<Mutex<Pcb>>) -> bool),
    Blocking,
}

pub struct Pcb {
    pub parent: Pid,
    pub pid: Pid,
    pub cwd: String,
    pub state: PcbState,
    pub memory_space: MemorySpace,
    pub fds: Vec<Option<Fd>>,
    pub children: Vec<Arc<Mutex<Pcb>>>,
    pub sabinds: SigActionBinds,
    // 进程文件系统根目录
    pub root: Inode,

    // times()
    utimes: usize,
    stimes: usize,
    cutimes: usize,
    cstimes: usize,

    pub block_fn: Option<Arc<dyn Fn(&mut Pcb) -> bool>>,
}

unsafe impl Send for Pcb {}

impl Pcb {
    pub fn new(memory_space: MemorySpace, parent: Pid, cwd: String) -> Self {
        let pcb = Self {
            parent,
            pid: alloc_pid(),
            state: PcbState::Running,
            cwd,
            memory_space,
            fds: vec![Some(STDIN.clone()), Some(STDOUT.clone()), Some(STDOUT.clone())],
            children: Vec::new(),
            sabinds: SigActionBinds::new(),
            // 默认根目录
            root: crate::vfs::ROOT.clone(),

            utimes: 0,
            stimes: 0,
            cutimes: 0,
            cstimes: 0,

            block_fn: None,
        };
        #[cfg(feature = "pcb")]
        unsafe {
            *DROPPCBS.lock() += 1;
        }
        sigqueue_init(pcb.pid);
        pcb
    }

    /**
     * 进程上下文
     */
    pub fn trapframe(&mut self) -> &mut TrapFrame {
        self.memory_space.trapframe()
    }

    pub fn clone_child(&mut self) -> Arc<Mutex<Pcb>> {
        let child_ms = self.memory_space.copy();
        let child = Arc::new(Mutex::new(Pcb::new(child_ms, self.pid, self.cwd.clone())));
        let mut childlock = child.lock();
        childlock.trapframe()["a0"] = 0;
        // 先删掉STDIN、STDOUT
        childlock.fds.clear();
        // todo: 考虑O_CLOSEXEC，不拷贝所有fd
        for fd in self.fds.iter() {
            childlock.fds.push(fd.clone())
        }
        drop(childlock);
        self.children.push(child.clone());
        child
    }

    /**
     * 进程文件描述符
     */
    pub fn get_fd(&self, idx: isize) -> Option<Fd> {
        if let Ok(idx) = usize::try_from(idx) {
            if let Some(fd) = self.fds.get(idx) {
                return fd.clone();
            }
        }
        None
    }

    // 在指定的fd处插入
    pub fn fds_add(&mut self, idx: isize, fd: Fd) -> bool {
        if let Ok(fd_ind) = usize::try_from(idx) {
            if self.fds.len() <= fd_ind && fd_ind < MAX_FDS {
                for _ in 0..fd_ind - self.fds.len() {
                    self.fds.push(None)
                }
                self.fds.push(Some(fd));
                return true;
            } else if self.fds.len() > fd_ind {
                if let None = self.get_fd(idx) {
                    self.fds[fd_ind] = Some(fd);
                    return true;
                }
            }
        }
        false
    }

    // 找到空闲的位置插入File，或者push
    pub fn fds_insert(&mut self, fd: Fd) -> Option<usize> {
        match self.fds.iter_mut().enumerate().find(|(_, fd)| fd.is_none()) {
            Some((idx, pos)) => {
                *pos = Some(fd);
                return Some(idx);
            }
            None => {
                self.fds.push(Some(fd));
                return Some(self.fds.len() - 1);
            }
        }
    }

    pub fn fds_close(&mut self, idx: isize) -> bool {
        if let Ok(fd_ind) = usize::try_from(idx) {
            if let Some(_) = self.get_fd(idx) {
                self.fds[fd_ind] = None;
                return true;
            }
        }
        false
    }

    /**
     * 进程状态相关
     */
    // 表明进程可以从阻塞态变为就绪态
    pub fn non_block(&mut self) -> bool {
        self.block_fn.clone().unwrap()(self)
    }

    pub fn state(&self) -> PcbState {
        self.state
    }

    pub fn set_state(&mut self, state: PcbState) -> PcbState {
        let old_state = self.state;
        self.state = state;
        old_state
    }

    pub fn exit(&mut self, xcode: isize) {
        self.state = PcbState::Zombie(xcode);
        // 进程退出就把打开的文件关闭
        self.fds.clear();
    }

    /**
     * 统计进程时间
     */
    pub fn utimes_add(&mut self, times: usize) {
        self.utimes += times;
    }

    pub fn stimes_add(&mut self, times: usize) {
        self.stimes += times;
    }

    pub fn utimes(&self) -> usize {
        self.utimes
    }

    pub fn stimes(&self) -> usize {
        self.stimes
    }

    pub fn cutimes_add(&mut self, times: usize) {
        self.cutimes += times;
    }

    pub fn cstimes_add(&mut self, times: usize) {
        self.cstimes += times;
    }

    pub fn cutimes(&self) -> usize {
        self.cutimes
    }

    pub fn cstimes(&self) -> usize {
        self.cstimes
    }

    /**
     * 信号处理
     */
    pub fn get_sigaction(&mut self, signal: Signal) -> SigAction {
        let act = self
            .sabinds
            .iter_mut()
            .find(|(sig, _)| if *sig == signal { true } else { false });
        if let None = act {
            sigactionbinds_default(signal)
        } else {
            act.unwrap().clone().1
        }
    }

    pub fn sigaction_bind(&mut self, signal: Signal, act: SigAction) {
        log!("signal":"bind">"signal({:?}) -> handler({:?})", signal, act);
        self.sabinds.push((signal, act));
    }

    pub fn try_handle_signal(&mut self) -> PcbState {
        // 信号处理
        // 应该加上一层循环，等待所有信号处理完毕后再调度
        while let Some(signal) = sigqueue_fetch(self.pid) {
            log!("signal":"handle">"pid({}) try handle signal({:?})", self.pid, signal);
            let act = self.get_sigaction(signal);
            match act {
                SigAction::Cont => {
                    // 不做处理
                    continue;
                }
                SigAction::Term => {
                    self.exit(-1);
                    return PcbState::Zombie(-1);
                }
                SigAction::Core => {
                    self.exit(-1);
                    return PcbState::Zombie(-1);
                }
                SigAction::Stop => {
                    self.exit(-1);
                    return PcbState::Zombie(-1);
                }
                SigAction::Ign => {
                    // 不做处理
                    continue;
                }
                SigAction::Custom(act) => {
                    // 使用原有的用户栈
                    // 暂时不支持处理信号时处理其他信号
                    // 不支持使用新的信号处理栈
                    log!("signal":"handle">"pid({}) custom action", self.pid);
                    let mask = sigqueue_mask(self.pid, act.borrow().sa_mask);
                    let oldsp = self.trapframe()["sp"];
                    let oldtf = self.swap_trapframe(act.borrow().trapframe);
                    self.trapframe().init(oldsp, act.borrow().sa_handler);
                    self.trapframe()["a0"] = signal.bits();
                    self.set_state(PcbState::SigHandling(oldtf, mask));
                    return PcbState::SigHandling(oldtf, mask);
                }
            }
        }
        self.set_state(PcbState::Running);
        PcbState::Running
    }

    // 从信号处理的上下文中恢复
    pub fn signal_return(&mut self) {
        if let PcbState::SigHandling(tf, mask) = self.state() {
            log!("signal":"return">"pid({})", self.pid);
            self.swap_trapframe(tf);
            sigqueue_mask(self.pid, mask);
        } else {
            panic!("Not handling signal");
        }
    }

    // 交换trapframe
    fn swap_trapframe(&mut self, tf: PageNum) -> PageNum {
        let old = self.memory_space.trapframe;
        self.memory_space.trapframe = tf;
        old
    }
}

impl Drop for Pcb {
    fn drop(&mut self) {
        #[cfg(feature = "pcb")]
        unsafe {
            *DROPPCBS.lock() -= 1;
        }
        log!("pcb":"drop">"pid({})", self.pid);
        sigqueue_clear(self.pid);
    }
}
