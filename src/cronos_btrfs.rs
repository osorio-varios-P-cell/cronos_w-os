//! Btrfs Driver de CRONOS original adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el driver de sistema de archivos Btrfs de CRONOS original,
//! adaptado al sistema de capabilities y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::BTreeMap;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Tipo de subvolumen Btrfs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubvolumeType {
    Root,
    Normal,
    Snapshot,
}

/// Subvolumen Btrfs
#[derive(Debug, Clone)]
pub struct BtrfsSubvolume {
    pub subvolume_id: u64,
    pub parent_id: u64,
    pub name: String,
    pub subvolume_type: SubvolumeType,
    pub uuid: [u8; 16],
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl BtrfsSubvolume {
    pub fn new(subvolume_id: u64, parent_id: u64, name: &str, subvolume_type: SubvolumeType) -> Self {
        Self {
            subvolume_id,
            parent_id,
            name: String::from(name),
            subvolume_type,
            uuid: [0; 16],
            graph_node_id: None,
        }
    }

    pub fn is_root(&self) -> bool {
        self.subvolume_type == SubvolumeType::Root
    }

    pub fn is_snapshot(&self) -> bool {
        self.subvolume_type == SubvolumeType::Snapshot
    }
}

/// Snapshot Btrfs
#[derive(Debug, Clone)]
pub struct BtrfsSnapshot {
    pub snapshot_id: u64,
    pub subvolume_id: u64,
    pub name: String,
    pub creation_time: u64,
    pub readonly: bool,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl BtrfsSnapshot {
    pub fn new(snapshot_id: u64, subvolume_id: u64, name: &str, readonly: bool) -> Self {
        Self {
            snapshot_id,
            subvolume_id,
            name: String::from(name),
            creation_time: 0,
            readonly,
            graph_node_id: None,
        }
    }
}

/// Entrada de archivo Btrfs
#[derive(Debug, Clone)]
pub struct BtrfsFileEntry {
    pub name: String,
    pub size: u64,
    pub inode: u64,
    pub subvolume_id: u64,
    pub compression: bool,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl BtrfsFileEntry {
    pub fn new(name: &str, size: u64, inode: u64, subvolume_id: u64) -> Self {
        Self {
            name: String::from(name),
            size,
            inode,
            subvolume_id,
            compression: false,
            graph_node_id: None,
        }
    }
}

/// Driver de Btrfs
#[derive(Debug, Clone)]
pub struct CronosBtrfsDriver {
    pub mounted: bool,
    pub volume_label: String,
    pub uuid: [u8; 16],
    pub chunk_size: u32,
    pub total_chunks: u64,
    pub free_chunks: u64,
    pub subvolumes: BTreeMap<u64, BtrfsSubvolume>,
    pub snapshots: BTreeMap<u64, BtrfsSnapshot>,
    pub root_directory: Vec<BtrfsFileEntry>,
    pub compression_enabled: bool,
    pub next_subvolume_id: u64,
    pub next_snapshot_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosBtrfsDriver {
    pub fn new() -> Self {
        Self {
            mounted: false,
            volume_label: String::new(),
            uuid: [0; 16],
            chunk_size: 256 * 1024, // 256KB chunks
            total_chunks: 0,
            free_chunks: 0,
            subvolumes: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            root_directory: Vec::new(),
            compression_enabled: false,
            next_subvolume_id: 6,
            next_snapshot_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    pub fn mount(&mut self, volume_label: &str, uuid: [u8; 16]) -> Result<(), String> {
        self.volume_label = String::from(volume_label);
        self.uuid = uuid;
        self.mounted = true;
        
        // Crear subvolumen raíz
        let mut root_subvolume = BtrfsSubvolume::new(
            5, // ID 5 es el subvolumen raíz en Btrfs
            0,
            "root",
            SubvolumeType::Root,
        );

        // Registrar el subvolumen raíz como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("btrfs_subvolume_root");
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            root_subvolume.graph_node_id = node_id;
        }

        self.subvolumes.insert(5, root_subvolume);
        
        // Simular lectura del sistema de archivos
        let mut default_entry = BtrfsFileEntry::new(
            "default",
            4096,
            256,
            5,
        );

        // Registrar la entrada como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("btrfs_file_default");
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            default_entry.graph_node_id = node_id;
        }

        self.root_directory.push(default_entry);
        
        Ok(())
    }

    pub fn unmount(&mut self) -> Result<(), String> {
        self.mounted = false;
        self.subvolumes.clear();
        self.snapshots.clear();
        self.root_directory.clear();
        Ok(())
    }

    pub fn is_mounted(&self) -> bool {
        self.mounted
    }

    pub fn volume_label(&self) -> &str {
        &self.volume_label
    }

    pub fn uuid(&self) -> [u8; 16] {
        self.uuid
    }

    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    pub fn total_chunks(&self) -> u64 {
        self.total_chunks
    }

    pub fn free_chunks(&self) -> u64 {
        self.free_chunks
    }

    pub fn total_space(&self) -> u64 {
        self.total_chunks * self.chunk_size as u64
    }

    pub fn free_space(&self) -> u64 {
        self.free_chunks * self.chunk_size as u64
    }

    pub fn used_space(&self) -> u64 {
        self.total_space() - self.free_space()
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.compression_enabled
    }

    pub fn set_compression(&mut self, enabled: bool) {
        self.compression_enabled = enabled;
    }

    pub fn create_subvolume(&mut self, name: &str, parent_id: u64) -> Result<u64, String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        let new_id = self.next_subvolume_id;
        self.next_subvolume_id += 1;
        
        let mut subvolume = BtrfsSubvolume::new(
            new_id,
            parent_id,
            name,
            SubvolumeType::Normal,
        );

        // Registrar el subvolumen como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("btrfs_subvolume_{}", new_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            subvolume.graph_node_id = node_id;
        }
        
