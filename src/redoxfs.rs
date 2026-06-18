//! RedoxFS - Filesystem ZFS-like de Redox adaptado a CRONOS W-OS
//!
//! Este módulo incorpora RedoxFS de Redox OS, un filesystem inspirado en ZFS
//! con copy-on-write, snapshots, data integrity, adaptado al VFS y grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;
use crate::filesystem::{VirtualFileSystem, VfsInode, FileType, FileMetadata, FilePermissions};

/// Bloque de datos con checksum (COW)
#[derive(Debug, Clone)]
pub struct CowBlock {
    pub block_id: u64,
    pub data: Vec<u8>,
    pub checksum: u64,
    pub ref_count: u32,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl CowBlock {
    pub fn new(block_id: u64, data: Vec<u8>) -> Self {
        let checksum = Self::calculate_checksum(&data);
        Self {
            block_id,
            data,
            checksum,
            ref_count: 1,
            graph_node_id: None,
        }
    }

    /// Calcular checksum del bloque
    fn calculate_checksum(data: &[u8]) -> u64 {
        // En un sistema real, aquí se usaría un algoritmo como CRC64 o SHA256
        data.iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64))
    }

    /// Verificar integridad del bloque
    pub fn verify_integrity(&self) -> bool {
        self.checksum == Self::calculate_checksum(&self.data)
    }
}

/// Snapshot del filesystem
#[derive(Debug, Clone)]
pub struct RedoxFSSnapshot {
    pub snapshot_id: u64,
    pub name: String,
    pub timestamp: u64,
    pub root_block_id: u64,
    pub metadata: SnapshotMetadata,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

/// Metadata del snapshot
#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    pub size: u64,
    pub block_count: u32,
    pub description: String,
}

/// Inodo RedoxFS
#[derive(Debug, Clone)]
pub struct RedoxFSInode {
    pub inode_id: u64,
    pub file_type: FileType,
    pub size: u64,
    pub blocks: Vec<u64>, // IDs de bloques COW
    pub permissions: FilePermissions,
    pub created_at: u64,
    pub modified_at: u64,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl RedoxFSInode {
    pub fn new(inode_id: u64, file_type: FileType) -> Self {
        Self {
            inode_id,
            file_type,
            size: 0,
            blocks: Vec::new(),
            permissions: FilePermissions::from_octal(0o644),
            created_at: 0,
            modified_at: 0,
            graph_node_id: None,
        }
    }

    /// Convertir a VfsInode
    pub fn to_vfs_inode(&self) -> VfsInode {
        let metadata = FileMetadata {
            file_type: self.file_type.clone(),
            size: self.size,
            permissions: self.permissions.clone(),
            owner_id: 0,
            group_id: 0,
            created_at: self.created_at,
            modified_at: self.modified_at,
            accessed_at: self.modified_at,
        };

        VfsInode {
            id: self.inode_id,
            name: String::new(),
            metadata,
            data: Vec::new(),
            blocks: self.blocks.clone(),
            parent_id: None,
            graph_node_id: self.graph_node_id,
        }
    }
}

/// Filesystem RedoxFS adaptado
#[derive(Debug, Clone)]
pub struct RedoxFS {
    pub blocks: BTreeMap<u64, CowBlock>,
    pub inodes: BTreeMap<u64, RedoxFSInode>,
    pub snapshots: BTreeMap<u64, RedoxFSSnapshot>,
    pub next_block_id: u64,
    pub next_inode_id: u64,
    pub next_snapshot_id: u64,
    pub root_inode_id: u64,
    pub vfs: VirtualFileSystem,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl RedoxFS {
    pub fn new() -> Self {
        let mut fs = Self {
            blocks: BTreeMap::new(),
            inodes: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            next_block_id: 1,
            next_inode_id: 1,
            next_snapshot_id: 1,
            root_inode_id: 1,
            vfs: VirtualFileSystem::new(),
            graph_kernel: None,
        };

        // Crear inodo raíz
        let root_inode = RedoxFSInode::new(1, FileType::Directory);
        fs.inodes.insert(1, root_inode);

        fs
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel.clone()));
        self.vfs.set_graph_kernel(graph_kernel);
    }

    /// Asignar un nuevo bloque COW
    pub fn allocate_block(&mut self, data: Vec<u8>) -> u64 {
        let block_id = self.next_block_id;
        self.next_block_id += 1;

        let mut block = CowBlock::new(block_id, data);

        // Registrar el bloque como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::MemoryRegion;
            let node_name = format!("redoxfs_block_{}", block_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            block.graph_node_id = node_id;
        }

        self.blocks.insert(block_id, block);
        block_id
    }

    /// Leer un bloque COW
    pub fn read_block(&self, block_id: u64) -> Option<&CowBlock> {
        self.blocks.get(&block_id)
    }

    /// Escribir un bloque con COW (copia si hay múltiples referencias)
    pub fn write_block(&mut self, block_id: u64, data: Vec<u8>) -> Result<u64, String> {
        if let Some(block) = self.blocks.get(&block_id) {
            if block.ref_count > 1 {
                // COW: crear un nuevo bloque
                let new_block_id = self.allocate_block(data);
                Ok(new_block_id)
            } else {
                // Sobrescribir el bloque existente
                let mut block = block.clone();
                block.data = data;
                block.checksum = CowBlock::calculate_checksum(&block.data);
                self.blocks.insert(block_id, block);
                Ok(block_id)
            }
        } else {
            Err(format!("Block {} not found", block_id))
        }
    }

