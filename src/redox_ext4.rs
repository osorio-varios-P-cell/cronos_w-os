//! Filesystem ext4 de Redox adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el filesystem ext4 de Redox OS adaptado al VFS
//! y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;
use crate::filesystem::{VirtualFileSystem, VfsInode, FileType, FileMetadata, FilePermissions};

/// Inodo ext4 (adaptado de Redox)
#[derive(Debug, Clone)]
pub struct Ext4Inode {
    pub inode_number: u32,
    pub mode: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub links_count: u16,
    pub blocks: Vec<u64>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl Ext4Inode {
    pub fn new(inode_number: u32) -> Self {
        Self {
            inode_number,
            mode: 0,
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            links_count: 0,
            blocks: Vec::new(),
            graph_node_id: None,
        }
    }

    /// Convertir a VfsInode
    pub fn to_vfs_inode(&self) -> VfsInode {
        let file_type = if self.mode & 0xF000 == 0x4000 {
            FileType::Directory
        } else if self.mode & 0xF000 == 0x8000 {
            FileType::Regular
        } else if self.mode & 0xF000 == 0xA000 {
            FileType::Symlink
        } else {
            FileType::Regular
        };

        let mut metadata = FileMetadata {
            file_type,
            size: self.size,
            permissions: FilePermissions::from_octal((self.mode & 0o777) as u32),
            owner_id: self.uid as u64,
            group_id: self.gid as u64,
            created_at: self.ctime,
            modified_at: self.mtime,
            accessed_at: self.atime,
        };

        VfsInode {
            id: self.inode_number as u64,
            name: String::new(),
            metadata,
            data: Vec::new(),
            blocks: self.blocks.clone(),
            parent_id: None,
            graph_node_id: self.graph_node_id,
        }
    }
}

/// Superbloque ext4 (adaptado de Redox)
#[derive(Debug, Clone)]
pub struct Ext4Superblock {
    pub inodes_count: u32,
    pub blocks_count: u64,
    pub free_inodes_count: u32,
    pub free_blocks_count: u64,
    pub block_size: u32,
    pub inode_size: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
}

impl Ext4Superblock {
    pub fn new() -> Self {
        Self {
            inodes_count: 0,
            blocks_count: 0,
            free_inodes_count: 0,
            free_blocks_count: 0,
            block_size: 4096,
            inode_size: 256,
            first_data_block: 0,
            log_block_size: 2,
        }
    }
}

impl Default for Ext4Superblock {
    fn default() -> Self {
        Self::new()
    }
}

/// Grupo de bloques ext4 (adaptado de Redox)
#[derive(Debug, Clone)]
pub struct Ext4BlockGroup {
    pub block_bitmap: Vec<u8>,
    pub inode_bitmap: Vec<u8>,
    pub inode_table: Vec<Ext4Inode>,
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,
}

impl Ext4BlockGroup {
    pub fn new() -> Self {
        Self {
            block_bitmap: Vec::new(),
            inode_bitmap: Vec::new(),
            inode_table: Vec::new(),
            free_blocks_count: 0,
            free_inodes_count: 0,
        }
    }
}

impl Default for Ext4BlockGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Directorio ext4 (adaptado de Redox)
#[derive(Debug, Clone)]
pub struct Ext4DirectoryEntry {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    pub name: String,
}

impl Ext4DirectoryEntry {
    pub fn new(inode: u32, name: String, file_type: u8) -> Self {
        Self {
            inode,
            rec_len: 0,
            name_len: name.len() as u8,
            file_type,
            name,
        }
    }
}

/// Filesystem ext4 de Redox adaptado
pub struct RedoxExt4Filesystem {
    pub superblock: Ext4Superblock,
    pub block_groups: Vec<Ext4BlockGroup>,
    pub inodes: BTreeMap<u32, Ext4Inode>,
    pub vfs: VirtualFileSystem,
    pub mount_point: String,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl RedoxExt4Filesystem {
    pub fn new(mount_point: String) -> Self {
        Self {
            superblock: Ext4Superblock::new(),
            block_groups: Vec::new(),
            inodes: BTreeMap::new(),
            vfs: VirtualFileSystem::new(),
            mount_point,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel.clone()));
        self.vfs.set_graph_kernel(graph_kernel);
    }

    /// Montar el filesystem ext4
    pub fn mount(&mut self, device_id: u64) -> Result<(), String> {
        // En un sistema real, aquí se:
        // 1. Leería el superbloque desde el dispositivo
        // 2. Analizaría los grupos de bloques
        // 3. Cargaría la tabla de inodos
        // 4. Construiría el árbol de directorios

        // Simulación: crear superbloque
        self.superblock = Ext4Superblock {
            inodes_count: 65536,
            blocks_count: 1048576,
            free_inodes_count: 65535,
            free_blocks_count: 1048575,
            block_size: 4096,
            inode_size: 256,
            first_data_block: 0,
            log_block_size: 2,
        };

        // Inicializar VFS
        self.vfs.initialize()?;

        // Crear inodo raíz
        let mut root_inode = Ext4Inode::new(2);
        root_inode.mode = 0o40755; // Directorio root
        root_inode.links_count = 2;
        
        // Registrar el inodo raíz como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("ext4_root_{}", self.mount_point);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            root_inode.graph_node_id = node_id;
        }

