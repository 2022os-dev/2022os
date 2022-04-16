use spin::MutexGuard;
use crate::mm::*;
use crate::process::Pcb;

#[repr(C)]
struct UtsName {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

const SYSNAME: &'static str = "2022os\0";
const NODENAME: &'static str = "\0";
const RELEASE: &'static str = "v0.1\0";
const VERSION: &'static str = "v0.1\0";
const MACHINE: &'static str = "Hifive Unmatched\0";
const DOMAINNAME: &'static str = "\0";

pub(super) fn sys_uname(pcb: &mut MutexGuard<Pcb>, utsname: VirtualAddr) -> isize {
    let mut phys: PhysAddr = utsname.into();
    let utsname: &mut UtsName = phys.as_mut();
    utsname.sysname[..SYSNAME.len()].copy_from_slice(SYSNAME.as_bytes());
    utsname.nodename[..NODENAME.len()].copy_from_slice(NODENAME.as_bytes());
    utsname.release[..RELEASE.len()].copy_from_slice(RELEASE.as_bytes());
    utsname.version[..VERSION.len()].copy_from_slice(VERSION.as_bytes());
    utsname.machine[..MACHINE.len()].copy_from_slice(MACHINE.as_bytes());
    utsname.domainname[..DOMAINNAME.len()].copy_from_slice(DOMAINNAME.as_bytes());
    0
}