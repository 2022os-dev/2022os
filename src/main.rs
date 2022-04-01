// [no_std] Don't use standard library
#![no_std]
// [no_main] Tell compiler we don't need initialization before main() #![no_main]
#![no_main]
#![feature(naked_functions)]
// [global_asm] allow include an assemble file
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(ptr_to_from_bits)]
#![feature(const_trait_impl)]

use crate::{process::cpu::{init_hart, hart_enable_timer_interrupt}, sbi::{sbi_hsm_hart_start}, clock::clock_init};
use core::arch::asm;

#[macro_use]
mod lang_items;
mod sbi;

#[macro_use]
mod console;

mod entry;
mod heap;
mod link_syms;
mod mm;
mod process;
mod task;
mod trap;
mod user;

mod config;
mod clock;

#[macro_use]
extern crate lazy_static;
extern crate alloc;
extern crate buddy_system_allocator;
extern crate spin;
#[macro_use]
extern crate bitflags;

use mm::*;
use task::*;
use process::cpu::hartid;

/// Clear .bss section
fn clear_bss() {
    (link_syms::sbss as usize..link_syms::ebss as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// 记录启动核
static mut BOOTHART: isize = -1 ;

// [no_mangle] Turn off Rust's name mangling
#[no_mangle]
extern "C" fn kernel_start() {
    log!("hart":"Booting">"");
    if unsafe { BOOTHART } == -1 {
        unsafe { BOOTHART = hartid() as isize; };

        clear_bss();

        // 需要在开启虚拟内存之前初始化时钟，
        // 因为内核不会映射时钟配置寄存器
        #[cfg(feature = "init_clock")]
        clock_init();

        heap::init();

        mm::init();

        init_hart();

        // Load tasks
        for i in user::APP.iter() {
            let virtual_space = MemorySpace::from_elf(i);
            scheduler_load_pcb(virtual_space);
        }

        #[cfg(feature = "multicore")]
        for i in 1..=4 {
            if hartid() != i {
                sbi_hsm_hart_start(i, 0x80200000, 0);
            }
        }
    } else {
        init_hart();
    }
    trap::init();
    hart_enable_timer_interrupt();
    log!(debug "Start schedule");
    schedule();
}
