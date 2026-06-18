//! Simple bump allocator for no_std kernel

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;
}

pub struct BumpAllocator {
    heap_start: AtomicUsize,
    heap_size: AtomicUsize,
    next: AtomicUsize,
}

unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        
        let current = self.next.load(Ordering::Relaxed);
        let aligned = (current + align - 1) & !(align - 1);
        let new_next = aligned + size;
        
        let heap_size = self.heap_size.load(Ordering::Relaxed);
        if new_next > heap_size {
            core::ptr::null_mut()
        } else {
            self.next.store(new_next, Ordering::Relaxed);
            let heap_start = self.heap_start.load(Ordering::Relaxed);
            (heap_start + aligned) as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't free
    }
}

impl BumpAllocator {
    /// Compatibilidad con interfaz LockedHeap antigua
    pub fn lock(&self) -> &Self {
        &self
    }
    
    /// Inicialización compatible (no-op para bump allocator)
    pub fn init(&self, _ptr: *mut u8, _size: usize) {
        // Bump allocator se inicializa via init_allocator()
    }
}

#[global_allocator]
pub static ALLOCATOR: BumpAllocator = BumpAllocator {
    heap_start: AtomicUsize::new(0),
    heap_size: AtomicUsize::new(0),
    next: AtomicUsize::new(0),
};

use crate::boot::BootInfo;

pub fn init_allocator(_boot_info: &BootInfo) {
    unsafe {
        extern "C" {
            static mut HEAP_MEMORY: [u8; 2 * 1024 * 1024];
        }
        const HEAP_SIZE: usize = 2 * 1024 * 1024;
        let heap_start = &raw mut HEAP_MEMORY as usize;
        
        crate::serial_println!("init_allocator: using static heap at {:#x}, size={:#x}", heap_start, HEAP_SIZE);
        ALLOCATOR.heap_start.store(heap_start, Ordering::Relaxed);
        ALLOCATOR.heap_size.store(HEAP_SIZE, Ordering::Relaxed);
        ALLOCATOR.next.store(0, Ordering::Relaxed);
        crate::serial_println!("init_allocator: BumpAllocator initialized OK");
    }
}