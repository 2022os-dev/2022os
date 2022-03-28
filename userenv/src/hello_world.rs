#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use console::*;
use syscall::*;

pub fn main() {
    println!("Hello world");
}