        self.inodes.insert(2, root_inode);

        Ok(())
    }

    /// Leer un inodo ext4
    pub fn read_inode(&self, inode_number: u32) -> Option<&Ext4Inode> {
        self.inodes.get(&inode_number)
    }

    /// Leer un archivo ext4
    pub fn read_file(&mut self, inode_number: u32, offset: u64, buffer: &mut [u8]) -> Result<usize, String> {
        if let Some(ext4_inode) = self.read_inode(inode_number) {
            let vfs_inode = ext4_inode.to_vfs_inode();
            self.vfs.read_file(vfs_inode.id, offset, buffer)
        } else {
            Err(format!("Inode {} not found", inode_number))
        }
    }

    /// Escribir a un archivo ext4
    pub fn write_file(&mut self, inode_number: u32, offset: u64, buffer: &[u8]) -> Result<usize, String> {
        if let Some(ext4_inode) = self.read_inode(inode_number) {
            let vfs_inode = ext4_inode.to_vfs_inode();
            self.vfs.write_file(vfs_inode.id, offset, buffer)
        } else {
            Err(format!("Inode {} not found", inode_number))
        }
    }

    /// Listar directorio ext4
    pub fn list_directory(&mut self, inode_number: u32) -> Result<Vec<String>, String> {
        if let Some(ext4_inode) = self.read_inode(inode_number) {
            let vfs_inode = ext4_inode.to_vfs_inode();
            self.vfs.list_directory(vfs_inode.id)
        } else {
            Err(format!("Inode {} not found", inode_number))
        }
    }

    /// Crear un archivo ext4
    pub fn create_file(&mut self, parent_inode: u32, name: String, file_type: FileType) -> Result<u32, String> {
        // En un sistema real, aquí se:
        // 1. Asignaría un nuevo número de inodo
        // 2. Inicializaría el inodo ext4
        // 3. Agregaría la entrada al directorio padre
        // 4. Actualizaría los bitmaps de bloques e inodos

        let new_inode_number = self.superblock.free_inodes_count;
        let mut new_inode = Ext4Inode::new(new_inode_number);
        
        let mode = match file_type {
            FileType::Regular => 0o100644,
            FileType::Directory => 0o40755,
            FileType::Symlink => 0o120777,
            _ => 0o100644,
        };
        
        new_inode.mode = mode;
        new_inode.links_count = 1;

        // Registrar el nuevo inodo como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("ext4_inode_{}", new_inode_number);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            new_inode.graph_node_id = node_id;
        }

        self.inodes.insert(new_inode_number, new_inode);
        self.superblock.free_inodes_count -= 1;

        Ok(new_inode_number)
    }

    /// Eliminar un archivo ext4
    pub fn delete_file(&mut self, inode_number: u32) -> Result<(), String> {
        if self.inodes.remove(&inode_number).is_some() {
            self.superblock.free_inodes_count += 1;
            Ok(())
        } else {
            Err(format!("Inode {} not found", inode_number))
        }
    }

    /// Obtener el VFS
    pub fn vfs(&mut self) -> &mut VirtualFileSystem {
        &mut self.vfs
    }

    /// Obtener estadísticas del filesystem
    pub fn stats(&self) -> (u32, u32, u32, u64) {
        (
            self.superblock.inodes_count,
            self.superblock.free_inodes_count,
            self.superblock.blocks_count as u32,
            self.superblock.free_blocks_count,
        )
    }
}

impl Default for RedoxExt4Filesystem {
    fn default() -> Self {
        Self::new(String::from("/"))
    }
}

/// Errores del filesystem ext4
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ext4Error {
    InvalidSuperblock,
    InodeNotFound,
    BlockNotFound,
    FileSystemCorrupted,
    ReadOnly,
    NoSpace,
}

impl fmt::Display for Ext4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ext4Error::InvalidSuperblock => write!(f, "Invalid superblock"),
            Ext4Error::InodeNotFound => write!(f, "Inode not found"),
            Ext4Error::BlockNotFound => write!(f, "Block not found"),
            Ext4Error::FileSystemCorrupted => write!(f, "Filesystem corrupted"),
            Ext4Error::ReadOnly => write!(f, "Read-only filesystem"),
            Ext4Error::NoSpace => write!(f, "No space left"),
        }
    }
}
