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

use crate::{process::cpu::init_hart, sbi::{sbi_hsm_hart_start}};
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

pub static mut KERNEL_PGTBL: Option<Pgtbl> = None; 

// 记录启动核
static mut BOOTHART: isize = -1 ;

// [no_mangle] Turn off Rust's name mangling
#[no_mangle]
extern "C" fn kernel_start() {
    log!(debug "Booting hart {}", hartid());
    if unsafe { BOOTHART } == -1 {
        unsafe { BOOTHART = hartid() as isize; };

        console::turn_on_log();
        clear_bss();
        mm::init();
        println!("[kernel] Clear bss");
        heap::init();
        println!("[kernel] Init heap");

        unsafe {
            init_hart(KERNEL_PGTBL.as_ref().unwrap());
        }

        // Run user space application
        println!("[kernel] Load user address space");

        // Load tasks
        for i in *user::APP {
            let virtual_space = MemorySpace::from_elf(i);
            scheduler_load_pcb(virtual_space);
        }
        for hart in 1..=4 {
            if hart != hartid() {
                sbi_hsm_hart_start(hart, 0x80200000, 0);
            }
        }
    } else {
        unsafe {
            init_hart(KERNEL_PGTBL.as_ref().unwrap());
        }
    }
    trap::init();
    // trap::enable_timer_interupt();
    log!(debug "Start schedule");
    schedule();
}
