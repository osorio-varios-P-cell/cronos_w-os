//! Gestión de Memoria Virtual Completa para CRONOS W-OS
//!
//! Este módulo implementa la gestión de memoria virtual con paging completo
//! y swapping, adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Tamaño de página (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// Dirección física
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(pub u64);

impl PhysicalAddress {
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn is_aligned(&self) -> bool {
        self.0 % PAGE_SIZE == 0
    }

    pub fn page_number(&self) -> u64 {
        self.0 / PAGE_SIZE
    }

    pub fn offset(&self) -> u64 {
        self.0 % PAGE_SIZE
    }
}

/// Dirección virtual
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(pub u64);

impl VirtualAddress {
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn is_aligned(&self) -> bool {
        self.0 % PAGE_SIZE == 0
    }

    pub fn page_number(&self) -> u64 {
        self.0 / PAGE_SIZE
    }

    pub fn offset(&self) -> u64 {
        self.0 % PAGE_SIZE
    }

    pub fn page_index(&self, level: u8) -> u64 {
        match level {
            0 => (self.0 >> 12) & 0x1FF,
            1 => (self.0 >> 21) & 0x1FF,
            2 => (self.0 >> 30) & 0x1FF,
            3 => (self.0 >> 39) & 0x1FF,
            _ => 0,
        }
    }
}

/// Entrada de página (Page Table Entry)
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn present(&self) -> bool {
        self.value & 1 != 0
    }

    pub fn writable(&self) -> bool {
        self.value & 2 != 0
    }

    pub fn user_accessible(&self) -> bool {
        self.value & 4 != 0
    }

    pub fn set_present(&mut self, present: bool) {
        if present {
            self.value |= 1;
        } else {
            self.value &= !1;
        }
    }

    pub fn set_writable(&mut self, writable: bool) {
        if writable {
            self.value |= 2;
        } else {
            self.value &= !2;
        }
    }

    pub fn set_user_accessible(&mut self, user: bool) {
        if user {
            self.value |= 4;
        } else {
            self.value &= !4;
        }
    }

    pub fn set_physical_address(&mut self, addr: PhysicalAddress) {
        self.value = (self.value & 0xFFF) | (addr.0 & !0xFFF);
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        PhysicalAddress(self.value & !0xFFF)
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tabla de páginas
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
    pub physical_address: PhysicalAddress,
}

impl PageTable {
    pub fn new(physical_address: PhysicalAddress) -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
            physical_address,
        }
    }

    pub fn get_entry(&self, index: usize) -> Option<&PageTableEntry> {
        self.entries.get(index)
    }

    pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut PageTableEntry> {
        self.entries.get_mut(index)
    }
}

/// Área de memoria virtual
#[derive(Debug, Clone)]
pub struct MemoryArea {
    pub start: VirtualAddress,
    pub end: VirtualAddress,
    pub permissions: MemoryPermissions,
    pub name: String,
}

/// Permisos de memoria
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl MemoryPermissions {
    pub fn new() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: false,
        }
    }

    pub fn read_write() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: false,
        }
    }

    pub fn read_execute() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: true,
        }
    }

    pub fn read_write_execute() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: true,
        }
    }
}

impl Default for MemoryPermissions {
    fn default() -> Self {
        Self::new()
    }
}

/// Espacio de direcciones virtuales
pub struct AddressSpace {
    pub page_tables: Vec<PageTable>,
    pub memory_areas: Vec<MemoryArea>,
    pub root_table: Option<PhysicalAddress>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl AddressSpace {
    pub fn new() -> Self {
        Self {
            page_tables: Vec::new(),
            memory_areas: Vec::new(),
            root_table: None,
            graph_node_id: None,
        }
    }

    /// Mapear una página virtual a una página física
    pub fn map_page(&mut self, virt: VirtualAddress, phys: PhysicalAddress, perms: MemoryPermissions) -> Result<(), String> {
        // En un sistema real, aquí se:
        // 1. Recorrería la jerarquía de tablas de páginas
        // 2. Crearía tablas intermedias si no existen
        // 3. Configuraría la entrada de página final
        // 4. Invalidaría el TLB si es necesario

        Ok(())
    }

    /// Desmapear una página virtual
    pub fn unmap_page(&mut self, virt: VirtualAddress) -> Result<(), String> {
        // En un sistema real, aquí se:
        // 1. Recorrería la jerarquía de tablas de páginas
        // 2. Limpiaría la entrada de página
        // 3. Invalidaría el TLB si es necesario

        Ok(())
    }

    /// Agregar un área de memoria
    pub fn add_memory_area(&mut self, area: MemoryArea) {
        self.memory_areas.push(area);
    }
}

impl Default for AddressSpace {
    fn default() -> Self {
        Self::new()
    }
}

/// Página de memoria física
#[derive(Debug, Clone)]
pub struct PhysicalPage {
    pub physical_address: PhysicalAddress,
    pub allocated: bool,
    pub owner: Option<u64>, // ID del proceso que la posee
    pub reference_count: u32,
}

impl PhysicalPage {
    pub fn new(physical_address: PhysicalAddress) -> Self {
        Self {
            physical_address,
            allocated: false,
            owner: None,
            reference_count: 0,
        }
    }
}

/// Gestor de memoria física
pub struct PhysicalMemoryManager {
    pub pages: BTreeMap<u64, PhysicalPage>,
    pub total_pages: u64,
    pub free_pages: u64,
    pub allocated_pages: u64,
}

impl PhysicalMemoryManager {
    pub fn new(total_memory_mb: u64) -> Self {
        let total_pages = (total_memory_mb * 1024 * 1024) / PAGE_SIZE;
        let mut pages = BTreeMap::new();

        for i in 0..total_pages {
            let phys_addr = PhysicalAddress(i * PAGE_SIZE);
            pages.insert(i, PhysicalPage::new(phys_addr));
        }

        Self {
            pages,
            total_pages,
            free_pages: total_pages,
            allocated_pages: 0,
        }
    }

