//! Address Space Module
//! 
//! This module implements separate address spaces for processes with virtual memory management.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Flags de memoria
#[derive(Debug, Clone, Copy)]
pub struct MemoryFlags {
    /// Lectura permitida
    pub readable: bool,
    /// Escritura permitida
    pub writable: bool,
    /// Ejecución permitida
    pub executable: bool,
    /// Mapeado por usuario
    pub user_accessible: bool,
}

impl MemoryFlags {
    /// Crear nuevos flags
    pub fn new(readable: bool, writable: bool, executable: bool, user_accessible: bool) -> Self {
        Self {
            readable,
            writable,
            executable,
            user_accessible,
        }
    }

    /// Solo lectura
    pub fn read_only() -> Self {
        Self::new(true, false, false, true)
    }

    /// Lectura y escritura
    pub fn read_write() -> Self {
        Self::new(true, true, false, true)
    }

    /// Lectura y ejecución
    pub fn read_execute() -> Self {
        Self::new(true, false, true, true)
    }

    /// Lectura, escritura y ejecución
    pub fn read_write_execute() -> Self {
        Self::new(true, true, true, true)
    }

    /// Kernel solo lectura
    pub fn kernel_read_only() -> Self {
        Self::new(true, false, false, false)
    }

    /// Kernel lectura y escritura
    pub fn kernel_read_write() -> Self {
        Self::new(true, true, false, false)
    }
}

impl Default for MemoryFlags {
    fn default() -> Self {
        Self::read_write()
    }
}

/// Región de memoria
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Dirección virtual de inicio
    pub virt_start: u64,
    /// Dirección virtual de fin
    pub virt_end: u64,
    /// Dirección física de inicio
    pub phys_start: u64,
    /// Tamaño de la región
    pub size: u64,
    /// Flags de memoria
    pub flags: MemoryFlags,
    /// Nombre de la región
    pub name: String,
}

impl MemoryRegion {
    /// Crear nueva región de memoria
    pub fn new(virt_start: u64, phys_start: u64, size: u64, flags: MemoryFlags, name: String) -> Self {
        Self {
            virt_start,
            virt_end: virt_start + size,
            phys_start,
            size,
            flags,
            name,
        }
    }

    /// Verificar si una dirección está en la región
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.virt_start && addr < self.virt_end
    }

    /// Verificar si hay solapamiento con otra región
    pub fn overlaps(&self, other: &MemoryRegion) -> bool {
        self.virt_start < other.virt_end && self.virt_end > other.virt_start
    }
}

/// Entrada de tabla de páginas
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    /// Dirección física de la página
    pub physical_address: u64,
    /// Flags
    pub flags: MemoryFlags,
    /// Presente
    pub present: bool,
}

impl PageTableEntry {
    /// Crear nueva entrada
    pub fn new(physical_address: u64, flags: MemoryFlags, present: bool) -> Self {
        Self {
            physical_address,
            flags,
            present,
        }
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new(0, MemoryFlags::default(), false)
    }
}

/// Tabla de páginas
#[derive(Debug, Clone)]
pub struct PageTable {
    /// Entradas de la tabla
    pub entries: Vec<Option<PageTableEntry>>,
    /// Nivel de la tabla (0 = PML4, 1 = PDPT, 2 = PD, 3 = PT)
    pub level: u8,
}

impl PageTable {
    /// Crear nueva tabla de páginas
    pub fn new(level: u8) -> Self {
        Self {
            entries: vec![None; 512], // 512 entradas por tabla en x86_64
            level,
        }
    }

    /// Obtener entrada
    pub fn get_entry(&self, index: usize) -> Option<PageTableEntry> {
        self.entries.get(index).copied().flatten()
    }