        self.subvolumes.insert(new_id, subvolume);
        Ok(new_id)
    }

    pub fn delete_subvolume(&mut self, subvolume_id: u64) -> Result<(), String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        if let Some(subvolume) = self.get_subvolume(subvolume_id) {
            if subvolume.is_root() {
                return Err(String::from("Cannot delete root subvolume"));
            }
        }

        if self.subvolumes.remove(&subvolume_id).is_some() {
            Ok(())
        } else {
            Err(format!("Subvolume {} not found", subvolume_id))
        }
    }

    pub fn get_subvolume(&self, subvolume_id: u64) -> Option<&BtrfsSubvolume> {
        self.subvolumes.get(&subvolume_id)
    }

    pub fn list_subvolumes(&self) -> Vec<&BtrfsSubvolume> {
        self.subvolumes.values().collect()
    }

    pub fn create_snapshot(&mut self, subvolume_id: u64, name: &str, readonly: bool) -> Result<u64, String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        if self.get_subvolume(subvolume_id).is_none() {
            return Err(format!("Subvolume {} not found", subvolume_id));
        }

        let snapshot_id = self.next_snapshot_id;
        self.next_snapshot_id += 1;

        let mut snapshot = BtrfsSnapshot::new(snapshot_id, subvolume_id, name, readonly);

        // Registrar el snapshot como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("btrfs_snapshot_{}", snapshot_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            snapshot.graph_node_id = node_id;
        }

        self.snapshots.insert(snapshot_id, snapshot);
        
        Ok(snapshot_id)
    }

    pub fn delete_snapshot(&mut self, snapshot_id: u64) -> Result<(), String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        if self.snapshots.remove(&snapshot_id).is_some() {
            Ok(())
        } else {
            Err(format!("Snapshot {} not found", snapshot_id))
        }
    }

    pub fn list_snapshots(&self) -> Vec<&BtrfsSnapshot> {
        self.snapshots.values().collect()
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<BtrfsFileEntry>, String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        if path == "/" || path == "\\" {
            Ok(self.root_directory.clone())
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_file_entry(&self, path: &str) -> Option<BtrfsFileEntry> {
        if !self.mounted {
            return None;
        }

        if path == "/" || path == "\\" {
            return None;
        }

        let filename = path.trim_start_matches('/').trim_start_matches('\\');
        
        for entry in &self.root_directory {
            if entry.name == filename {
                return Some(entry.clone());
            }
        }

        None
    }

    pub fn create_file(&mut self, path: &str, size: u64) -> Result<(), String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        let filename = path.trim_start_matches('/').trim_start_matches('\\');
        
        let mut entry = BtrfsFileEntry::new(filename, size, 0, 5);

        // Registrar la entrada como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("btrfs_file_{}", filename);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            entry.graph_node_id = node_id;
        }

        self.root_directory.push(entry);
        
        Ok(())
    }

    pub fn delete_file(&mut self, path: &str) -> Result<(), String> {
        if !self.mounted {
            return Err(String::from("Btrfs not mounted"));
        }

        let filename = path.trim_start_matches('/').trim_start_matches('\\');
        
        if let Some(pos) = self.root_directory.iter().position(|e| e.name == filename) {
            self.root_directory.remove(pos);
            Ok(())
        } else {
            Err(format!("File {} not found", filename))
        }
    }
}

impl Default for CronosBtrfsDriver {
    fn default() -> Self {
        Self::new()
    }
}
