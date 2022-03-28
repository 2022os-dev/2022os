/// Symbol about memory map,
/// provided by linker

extern "C" {
    #![allow(unused)]
    pub fn sdata();
    pub fn edata();
    pub fn srodata();
    pub fn erodata();
    pub fn stext();
    pub fn etext();
    pub fn frames();
    pub fn sbss();
    pub fn ebss();
    pub fn skernel();
    pub fn ekernel();
    pub fn boot_stack();
    pub fn boot_stack_top();
}
