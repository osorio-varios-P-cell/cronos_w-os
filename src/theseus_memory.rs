//! Técnicas de Theseus para Memory Manager (Single Address Space)
//!
//! Este módulo incorpora las técnicas de single address space de Theseus OS
//! para optimizar el gestor de memoria de CRONOS W-OS, adaptadas a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Región de memoria en single address space (inspirado en Theseus)
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub region_id: u64,
    pub start: u64,
    pub size: u64,
    pub owner_id: u64, // ID del proceso dueño
    pub permissions: MemoryPermissions,
    pub region_type: MemoryRegionType,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

/// Permisos de memoria
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryPermissions {
    /// Solo lectura
    ReadOnly,
    /// Lectura y escritura
    ReadWrite,
    /// Ejecutable
    Execute,
    /// Lectura y ejecutable
    ReadExecute,
    /// Lectura, escritura y ejecutable
    ReadWriteExecute,
    /// Sin permisos
    None,
}

/// Tipo de región de memoria
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryRegionType {
    /// Código
    Code,
    /// Datos
    Data,
    /// Heap
    Heap,
    /// Stack
    Stack,
    /// Mapeo de dispositivo
    DeviceMapping,
    /// Compartida
    Shared,
    /// Reservada
    Reserved,
}

impl MemoryRegion {
    pub fn new(region_id: u64, start: u64, size: u64, owner_id: u64) -> Self {
        Self {
            region_id,
            start,
            size,
            owner_id,
            permissions: MemoryPermissions::ReadWrite,
            region_type: MemoryRegionType::Data,
            graph_node_id: None,
        }
    }

    /// Verificar si una dirección está dentro de la región
    pub fn contains(&self, address: u64) -> bool {
        address >= self.start && address < (self.start + self.size)
    }

    /// Obtener el rango de direcciones
    pub fn range(&self) -> (u64, u64) {
        (self.start, self.start + self.size)
    }
}

/// Gestor de memoria con single address space (inspirado en Theseus)
#[derive(Debug, Clone)]
pub struct SingleAddressSpaceManager {
    pub regions: BTreeMap<u64, MemoryRegion>,
    pub next_region_id: u64,
    pub base_address: u64,
    pub max_address: u64,
    pub free_regions: Vec<(u64, u64)>, // (start, size)
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl SingleAddressSpaceManager {
    pub fn new(base_address: u64, max_address: u64) -> Self {
        let mut free_regions = Vec::new();
        free_regions.push((base_address, max_address - base_address));

        Self {
            regions: BTreeMap::new(),
            next_region_id: 1,
            base_address,
            max_address,
            free_regions,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Asignar una región de memoria
    pub fn allocate_region(&mut self, owner_id: u64, size: u64, permissions: MemoryPermissions, region_type: MemoryRegionType) -> Result<u64, String> {
        // Encontrar una región libre del tamaño adecuado
        let region_index = self.free_regions.iter().position(|(start, region_size)| *region_size >= size);

        if let Some(index) = region_index {
            let (start, region_size) = self.free_regions.remove(index);

            // Crear la nueva región
            let region_id = self.next_region_id;
            self.next_region_id += 1;

            let mut region = MemoryRegion::new(region_id, start, size, owner_id);
            region.permissions = permissions;
            region.region_type = region_type;

            // Registrar la región como nodo en el grafo
            if let Some(ref graph_kernel) = self.graph_kernel {
                use crate::graph_kernel::{NodeType, GraphKernel};
                let node_type = NodeType::MemoryRegion;
                let node_name = format!("sas_region_{}", region_id);
                let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                    gk.create_node(node_type, node_name)
                });
                region.graph_node_id = node_id;
            }

            self.regions.insert(region_id, region);

            // Actualizar las regiones libres
            if region_size > size {
                self.free_regions.push((start + size, region_size - size));
            }

            Ok(start)
        } else {
            Err(String::from("No free region available"))
        }
    }

    /// Liberar una región de memoria
    pub fn free_region(&mut self, region_id: u64) -> Result<(), String> {
        if let Some(region) = self.regions.remove(&region_id) {
            // Agregar la región liberada a las regiones libres
            self.free_regions.push((region.start, region.size));
            Ok(())
        } else {
            Err(format!("Region {} not found", region_id))
        }
    }

    /// Mapear una dirección física a una dirección virtual
    pub fn map_physical(&mut self, physical: u64, size: u64, owner_id: u64) -> Result<u64, String> {
        self.allocate_region(owner_id, size, MemoryPermissions::ReadWrite, MemoryRegionType::DeviceMapping)
    }

    /// Desmapear una región
    pub fn unmap(&mut self, virtual_address: u64) -> Result<(), String> {
        // Encontrar la región que contiene la dirección
        let region_id = self.regions.iter()
            .find(|(_, region)| region.contains(virtual_address))
            .map(|(id, _)| *id);

        if let Some(region_id) = region_id {
            self.free_region(region_id)
        } else {
            Err(format!("No region found at address {}", virtual_address))
        }
    }

    /// Cambiar los permisos de una región
    pub fn change_permissions(&mut self, region_id: u64, new_permissions: MemoryPermissions) -> Result<(), String> {
        if let Some(region) = self.regions.get_mut(&region_id) {
            region.permissions = new_permissions;
            Ok(())
        } else {
            Err(format!("Region {} not found", region_id))
        }
    }

    /// Obtener una región por dirección
    pub fn get_region_by_address(&self, address: u64) -> Option<&MemoryRegion> {
        self.regions.values().find(|region| region.contains(address))
    }

    /// Obtener una región por ID
    pub fn get_region(&self, region_id: u64) -> Option<&MemoryRegion> {
        self.regions.get(&region_id)
    }

    /// Obtener todas las regiones de un propietario
    pub fn get_regions_by_owner(&self, owner_id: u64) -> Vec<&MemoryRegion> {
        self.regions.values()
            .filter(|region| region.owner_id == owner_id)
            .collect()
    }

    /// Verificar si una dirección es válida
    pub fn is_valid_address(&self, address: u64) -> bool {
        address >= self.base_address && address < self.max_address &&
            self.regions.values().any(|region| region.contains(address))
    }

    /// Obtener estadísticas de memoria
    pub fn stats(&self) -> MemoryStats {
        let total_used = self.regions.values().map(|r| r.size).sum();
        let total_free = self.free_regions.iter().map(|(_, size)| *size).sum();
        let region_count = self.regions.len();

        MemoryStats {
            total_size: self.max_address - self.base_address,
            used_size: total_used,
            free_size: total_free,
            region_count,
        }
    }

    /// Defragmentar el espacio de direcciones
    pub fn defragment(&mut self) {
        // Ordenar las regiones libres por dirección de inicio
        self.free_regions.sort_by_key(|(start, _)| *start);

        // Fusionar regiones libres adyacentes
        let mut merged = Vec::new();
        if let Some((first_start, first_size)) = self.free_regions.first() {
            let mut current_start = *first_start;
            let mut current_end = *first_start + first_size;

            for (start, size) in self.free_regions.iter().skip(1) {
                if *start == current_end {
                    // Regiones adyacentes, fusionar
                    current_end += size;
                } else {
                    merged.push((current_start, current_end - current_start));
                    current_start = *start;
                    current_end = *start + size;
                }
            }
            merged.push((current_start, current_end - current_start));
        }

        self.free_regions = merged;
    }
}

impl Default for SingleAddressSpaceManager {
    fn default() -> Self {
        Self::new(0x1000, 0x7FFFFFFFFFFF) // Rango típico de direcciones virtuales x86_64
    }
}

/// Estadísticas de memoria
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_size: u64,
    pub used_size: u64,
    pub free_size: u64,
    pub region_count: usize,
}

