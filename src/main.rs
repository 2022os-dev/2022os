// [no_std] Don't use standard library
#![no_std]
// [no_main] Tell compiler we don't need initialization before main() #![no_main]
#![no_main]
#![feature(naked_functions)]
// [global_asm] allow include an assemble file
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use crate::process::cpu::init_hart;
use core::arch::asm;

#[macro_use]
mod lang_items;
mod sbi;

#[macro_use]
mod console;

mod batch;
mod entry;
mod heap;
mod link_syms;
mod mm;
mod process;
mod syscall;
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

/// Clear .bss section
fn clear_bss() {
    (link_syms::sbss as usize..link_syms::ebss as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// [no_mangle] Turn off Rust's name mangling
#[no_mangle]
extern "C" fn kernel_start() {
    use mm::memory_space::MemorySpace;
    use task::{schedule_pcb, TASKMANAGER};

    console::turn_on_log();
    // Use new stack
    unsafe {
        asm!("mv sp, {0}",
         in(reg) batch::KERNEL_STACK.get_top());
    }
    clear_bss();
    mm::init();
    println!("[kernel] Clear bss");
    heap::init();
    println!("[kernel] Init heap");
    trap::init();
    println!("[kernel] Init trap");

    init_hart();

    // Run user space application
    println!("[kernel] Load user address space");
    // Load task #1
    let mut virtual_space = MemorySpace::from_elf(user::APP[0]);
    virtual_space.map_trampoline();
    TASKMANAGER.lock().load_pcb(virtual_space);

    // Load task #1
    let mut virtual_space = MemorySpace::from_elf(user::APP[1]);
    virtual_space.map_trampoline();
    TASKMANAGER.lock().load_pcb(virtual_space);

    trap::enable_timer_interupt();
    log!(debug "Start schedule");
    schedule_pcb();
}
