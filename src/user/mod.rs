#![allow(unused)]
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub type INT = i32;

pub static CONTEST_TEST: &'static [u8] = include_bytes!("bin/contest_test");
pub static SHELL: &'static [u8] = include_bytes!("bin/shell");