impl MemoryStats {
    /// Obtener el porcentaje de uso
    pub fn usage_percentage(&self) -> f32 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.used_size as f32 / self.total_size as f32) * 100.0
        }
    }
}

/// Gestor de memoria compartida (inspirado en Theseus)
pub struct SharedMemoryManager {
    pub shared_regions: BTreeMap<u64, SharedMemoryRegion>,
    pub next_shared_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

/// Región de memoria compartida
#[derive(Debug, Clone)]
pub struct SharedMemoryRegion {
    pub shared_id: u64,
    pub address: u64,
    pub size: u64,
    pub references: BTreeSet<u64>, // IDs de procesos que referencian esta región
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl SharedMemoryManager {
    pub fn new() -> Self {
        Self {
            shared_regions: BTreeMap::new(),
            next_shared_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Crear una región de memoria compartida
    pub fn create_shared(&mut self, address: u64, size: u64) -> u64 {
        let shared_id = self.next_shared_id;
        self.next_shared_id += 1;

        let mut region = SharedMemoryRegion {
            shared_id,
            address,
            size,
            references: BTreeSet::new(),
            graph_node_id: None,
        };

        // Registrar la región compartida como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::MemoryRegion;
            let node_name = format!("shared_mem_{}", shared_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            region.graph_node_id = node_id;
        }

        self.shared_regions.insert(shared_id, region);
        shared_id
    }

    /// Agregar una referencia a una región compartida
    pub fn add_reference(&mut self, shared_id: u64, process_id: u64) -> Result<(), String> {
        if let Some(region) = self.shared_regions.get_mut(&shared_id) {
            region.references.insert(process_id);
            Ok(())
        } else {
            Err(format!("Shared region {} not found", shared_id))
        }
    }

    /// Remover una referencia de una región compartida
    pub fn remove_reference(&mut self, shared_id: u64, process_id: u64) -> Result<bool, String> {
        if let Some(region) = self.shared_regions.get_mut(&shared_id) {
            region.references.remove(&process_id);
            Ok(region.references.is_empty())
        } else {
            Err(format!("Shared region {} not found", shared_id))
        }
    }

    /// Obtener una región compartida
    pub fn get_shared(&self, shared_id: u64) -> Option<&SharedMemoryRegion> {
        self.shared_regions.get(&shared_id)
    }

    /// Eliminar una región compartida
    pub fn remove_shared(&mut self, shared_id: u64) -> Result<(), String> {
        if self.shared_regions.remove(&shared_id).is_some() {
            Ok(())
        } else {
            Err(format!("Shared region {} not found", shared_id))
        }
    }

    /// Obtener el número de referencias
    pub fn reference_count(&self, shared_id: u64) -> Option<usize> {
        self.shared_regions.get(&shared_id).map(|r| r.references.len())
    }
}

impl Default for SharedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del gestor de memoria de Theseus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheseusMemoryError {
    RegionNotFound,
    OutOfMemory,
    InvalidAddress,
    PermissionDenied,
    AlreadyMapped,
}

impl fmt::Display for TheseusMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TheseusMemoryError::RegionNotFound => write!(f, "Region not found"),
            TheseusMemoryError::OutOfMemory => write!(f, "Out of memory"),
            TheseusMemoryError::InvalidAddress => write!(f, "Invalid address"),
            TheseusMemoryError::PermissionDenied => write!(f, "Permission denied"),
            TheseusMemoryError::AlreadyMapped => write!(f, "Already mapped"),
        }
    }
}
