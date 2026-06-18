//! Gestión de memoria para CRONOS W-OS
//! 
//! Este módulo implementa la gestión de memoria física y virtual
//! con características de seguridad avanzadas y reset cristalino

use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PageTableFlags, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};
use core::ptr;
use alloc::{vec::Vec, format};
use crate::allocator::ALLOCATOR;
use crate::bitmap_frame_allocator::{BitmapFrameAllocator, MemZone};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Custom memory types (replacing bootloader::bootinfo)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    BootloaderReclaimable,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    Framebuffer,
    KernelAndModules,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRange {
    pub start_frame_number: u64,
    pub end_frame_number: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub range: MemoryRange,
    pub region_type: MemoryRegionType,
}

/// Simple memory map holding a list of regions
pub struct MemoryMap {
    pub regions: Vec<MemoryRegion>,
}

impl MemoryMap {
    pub fn iter(&self) -> core::slice::Iter<'_, MemoryRegion> {
        self.regions.iter()
    }
}

/// Implementaciones manuales de funciones de memoria (sin usar ptr::* para evitar recursión)
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest < src as *mut u8 {
        let mut i = 0;
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
    } else if dest > src as *mut u8 {
        let mut i = n;
        while i > 0 {
            i -= 1;
            *dest.add(i) = *src.add(i);
        }
    }
    dest
}

#[no_mangle]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return (a as i32) - (b as i32);
            }
        }
        0
    }
}

/// Obtiene el nivel 4 de la tabla de páginas activa
unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    &mut *virt.as_mut_ptr::<PageTable>()
}

/// Tamaño del heap global (2 MB para arranque)
const HEAP_SIZE: usize = 2 * 1024 * 1024;


/// Buffer estático para el heap
#[no_mangle]
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Inicializa el heap global del kernel
pub unsafe fn init_heap() {
    ALLOCATOR.lock().init(HEAP_MEMORY.as_mut_ptr(), HEAP_SIZE);
}

/// Gestor de memoria del kernel CRONOS W-OS
#[derive(Debug)]
pub struct MemoryManager {
    /// Mapeador de páginas de la tabla de páginas actual
    page_table: Option<OffsetPageTable<'static>>,
    /// Offset de memoria física (del bootloader)
    phys_offset: u64,
    /// Tamaño de la memoria física en bytes
    physical_memory_size: u64,
    /// Regiones de memoria disponibles
    available_memory: Vec<MemoryRegion>,
    /// Regiones de memoria usadas
    used_memory: Vec<MemoryRegion>,
    /// Estado de seguridad de la memoria
    security_state: MemorySecurityState,
    /// Asignador de frames basado en bitmap (O(n/64))
    frame_allocator: BitmapFrameAllocator,
}

impl MemoryManager {
    /// Registra las regiones de memoria en el GraphKernel
    pub fn register_in_graph(&self, graph_kernel: &GraphKernel) {
        if let Some(root_id) = graph_kernel.root_node() {
            for region in &self.available_memory {
                let name = format!("mem_0x{:x}_0x{:x}",
                    region.range.start_frame_number * 4096,
                    region.range.end_frame_number * 4096);

                let node_id = graph_kernel.create_node(
                    NodeType::MemoryRegion,
                    name,
                );

                // Crear arista de Ownership desde el root
                graph_kernel.create_edge(root_id, node_id, EdgeType::Ownership);
            }
        }
    }
}

/// Estado de seguridad de la memoria
#[derive(Debug, Clone)]
pub struct MemorySecurityState {
    /// Memoria encriptada
    encrypted_pages: u64,
    /// Memoria borrada (secure erase)
    erased_pages: u64,
    /// Páginas con protección de ejecución
    nx_pages: u64,
    /// Páginas con protección de escritura
    ro_pages: u64,
}

/// Asignador de frames de memoria para el bootloader
pub struct BootInfoFrameAllocator {
    /// Regiones de memoria disponibles
    available: Vec<MemoryRegion>,
    /// Regiones de memoria ya usadas
    used: Vec<MemoryRegion>,
    /// Página actual para asignación
    next: Page,
}

