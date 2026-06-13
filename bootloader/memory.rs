//! Módulo de gestión de memoria para CRONOS W-OS
//! Implementa gestión de memoria física y virtual con soporte para grafos de memoria

use bootloader::boot_info::{MemoryMap, MemoryRegion, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Gestor de memoria del kernel
pub struct MemoryManager {
    kernel_page_table: Option<OffsetPageTable<'static>>,
    total_memory: u64,
    used_memory: u64,
}

impl MemoryManager {
    /// Crea un nuevo gestor de memoria
    pub fn new() -> Self {
        MemoryManager {
            kernel_page_table: None,
            total_memory: 0,
            used_memory: 0,
        }
    }

    /// Inicializa el gestor de memoria
    pub unsafe fn init(&mut self, boot_info: &bootloader::BootInfo, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
        // Calcular memoria total
        self.total_memory = boot_info.memory_map.iter().filter(|r| r.kind == MemoryRegionType::Usable).map(|r| r.end - r.start).sum();

        // Crear tabla de páginas del kernel
        let phys_to_virt_offset = boot_info.physical_memory_offset.into_option();
        let level_4_table = boot_info.recursive_index;
        let level_4_table_addr = phys_to_virt_offset.map(|offset| offset + level_4_table * 4096);
        
        if let Some(addr) = level_4_table_addr {
            self.kernel_page_table = Some(OffsetPageTable::new(
                active_level_4_table(frame_allocator),
                VirtAddr::new(addr.as_u64()),
            ));
        }

        println!("💾 Memoria total: {} MB", self.total_memory / (1024 * 1024));
    }

    /// Obtiene la memoria total en bytes
    pub fn total_memory(&self) -> u64 {
        self.total_memory
    }

    /// Obtiene la memoria usada en bytes
    pub fn used_memory(&self) -> u64 {
        self.used_memory
    }

    /// Obtiene la memoria libre en bytes
    pub fn free_memory(&self) -> u64 {
        self.total_memory - self.used_memory
    }
}

/// Allocator de frames basado en información de boot
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Crea un nuevo allocator de frames
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

impl BootInfoFrameAllocator {
    /// Itera sobre las regiones de memoria usables
    pub fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

use x86_64::structures::paging::PhysFrame;
use x86_64::structures::paging::frame::PhysFrame;

/// Obtiene la tabla de páginas de nivel 4 activa
unsafe fn active_level_4_table(frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> PhysFrame {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    level_4_table_frame
}