    /// Establecer entrada
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        if index < self.entries.len() {
            self.entries[index] = Some(entry);
        }
    }

    /// Limpiar entrada
    pub fn clear_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries[index] = None;
        }
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Espacio de direcciones
#[derive(Debug, Clone)]
pub struct AddressSpace {
    /// ID del espacio de direcciones
    pub id: u32,
    /// Tabla de páginas raíz (PML4)
    pub root_page_table: PageTable,
    /// Regiones de memoria
    pub regions: Vec<MemoryRegion>,
    /// Heap actual
    pub heap_start: u64,
    /// Heap actual (brk)
    pub heap_end: u64,
    /// Stack actual
    pub stack_start: u64,
    /// Stack actual
    pub stack_end: u64,
    /// Tamaño del espacio de direcciones
    pub size: u64,
}

impl AddressSpace {
    /// Crear nuevo espacio de direcciones
    pub fn new(id: u32) -> Self {
        Self {
            id,
            root_page_table: PageTable::new(0),
            regions: Vec::new(),
            heap_start: 0,
            heap_end: 0,
            stack_start: 0,
            stack_end: 0,
            size: 0x8000_0000_0000, // 8TB (espacio de usuario en x86_64)
        }
    }

    /// Agregar región de memoria
    pub fn add_region(&mut self, region: MemoryRegion) -> Result<(), String> {
        // Verificar solapamiento
        for existing in &self.regions {
            if region.overlaps(existing) {
                return Err(format!("Region overlaps with existing region: {}", existing.name));
            }
        }
        
        self.regions.push(region);
        Ok(())
    }

    /// Remover región de memoria
    pub fn remove_region(&mut self, virt_start: u64) -> Result<(), String> {
        let index = self.regions.iter()
            .position(|r| r.virt_start == virt_start)
            .ok_or_else(|| String::from("Region not found"))?;
        
        self.regions.remove(index);
        Ok(())
    }

    /// Obtener región por dirección
    pub fn get_region(&self, addr: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    /// Mapear página
    pub fn map_page(&mut self, virt_addr: u64, phys_addr: u64, flags: MemoryFlags) -> Result<(), String> {
        // En un sistema real, esto mapearía la página en la tabla de páginas
        let entry = PageTableEntry::new(phys_addr, flags, true);
        
        // Calcular índices de la tabla de páginas
        let pml4_index = ((virt_addr >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt_addr >> 30) & 0x1FF) as usize;
        let pd_index = ((virt_addr >> 21) & 0x1FF) as usize;
        let pt_index = ((virt_addr >> 12) & 0x1FF) as usize;
        
        // En un sistema real, esto recorrería las tablas de páginas
        // Para este ejemplo, solo mostramos los índices
        let _ = (pml4_index, pdpt_index, pd_index, pt_index);
        
        Ok(())
    }

    /// Desmapear página
    pub fn unmap_page(&mut self, virt_addr: u64) -> Result<(), String> {
        // En un sistema real, esto desmapearía la página
        Ok(())
    }

    /// Configurar heap
    pub fn setup_heap(&mut self, start: u64, size: u64) {
        self.heap_start = start;
        self.heap_end = start + size;
        
        let region = MemoryRegion::new(
            start,
            0, // Dirección física asignada dinámicamente
            size,
            MemoryFlags::read_write(),
            String::from("heap"),
        );
        
        let _ = self.add_region(region);
    }

    /// Configurar stack
    pub fn setup_stack(&mut self, start: u64, size: u64) {
        self.stack_start = start;
        self.stack_end = start + size;
        
        let region = MemoryRegion::new(
            start,
            0, // Dirección física asignada dinámicamente
            size,
            MemoryFlags::read_write(),
            String::from("stack"),
        );
        
        let _ = self.add_region(region);
    }

    /// Expandir heap (brk)
    pub fn expand_heap(&mut self, delta: i64) -> Result<u64, String> {
        let new_end = if delta >= 0 {
            self.heap_end + delta as u64
        } else {
            if self.heap_end < (-delta) as u64 {
                return Err(String::from("Heap underflow"));
            }
            self.heap_end - (-delta) as u64
        };
        
        self.heap_end = new_end;
        Ok(new_end)
    }

    /// Obtener tamaño de memoria usada
    pub fn get_used_memory(&self) -> u64 {
        self.regions.iter().map(|r| r.size).sum()
    }

    /// Clonar espacio de direcciones (para fork)
    pub fn clone(&self, new_id: u32) -> Self {
        let mut new_space = Self::new(new_id);
        
        // Clonar regiones
        new_space.regions = self.regions.clone();
        
        // Clonar heap y stack
        new_space.heap_start = self.heap_start;
        new_space.heap_end = self.heap_end;
        new_space.stack_start = self.stack_start;
        new_space.stack_end = self.stack_end;
        
        // En un sistema real, esto también clonaría las tablas de páginas
        // y copiaría el contenido de las páginas
        
        new_space
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Address Space Status\n");
        report.push_str("====================\n\n");
        
        report.push_str(&format!("ID: {}\n", self.id));
        report.push_str(&format!("Size: {} bytes ({} GB)\n", 
            self.size, self.size / (1024 * 1024 * 1024)));
        
        report.push_str(&format!("\nHeap: 0x{:X} - 0x{:X} ({} bytes)\n",
            self.heap_start, self.heap_end, self.heap_end - self.heap_start));
        report.push_str(&format!("Stack: 0x{:X} - 0x{:X} ({} bytes)\n",
            self.stack_start, self.stack_end, self.stack_end - self.stack_start));
        
        report.push_str(&format!("\nUsed Memory: {} bytes ({} MB)\n",
            self.get_used_memory(), self.get_used_memory() / (1024 * 1024)));
        
        report.push_str(&format!("\nMemory Regions: {}\n", self.regions.len()));
        for (i, region) in self.regions.iter().enumerate() {
            report.push_str(&format!(
                "  Region {}: 0x{:X} - 0x{:X} ({} bytes), Flags: R{}W{}X{}, User: {}, Name: {}\n",
                i, region.virt_start, region.virt_end, region.size,
                if region.flags.readable { "+" } else { "-" },
                if region.flags.writable { "+" } else { "-" },
                if region.flags.executable { "+" } else { "-" },
                if region.flags.user_accessible { "+" } else { "-" },
                region.name
            ));
        }
        
        report
    }
}

impl Default for AddressSpace {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Gestor de espacios de direcciones
pub struct AddressSpaceManager {
    /// Espacios de direcciones
    pub address_spaces: Vec<AddressSpace>,
    /// Siguiente ID disponible
    pub next_id: u32,
}

impl AddressSpaceManager {
    /// Crear nuevo gestor
    pub fn new() -> Self {
        Self {
            address_spaces: Vec::new(),
            next_id: 1,
        }
    }