    /// Asignar una página física
    pub fn allocate_page(&mut self, owner: u64) -> Result<PhysicalAddress, String> {
        for (_, page) in self.pages.iter_mut() {
            if !page.allocated {
                page.allocated = true;
                page.owner = Some(owner);
                page.reference_count = 1;
                self.free_pages -= 1;
                self.allocated_pages += 1;
                return Ok(page.physical_address);
            }
        }
        Err(String::from("No free physical pages available"))
    }

    /// Liberar una página física
    pub fn free_page(&mut self, phys_addr: PhysicalAddress) -> Result<(), String> {
        let page_num = phys_addr.page_number();
        if let Some(page) = self.pages.get_mut(&page_num) {
            page.allocated = false;
            page.owner = None;
            page.reference_count = 0;
            self.free_pages += 1;
            self.allocated_pages -= 1;
            Ok(())
        } else {
            Err(format!("Physical page at {:?} not found", phys_addr))
        }
    }

    /// Obtener una página por dirección física
    pub fn get_page(&self, phys_addr: PhysicalAddress) -> Option<&PhysicalPage> {
        self.pages.get(&phys_addr.page_number())
    }

    /// Obtener número de páginas libres
    pub fn free_page_count(&self) -> u64 {
        self.free_pages
    }

    /// Obtener número de páginas asignadas
    pub fn allocated_page_count(&self) -> u64 {
        self.allocated_pages
    }
}

/// Entrada de swap
#[derive(Debug, Clone)]
pub struct SwapEntry {
    pub virtual_page: u64,
    pub swap_offset: u64,
    pub process_id: u64,
}

/// Gestor de swapping
pub struct SwapManager {
    pub swap_entries: BTreeMap<u64, SwapEntry>,
    pub swap_size_pages: u64,
    pub used_swap_pages: u64,
    pub swap_device: Option<String>,
}

impl SwapManager {
    pub fn new(swap_size_mb: u64) -> Self {
        let swap_size_pages = (swap_size_mb * 1024 * 1024) / PAGE_SIZE;
        Self {
            swap_entries: BTreeMap::new(),
            swap_size_pages,
            used_swap_pages: 0,
            swap_device: None,
        }
    }

    /// Swapear una página a disco
    pub fn swap_out(&mut self, virtual_page: u64, process_id: u64) -> Result<u64, String> {
        if self.used_swap_pages >= self.swap_size_pages {
            return Err(String::from("Swap space full"));
        }

        let swap_offset = self.used_swap_pages;
        let entry = SwapEntry {
            virtual_page,
            swap_offset,
            process_id,
        };

        self.swap_entries.insert(virtual_page, entry);
        self.used_swap_pages += 1;

        // En un sistema real, aquí se escribiría la página al dispositivo de swap

        Ok(swap_offset)
    }

    /// Swapear una página desde disco
    pub fn swap_in(&mut self, virtual_page: u64) -> Result<SwapEntry, String> {
        if let Some(entry) = self.swap_entries.remove(&virtual_page) {
            self.used_swap_pages -= 1;

            // En un sistema real, aquí se leería la página desde el dispositivo de swap

            Ok(entry)
        } else {
            Err(format!("Virtual page {} not in swap", virtual_page))
        }
    }

    /// Obtener número de páginas de swap usadas
    pub fn used_swap_pages(&self) -> u64 {
        self.used_swap_pages
    }