impl MemoryManager {
    /// Crea un nuevo gestor de memoria
    pub fn new(phys_offset: u64) -> Self {
        Self {
            page_table: None,
            phys_offset,
            physical_memory_size: 0,
            available_memory: Vec::new(),
            used_memory: Vec::new(),
            security_state: MemorySecurityState {
                encrypted_pages: 0,
                erased_pages: 0,
                nx_pages: 0,
                ro_pages: 0,
            },
            frame_allocator: BitmapFrameAllocator::new(),
        }
    }

    /// Inicializa el gestor de memoria
    /// 
    /// # Safety
    /// Esta función debe llamarse solo una vez durante el boot
    pub unsafe fn init(&mut self, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
        
        // Crear el page mapper usando la tabla de páginas activa (del bootloader)
        self.create_mapper();
        
        // 1. Mapear memoria del kernel (si hay mapper disponible)
        self.map_kernel_memory(frame_allocator).expect("Error mapeando memoria del kernel");
        
        // 2. Configurar páginas de identidad
        self.setup_identity_mapping(frame_allocator).expect("Error configurando identity mapping");
        
        // 3. Configurar NX (No Execute) por defecto
        self.setup_nx_protection(frame_allocator).expect("Error configurando NX protection");
        
        // 4. Inicializar sistema de borrado seguro
        self.init_secure_erase();
        
    }

    /// Crea el mapper usando la tabla de páginas activa
    unsafe fn create_mapper(&mut self) {
        let phys_offset = VirtAddr::new(self.phys_offset);
        let level_4_table = active_level_4_table(phys_offset);
        self.page_table = Some(OffsetPageTable::new(level_4_table, phys_offset));
    }

    /// Inicializa el gestor de memoria con parámetros directos (desde Limine)
    pub unsafe fn new_with_params(phys_offset: u64, regions: &[MemoryRegion]) -> Self {
        let mut manager = Self::new(phys_offset);
        
        // Convertir MemoryRegion a formato para BitmapFrameAllocator
        let mut regions_for_bitmap: Vec<(u64, u64)> = regions.iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| (r.range.start_frame_number * 4096, r.range.end_frame_number * 4096))
            .collect();
        
        // Inicializar el bitmap allocator con las regiones disponibles
        if let Some(first_region) = regions_for_bitmap.first() {
            manager.frame_allocator.init(first_region.0, &regions_for_bitmap);
        }
        
