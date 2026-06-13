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

use crate::boot::BootInfo;

pub fn init_allocator(boot_info: &BootInfo) {
    unsafe {
        // BUG #16 Corregido: Buscar una región usable real en el mapa de memoria
        // En lugar de una dirección estática, usamos la primera región usable >= 32MB
        let heap_size = 32 * 1024 * 1024;
        let mut heap_start = 0;

        for region in boot_info.usable_regions.iter().flatten() {
            if region.length >= heap_size as u64 {
                heap_start = boot_info.hhdm_offset + region.base;
                break;
            }
        }

        if heap_start != 0 {
            ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
        } else {
            // Fallback de emergencia si no hay regiones grandes (no debería ocurrir en HW real)
            ALLOCATOR.lock().init((boot_info.hhdm_offset + 0x2000000) as *mut u8, heap_size);
        }
    }
}
