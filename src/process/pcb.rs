use super::signal::*;
use super::TrapFrame;
use crate::mm::MemorySpace;
use crate::mm::PageNum;
use crate::vfs::*;
use crate::config::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::vec;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use spin::Mutex;

pub type Pid = usize;

lazy_static! {
    // 0 用作不存在的根Pcb
    static ref PIDALLOCATOR: AtomicUsize = AtomicUsize::new(1);
}

pub fn alloc_pid() -> usize {
    PIDALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

// Note: 使用Atomic类型会出错
// 统计所有Pcb是否释放，检测引用计数
#[cfg(feature = "pcb")]
pub static mut DROPPCBS: Mutex<usize> = Mutex::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PcbState {
    Running,
    Zombie(isize),
    // 保持信号处理前的trapframe和signal mask
    SigHandling(PageNum, Signal),
    Blocking(fn(Arc<Mutex<Pcb>>) -> bool),
}

pub struct Pcb {
    pub parent: Pid,
    pub pid: Pid,
    pub cwd: String,
    pub state: PcbState,
    pub memory_space: MemorySpace,
    pub fds: Vec<Option<File>>,
    pub children: Vec<Arc<Mutex<Pcb>>>,
    pub sabinds: SigActionBinds,

    // times()
    utimes: usize,
    stimes: usize,
    cutimes: usize,
    cstimes: usize,

    // for nanosleep
    pub wakeup_time: Option<usize>,
}

unsafe impl Send for Pcb {}

impl Pcb {
    pub fn new(memory_space: MemorySpace, parent: Pid) -> Self {
        let pcb = Self {
            parent,
            pid: alloc_pid(),
            state: PcbState::Running,
            cwd: String::from("/"),
            memory_space,
            fds: vec![Some(File::open(CONSOLE.clone(), OpenFlags::RDONLY).unwrap()),
                     Some(File::open(CONSOLE.clone(), OpenFlags::WRONLY).unwrap())],
            children: Vec::new(),
            sabinds: SigActionBinds::new(),

            utimes: 0,
            stimes: 0,
            cutimes: 0,
            cstimes: 0,

            wakeup_time: None,
        };
        #[cfg(feature = "pcb")]
        unsafe {
            *DROPPCBS.lock() += 1;
        }
        sigqueue_init(pcb.pid);
        pcb
    }

    pub fn get_fd(&self, idx: usize) -> Option<&File> {
        if let Some(fd) = self.fds.get(idx) {
            fd.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut_fd(&mut self, idx: usize) -> Option<&mut File> {
        if let Some(fd) = self.fds.get_mut(idx) {
            fd.as_mut()
        } else {
            None
        }
    }

    pub fn fds_add(&mut self, idx: usize, file: File) -> bool {
        if self.fds.len() <= idx && idx < MAX_FDS {
            for _ in 0..idx - self.fds.len() {
                self.fds.push(None)
            }
            self.fds.push(Some(file));
            return true
        } else if self.fds.len() > idx {
            if let None = self.get_fd(idx) {
                self.fds[idx] = Some(file);
                return true
            }
        }
        false
    }

    pub fn state(&self) -> PcbState {
        self.state
    }

    pub fn set_state(&mut self, state: PcbState) -> PcbState {
        let old_state = self.state;
        self.state = state;
        old_state
    }

    pub fn trapframe(&mut self) -> &mut TrapFrame {
        self.memory_space.trapframe()
    }

    pub fn exit(&mut self, xcode: isize) {
        self.state = PcbState::Zombie(xcode);
    }

    pub fn get_sigaction(&mut self, signal: Signal) -> SigAction {
        let act =
            self.sabinds
                .iter_mut()
                .find(|(sig, _)| if *sig == signal { true } else { false });
        if let None = act {
            sigactionbinds_default(signal)
        } else {
            act.unwrap().clone().1
        }
    }

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

/*
pub fn pcb_block_slot(pcb: Arc<Mutex<Pcb>>, reason: BlockReason) {
    log!("pcb":"slot">"pid({}) - Reason({:?})", pcb.lock().pid, reason);
    match reason {
        BlockReason::Wait => {
            scheduler_ready_pcb(pcb);
        }
    }
}
*/