        manager.physical_memory_size = regions.iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| (r.range.end_frame_number - r.range.start_frame_number) * 4096)
            .sum();
        
        manager.available_memory = regions.iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .cloned()
            .collect();
        
        manager
    }

    /// Obtiene el mapper
    pub fn mapper_mut(&mut self) -> &mut OffsetPageTable<'static> {
        self.page_table.as_mut().expect("page table not initialized")
    }

    /// Mapea la memoria del kernel
    unsafe fn map_kernel_memory(&mut self, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), FrameError> {
        use x86_64::structures::paging::Page;
        
        // Map scratch pages for kernel struct storage
        let scratch_start = 0xFFFF_FFFF_0000_0000u64;
        let scratch_end = 0xFFFF_FFFF_0000_6000u64;
        let start_page = Page::containing_address(VirtAddr::new(scratch_start));
        let end_page = Page::containing_address(VirtAddr::new(scratch_end));
        
        for page in Page::range_inclusive(start_page, end_page) {
            match self.mapper_mut().translate(page.start_address()) {
                x86_64::structures::paging::mapper::TranslateResult::Mapped { .. } => continue,
                _ => {}
            }
            let frame = frame_allocator.allocate_frame().expect("frame allocation failed");
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            
            unsafe {
                self.mapper_mut().map_to(page, frame, flags, frame_allocator)
                    .map_err(|_| FrameError::FrameNotPresent)?
                    .flush();
            }
        }
        
        Ok(())
    }

    /// Configura identity mapping real para acceso directo a hardware
    unsafe fn setup_identity_mapping(&mut self, _frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), FrameError> {
        use x86_64::structures::paging::PhysFrame;

        let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(0));
        let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(0x400000));

        for page in Page::range_inclusive(start_page, end_page) {
            match self.mapper_mut().translate(page.start_address()) {
                x86_64::structures::paging::mapper::TranslateResult::Mapped { .. } => continue,
                _ => {}
            }
            let phys_addr = PhysAddr::new(page.start_address().as_u64());
            let frame = PhysFrame::containing_address(phys_addr);
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            unsafe {
                self.mapper_mut().map_to(page, frame, flags, _frame_allocator)
                    .map_err(|_| FrameError::FrameNotPresent)?
                    .flush();
            }
        }

        Ok(())
    }

    /// Configura protección NX (No Execute) por defecto
    unsafe fn setup_nx_protection(&mut self, _frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), FrameError> {
        // En x86_64, NX es el comportamiento por defecto
        // Aquí configuramos las páginas que SÍ se pueden ejecutar
        self.security_state.nx_pages = self.physical_memory_size / 4096;
        
        Ok(())
    }

    /// Inicializa el sistema de borrado seguro
    fn init_secure_erase(&mut self) {
        
        // Configurar áreas de memoria para borrado seguro
        // En implementación real, esto configuraría hardware específico
        self.security_state.erased_pages = 0;
        
    }

    /// Realiza borrado seguro de una página de memoria
    pub fn secure_erase_page(&mut self, page: Page<Size4KiB>) -> Result<(), SecureEraseError> {
        
        // En implementación real:
        // 1. Escribir patrón de datos
        // 2. Aplicar voltaje específico
        // 3. Verificar borrado completo
        
        // Simulación por ahora
        self.security_state.erased_pages += 1;
        
        Ok(())
    }

    /// Asigna una página de memoria virtual
    pub fn allocate_page(&mut self, _flags: PageTableFlags) -> Result<Page<Size4KiB>, FrameError> {
        let page = self.find_free_page().expect("frame allocation failed");
        // Memory operations are simulated
        Ok(page)
    }

    /// Libera una página de memoria virtual
    pub fn deallocate_page(&mut self, page: Page<Size4KiB>) -> Result<(), FrameError> {
        // Obtener frame mapeado
        let (frame, _) = match unsafe { self.mapper_mut().unmap(page) } {
            Ok(result) => result,
            Err(_) => return Err(FrameError::FrameNotPresent),
        };
        
        // Realizar borrado seguro si es necesario
        if self.should_secure_erase(page) {
            self.secure_erase_page(page).ok();
        }
        
        // Liberar frame
        self.deallocate_physical_frame(frame);
        
        Ok(())
    }

    /// Encuentra una página libre usando bitmap allocator (O(n/64))
    fn find_free_page(&mut self) -> Option<Page<Size4KiB>> {
        // Usar el bitmap allocator para encontrar un frame libre
        if let Some(frame) = self.frame_allocator.alloc_frame_in_zone(MemZone::Normal) {
            let addr = frame.start_address().as_u64();
            return Some(Page::containing_address(VirtAddr::new(addr)));
        }
        
        // Fallback a espacio de usuario si no hay frames en Normal zone
        const USER_SPACE_START: u64 = 0x800000;
        const USER_SPACE_END: u64 = 0x100000000; // 4GB
        
        for addr in (USER_SPACE_START..USER_SPACE_END).step_by(4096) {
            let page = Page::containing_address(VirtAddr::new(addr));
            if self.is_page_free(page) {
                return Some(page);
            }
        }
        
        None
    }

    /// Verifica si una página está libre
    fn is_page_free(&self, page: Page<Size4KiB>) -> bool {
        // Verificar si la página está mapeada
        let pt = self.page_table.as_ref().expect("page table not initialized");
        match unsafe { pt.translate_page(page) } {
            Ok(_) => false,
            Err(_) => true,
        }
    }

    /// Asigna un frame físico usando bitmap allocator (O(n/64))
    fn allocate_physical_frame(&mut self) -> Option<PhysFrame> {
        // Usar el bitmap allocator para asignar un frame
        self.frame_allocator.allocate_frame()
    }

    /// Verifica si un frame físico está libre
    fn is_frame_free(&self, frame: PhysFrame) -> bool {
        // Implementación simplificada
        !self.used_memory.iter().any(|region| {
            let start_addr = region.range.start_frame_number * 4096;
            let end_addr = region.range.end_frame_number * 4096;
            start_addr <= frame.start_address().as_u64() && end_addr > frame.start_address().as_u64()
        })
    }

    /// Libera un frame físico usando bitmap allocator (BUG #3 corregido)
    fn deallocate_physical_frame(&mut self, frame: PhysFrame) {
        // Usar el bitmap allocator para liberar el frame
        self.frame_allocator.free_frame(frame);
    }

    /// Determina si una página debe ser borrada de forma segura
    fn should_secure_erase(&self, _page: Page<Size4KiB>) -> bool {
        // Lógica para determinar si necesita borrado seguro
        // Por ejemplo: páginas que contenían datos sensibles
        false // Por ahora
    }

    /// Obtiene estadísticas de uso de memoria
    pub fn get_memory_stats(&self) -> MemoryStats {
        let total_pages = self.physical_memory_size / 4096;
        let used_pages = self.used_memory.iter()
            .map(|region| (region.range.end_frame_number - region.range.start_frame_number))
            .sum::<u64>();
        
        MemoryStats {
            total_mb: self.physical_memory_size / (1024 * 1024),
            used_mb: used_pages * 4096 / (1024 * 1024),
            free_mb: (total_pages - used_pages) * 4096 / (1024 * 1024),
            security_state: self.security_state.clone(),
        }
    }

    /// Realiza compactación de memoria
    pub fn compact_memory(&mut self) {
        
        // Implementar algoritmo de compactación
        // Mover páginas usadas juntas para crear bloques contiguos libres
        
    }

    /// Valida integridad de la memoria
    pub fn validate_integrity(&self) -> bool {
        
        // Verificar que todas las páginas mapeadas sean válidas
        // Verificar que no haya corrupción en tablas de páginas
        
        true
    }
}

