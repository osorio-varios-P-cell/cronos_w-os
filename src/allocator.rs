//! Simple bump allocator for no_std kernel

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};
use linked_list_allocator::LockedHeap;

extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;
}

struct BumpAllocator {
    heap_start: usize,
    heap_size: usize,
    next: AtomicUsize,
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alloc_start = self.next.fetch_add(layout.size(), Ordering::Relaxed);
        let alloc_end = alloc_start + layout.size();

        if alloc_end > self.heap_size {
            // Out of memory
            self.next.fetch_sub(layout.size(), Ordering::Relaxed);
            core::ptr::null_mut()
        } else {
            (self.heap_start + alloc_start) as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't free
    }
}

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_allocator() {
    unsafe {
        // For now, use a fixed heap location since linker script symbols aren't working
        // TODO: Fix linker script to properly define __heap_start and __heap_end
        const HEAP_START: usize = 0x2000000; // 32MB mark
        const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16MB heap
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
}