    /// Crear nuevo espacio de direcciones
    pub fn create_address_space(&mut self) -> u32 {
        let id = self.next_id;
        let space = AddressSpace::new(id);
        self.address_spaces.push(space);
        self.next_id += 1;
        id
    }

    /// Obtener espacio de direcciones por ID
    pub fn get_address_space(&self, id: u32) -> Option<&AddressSpace> {
        self.address_spaces.iter().find(|a| a.id == id)
    }

    /// Obtener espacio de direcciones mutable por ID
    pub fn get_address_space_mut(&mut self, id: u32) -> Option<&mut AddressSpace> {
        self.address_spaces.iter_mut().find(|a| a.id == id)
    }

    /// Clonar espacio de direcciones
    pub fn clone_address_space(&mut self, id: u32) -> Result<u32, String> {
        let original = self.get_address_space(id)
            .ok_or_else(|| String::from("Address space not found"))?;
        
        let new_id = self.next_id;
        let cloned = original.clone(new_id);
        self.address_spaces.push(cloned);
        self.next_id += 1;
        
        Ok(new_id)
    }

    /// Remover espacio de direcciones
    pub fn remove_address_space(&mut self, id: u32) -> Result<(), String> {
        let index = self.address_spaces.iter()
            .position(|a| a.id == id)
            .ok_or_else(|| String::from("Address space not found"))?;
        
        self.address_spaces.remove(index);
        Ok(())
    }

    /// Obtener número de espacios de direcciones
    pub fn get_count(&self) -> usize {
        self.address_spaces.len()
    }
}

impl Default for AddressSpaceManager {
    fn default() -> Self {
        Self::new()
    }
}