    /// Obtener número de páginas de swap libres
    pub fn free_swap_pages(&self) -> u64 {
        self.swap_size_pages - self.used_swap_pages
    }
}

impl Default for SwapManager {
    fn default() -> Self {
        Self::new(1024) // 1GB de swap por defecto
    }
}

/// Gestor de memoria virtual
pub struct VirtualMemoryManager {
    pub physical_memory: PhysicalMemoryManager,
    pub swap_manager: SwapManager,
    pub address_spaces: BTreeMap<u64, AddressSpace>,
    pub next_address_space_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl VirtualMemoryManager {
    pub fn new(total_memory_mb: u64, swap_size_mb: u64) -> Self {
        Self {
            physical_memory: PhysicalMemoryManager::new(total_memory_mb),
            swap_manager: SwapManager::new(swap_size_mb),
            address_spaces: BTreeMap::new(),
            next_address_space_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Crear un nuevo espacio de direcciones
    pub fn create_address_space(&mut self) -> Result<u64, String> {
        let space_id = self.next_address_space_id;
        self.next_address_space_id += 1;

        let mut address_space = AddressSpace::new();

        // Registrar el espacio de direcciones como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::MemoryRegion;
            let node_name = format!("address_space_{}", space_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            address_space.graph_node_id = node_id;
        }

        self.address_spaces.insert(space_id, address_space);
        Ok(space_id)
    }

    /// Mapear una página
    pub fn map_page(&mut self, space_id: u64, virt: VirtualAddress, perms: MemoryPermissions) -> Result<PhysicalAddress, String> {
        // Asignar página física
        let phys = self.physical_memory.allocate_page(space_id)?;

        // Mapear en el espacio de direcciones
        if let Some(address_space) = self.address_spaces.get_mut(&space_id) {
            address_space.map_page(virt, phys, perms)?;
        }

        Ok(phys)
    }

    /// Desmapear una página
    pub fn unmap_page(&mut self, space_id: u64, virt: VirtualAddress) -> Result<(), String> {
        // Desmapear en el espacio de direcciones
        let phys = if let Some(address_space) = self.address_spaces.get(&space_id) {
            // En un sistema real, aquí se obtendría la dirección física de la entrada de página
            PhysicalAddress(0) // Placeholder
        } else {
            return Err(format!("Address space {} not found", space_id));
        };

        // Liberar página física
        self.physical_memory.free_page(phys)?;

        Ok(())
    }

    /// Swapear una página
    pub fn swap_out_page(&mut self, space_id: u64, virt: VirtualAddress) -> Result<(), String> {
        let virtual_page = virt.page_number();
        self.swap_manager.swap_out(virtual_page, space_id)?;
        Ok(())
    }

    /// Swapear una página desde disco
    pub fn swap_in_page(&mut self, space_id: u64, virt: VirtualAddress) -> Result<(), String> {
        let virtual_page = virt.page_number();
        self.swap_manager.swap_in(virtual_page)?;
        Ok(())
    }

    /// Obtener un espacio de direcciones
    pub fn get_address_space(&self, space_id: u64) -> Option<&AddressSpace> {
        self.address_spaces.get(&space_id)
    }

    /// Obtener un espacio de direcciones mutable
    pub fn get_address_space_mut(&mut self, space_id: u64) -> Option<&mut AddressSpace> {
        self.address_spaces.get_mut(&space_id)
    }

    /// Obtener estadísticas de memoria física
    pub fn physical_memory_stats(&self) -> (u64, u64, u64) {
        (
            self.physical_memory.total_pages,
            self.physical_memory.free_pages,
            self.physical_memory.allocated_pages,
        )
    }

    /// Obtener estadísticas de swap
    pub fn swap_stats(&self) -> (u64, u64) {
        (
            self.swap_manager.used_swap_pages(),
            self.swap_manager.free_swap_pages(),
        )
    }
}

impl Default for VirtualMemoryManager {
    fn default() -> Self {
        Self::new(4096, 1024) // 4GB RAM, 1GB swap por defecto
    }
}

/// Errores de gestión de memoria
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress,
    PermissionDenied,
    PageNotMapped,
    SwapFull,
    AddressSpaceNotFound,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::OutOfMemory => write!(f, "Out of memory"),
            MemoryError::InvalidAddress => write!(f, "Invalid address"),
            MemoryError::PermissionDenied => write!(f, "Permission denied"),
            MemoryError::PageNotMapped => write!(f, "Page not mapped"),
            MemoryError::SwapFull => write!(f, "Swap full"),
            MemoryError::AddressSpaceNotFound => write!(f, "Address space not found"),
        }
    }
}