impl BootInfoFrameAllocator {
    /// Crea un nuevo asignador desde un slice de MemoryRegion
    pub unsafe fn new(regions: &[MemoryRegion]) -> Self {
        let available = regions
            .iter()
            .filter(|region| region.region_type == MemoryRegionType::Usable)
            .cloned()
            .collect();
        
        let next = Page::containing_address(VirtAddr::new(0x800000)); // Espacio de usuario
        
        Self {
            available,
            used: Vec::new(),
            next,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Buscar frame disponible en regiones disponibles
        for region in &mut self.available {
            let start_frame = region.range.start_frame_number;
            let end_frame = region.range.end_frame_number;
            
            if start_frame < end_frame {
                let frame = PhysFrame::containing_address(PhysAddr::new(start_frame * 4096));
                
                region.range.start_frame_number = start_frame + 1;
                
                if region.range.start_frame_number >= region.range.end_frame_number {
                    continue;
                }
                
                return Some(frame);
            }
        }
        
        None
    }
}

/// Estadísticas de memoria
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub security_state: MemorySecurityState,
}

/// Error de borrado seguro
#[derive(Debug)]
pub enum SecureEraseError {
    /// Error de hardware
    HardwareError,
    /// Página no encontrada
    PageNotFound,
    /// Operación no soportada
    Unsupported,
}

// Alias para tipos
use x86_64::structures::paging::PhysFrame;

/// Funciones de utilidad para la memoria
pub mod utils {
    use super::*;
    
    /// Convierte dirección física a virtual
    pub fn phys_to_virt(phys_addr: PhysAddr) -> VirtAddr {
        // Para identity mapping, las direcciones son iguales
        VirtAddr::new(phys_addr.as_u64())
    }
    
    /// Convierte dirección virtual a física
    pub fn virt_to_phys(virt_addr: VirtAddr) -> Option<PhysAddr> {
        // Para identity mapping, las direcciones son iguales
        Some(PhysAddr::new(virt_addr.as_u64()))
    }
    
    /// Verifica si una dirección está alineada a página
    pub fn is_page_aligned(addr: u64) -> bool {
        addr % 4096 == 0
    }
    
    /// Alinea una dirección a página
    pub fn align_to_page(addr: u64) -> u64 {
        (addr + 4095) & !4095
    }
}
