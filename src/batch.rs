/**
 * 该文件将会被删除，有用的结构将被移动到相应的目录中
 */

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

#[repr(align(4096))]
pub struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    pub fn get_top(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
    pub fn get_bottom(&self) -> usize {
        self.data.as_ptr() as usize
    }
}