    /// Crear un snapshot del filesystem
    pub fn create_snapshot(&mut self, name: String, description: String) -> u64 {
        let snapshot_id = self.next_snapshot_id;
        self.next_snapshot_id += 1;

        let metadata = SnapshotMetadata {
            size: self.blocks.values().map(|b| b.data.len() as u64).sum(),
            block_count: self.blocks.len() as u32,
            description,
        };

        let mut snapshot = RedoxFSSnapshot {
            snapshot_id,
            name,
            timestamp: 0, // En un sistema real, timestamp actual
            root_block_id: self.root_inode_id,
            metadata,
            graph_node_id: None,
        };

        // Registrar el snapshot como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("redoxfs_snapshot_{}", snapshot_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            snapshot.graph_node_id = node_id;
        }

        self.snapshots.insert(snapshot_id, snapshot);
        snapshot_id
    }

    /// Restaurar un snapshot
    pub fn restore_snapshot(&mut self, snapshot_id: u64) -> Result<(), String> {
        if let Some(snapshot) = self.snapshots.get(&snapshot_id) {
            // En un sistema real, aquí se:
            // 1. Restauraría el estado del filesystem al snapshot
            // 2. Mantendría los bloques COW para integridad
            // 3. Actualizaría el inodo raíz

            self.root_inode_id = snapshot.root_block_id;
            Ok(())
        } else {
            Err(format!("Snapshot {} not found", snapshot_id))
        }
    }

    /// Eliminar un snapshot
    pub fn delete_snapshot(&mut self, snapshot_id: u64) -> Result<(), String> {
        if self.snapshots.remove(&snapshot_id).is_some() {
            Ok(())
        } else {
            Err(format!("Snapshot {} not found", snapshot_id))
        }
    }

    /// Obtener un snapshot
    pub fn get_snapshot(&self, snapshot_id: u64) -> Option<&RedoxFSSnapshot> {
        self.snapshots.get(&snapshot_id)
    }

    /// Verificar integridad del filesystem
    pub fn verify_integrity(&self) -> bool {
        self.blocks.values().all(|block| block.verify_integrity())
    }

    /// Crear un archivo
    pub fn create_file(&mut self, parent_id: u64, name: String, file_type: FileType) -> Result<u64, String> {
        let inode_id = self.next_inode_id;
        self.next_inode_id += 1;

        let mut inode = RedoxFSInode::new(inode_id, file_type);

        // Registrar el inodo como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("redoxfs_inode_{}", inode_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            inode.graph_node_id = node_id;
        }

        self.inodes.insert(inode_id, inode);
        Ok(inode_id)
    }

    /// Leer un archivo
    pub fn read_file(&mut self, inode_id: u64, offset: u64, buffer: &mut [u8]) -> Result<usize, String> {
        if let Some(inode) = self.inodes.get(&inode_id) {
            let vfs_inode = inode.to_vfs_inode();
            self.vfs.read_file(vfs_inode.id, offset, buffer)
        } else {
            Err(format!("Inode {} not found", inode_id))
        }
    }

    /// Escribir a un archivo
    pub fn write_file(&mut self, inode_id: u64, offset: u64, buffer: &[u8]) -> Result<usize, String> {
        if let Some(inode) = self.inodes.get(&inode_id) {
            let vfs_inode = inode.to_vfs_inode();
            self.vfs.write_file(vfs_inode.id, offset, buffer)
        } else {
            Err(format!("Inode {} not found", inode_id))
        }
    }

    /// Obtener estadísticas
    pub fn stats(&self) -> RedoxFSStats {
        let total_blocks = self.blocks.len();
        let total_size = self.blocks.values().map(|b| b.data.len() as u64).sum();
        let total_inodes = self.inodes.len();
        let total_snapshots = self.snapshots.len();

        RedoxFSStats {
            total_blocks,
            total_size,
            total_inodes,
            total_snapshots,
            integrity_valid: self.verify_integrity(),
        }
    }
}

impl Default for RedoxFS {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de RedoxFS
#[derive(Debug, Clone)]
pub struct RedoxFSStats {
    pub total_blocks: usize,
    pub total_size: u64,
    pub total_inodes: usize,
    pub total_snapshots: usize,
    pub integrity_valid: bool,
}

/// Errores de RedoxFS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedoxFSError {
    BlockNotFound,
    InodeNotFound,
    SnapshotNotFound,
    IntegrityCheckFailed,
    OutOfSpace,
}

impl fmt::Display for RedoxFSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedoxFSError::BlockNotFound => write!(f, "Block not found"),
            RedoxFSError::InodeNotFound => write!(f, "Inode not found"),
            RedoxFSError::SnapshotNotFound => write!(f, "Snapshot not found"),
            RedoxFSError::IntegrityCheckFailed => write!(f, "Integrity check failed"),
            RedoxFSError::OutOfSpace => write!(f, "Out of space"),
        }
    }
}
