const KERNEL_HEAP_SIZE: usize = PAGE_SIZE * 16;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

use crate::{buddy_system_allocator::LockedHeap, config::PAGE_SIZE};

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    panic!("Alloc Error");
}
