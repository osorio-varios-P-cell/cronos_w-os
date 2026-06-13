//! Sistema de Archivos (VFS + FAT32) para CRONOS W-OS
//!
//! Este módulo implementa el Virtual File System (VFS) con soporte para FAT32,
//! adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Tipo de sistema de archivos
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemType {
    /// CRONOSFS - Sistema de archivos nativo
    CronosFs,
    /// FAT32
    Fat32,
    /// ext4
    Ext4,
    /// NTFS
    Ntfs,
    /// TMPFS
    TmpFs,
    /// PROCFS
    ProcFs,
    /// SYSFS
    SysFs,
    /// Desconocido
    Unknown,
}

/// Tipo de nodo de archivo
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    /// Archivo regular
    Regular,
    /// Directorio
    Directory,
    /// Dispositivo de bloque
    BlockDevice,
    /// Dispositivo de carácter
    CharDevice,
    /// Enlace simbólico
    Symlink,
    /// Socket
    Socket,
    /// Pipe
    Pipe,
}

/// Permisos de archivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions {
    /// Permisos del propietario
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_execute: bool,
    /// Permisos del grupo
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    /// Permisos de otros
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
}

impl FilePermissions {
    pub fn new() -> Self {
        Self {
            owner_read: true,
            owner_write: true,
            owner_execute: false,
            group_read: true,
            group_write: false,
            group_execute: false,
            other_read: true,
            other_write: false,
            other_execute: false,
        }
    }

    pub fn from_octal(octal: u32) -> Self {
        Self {
            owner_read: (octal & 0o400) != 0,
            owner_write: (octal & 0o200) != 0,
            owner_execute: (octal & 0o100) != 0,
            group_read: (octal & 0o040) != 0,
            group_write: (octal & 0o020) != 0,
            group_execute: (octal & 0o010) != 0,
            other_read: (octal & 0o004) != 0,
            other_write: (octal & 0o002) != 0,
            other_execute: (octal & 0o001) != 0,
        }
    }

    pub fn to_octal(&self) -> u32 {
        let mut octal = 0u32;
        if self.owner_read { octal |= 0o400; }
        if self.owner_write { octal |= 0o200; }
        if self.owner_execute { octal |= 0o100; }
        if self.group_read { octal |= 0o040; }
        if self.group_write { octal |= 0o020; }
        if self.group_execute { octal |= 0o010; }
        if self.other_read { octal |= 0o004; }
        if self.other_write { octal |= 0o002; }
        if self.other_execute { octal |= 0o001; }
        octal
    }
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadatos de archivo
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Tipo de archivo
    pub file_type: FileType,
    /// Tamaño en bytes
    pub size: u64,
    /// Permisos
    pub permissions: FilePermissions,
    /// ID del propietario
    pub owner_id: u64,
    /// ID del grupo
    pub group_id: u64,
    /// Tiempo de creación (timestamp)
    pub created_at: u64,
    /// Tiempo de modificación (timestamp)
    pub modified_at: u64,
    /// Tiempo de acceso (timestamp)
    pub accessed_at: u64,
}

impl Default for FileMetadata {
    fn default() -> Self {
        Self {
            file_type: FileType::Regular,
            size: 0,
            permissions: FilePermissions::default(),
            owner_id: 0,
            group_id: 0,
            created_at: 0,
            modified_at: 0,
            accessed_at: 0,
        }
    }
}

/// Inodo VFS
#[derive(Debug, Clone)]
pub struct VfsInode {
    /// ID del inodo
    pub id: u64,
    /// Nombre del archivo
    pub name: String,
    /// Metadatos
    pub metadata: FileMetadata,
    /// Datos del archivo (para archivos pequeños)
    pub data: Vec<u8>,
    /// Bloques del archivo (para archivos grandes)
    pub blocks: Vec<u64>,
    /// ID del inodo padre (directorio padre)
    pub parent_id: Option<u64>,
    /// ID del nodo en el grafo
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl VfsInode {
    pub fn new(id: u64, name: String, file_type: FileType) -> Self {
        let mut metadata = FileMetadata::default();
        metadata.file_type = file_type;
        
        Self {
            id,
            name,
            metadata,
            data: Vec::new(),
            blocks: Vec::new(),
            parent_id: None,
            graph_node_id: None,
        }
    }

    /// Verificar si es un directorio
    pub fn is_directory(&self) -> bool {
        self.metadata.file_type == FileType::Directory
    }

    /// Verificar si es un archivo regular
    pub fn is_regular(&self) -> bool {
        self.metadata.file_type == FileType::Regular
    }
}

/// Sistema de archivos virtual (VFS)
pub struct VirtualFileSystem {
    /// Inodos en el sistema
    pub inodes: BTreeMap<u64, VfsInode>,
    /// Próximo ID de inodo
    pub next_inode_id: u64,
    /// Inodo raíz
    pub root_inode_id: Option<u64>,
    /// Referencia al graph kernel
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            inodes: BTreeMap::new(),
            next_inode_id: 1,
            root_inode_id: None,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Inicializar el VFS
    pub fn initialize(&mut self) -> Result<(), String> {
        // Crear el directorio raíz
        let root_id = 0;
        let root_inode = VfsInode::new(root_id, String::from("/"), FileType::Directory);
        self.root_inode_id = Some(root_id);
        self.inodes.insert(root_id, root_inode);
        self.next_inode_id = 1;

        // Crear directorios básicos
        self.create_file(root_id, String::from("bin"), FileType::Directory)?;
        self.create_file(root_id, String::from("etc"), FileType::Directory)?;
        self.create_file(root_id, String::from("home"), FileType::Directory)?;
        self.create_file(root_id, String::from("usr"), FileType::Directory)?;
        self.create_file(root_id, String::from("var"), FileType::Directory)?;
        self.create_file(root_id, String::from("tmp"), FileType::Directory)?;
        self.create_file(root_id, String::from("dev"), FileType::Directory)?;
        self.create_file(root_id, String::from("proc"), FileType::Directory)?;
        self.create_file(root_id, String::from("sys"), FileType::Directory)?;

        Ok(())
    }

    /// Crear un nuevo archivo
    pub fn create_file(&mut self, parent_id: u64, name: String, file_type: FileType) -> Result<u64, String> {
        let inode_id = self.next_inode_id;
        self.next_inode_id += 1;

        let mut inode = VfsInode::new(inode_id, name, file_type);
        inode.parent_id = Some(parent_id);

        // Registrar el inodo como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("inode_{}", inode_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            inode.graph_node_id = node_id;
        }

        self.inodes.insert(inode_id, inode);
        Ok(inode_id)
    }

    /// Obtener un inodo por ID
    pub fn get_inode(&self, inode_id: u64) -> Option<&VfsInode> {
        self.inodes.get(&inode_id)
    }

    /// Obtener un inodo mutable por ID
    pub fn get_inode_mut(&mut self, inode_id: u64) -> Option<&mut VfsInode> {
        self.inodes.get_mut(&inode_id)
    }

    /// Leer un archivo
    pub fn read_file(&mut self, inode_id: u64, offset: u64, buffer: &mut [u8]) -> Result<usize, String> {
        if let Some(inode) = self.get_inode(inode_id) {
            if !inode.is_regular() {
                return Err(String::from("Not a regular file"));
            }

            let data_len = inode.data.len() as u64;
            if offset >= data_len {
                return Ok(0);
            }

            let start = offset as usize;
            let end = core::cmp::min(start + buffer.len(), data_len as usize);
            let bytes_to_copy = end - start;

            if bytes_to_copy > 0 {
                buffer[..bytes_to_copy].copy_from_slice(&inode.data[start..end]);
            }

            // Actualizar tiempo de acceso
            if let Some(inode) = self.get_inode_mut(inode_id) {
                inode.metadata.accessed_at = 0; // TODO: usar timestamp real
            }

            Ok(bytes_to_copy)
        } else {
            Err(format!("Inode {} not found", inode_id))
        }
    }

    /// Escribir a un archivo
    pub fn write_file(&mut self, inode_id: u64, offset: u64, buffer: &[u8]) -> Result<usize, String> {
        if let Some(inode) = self.get_inode(inode_id) {
            if !inode.is_regular() {
                return Err(String::from("Not a regular file"));
            }

            let inode = self.get_inode_mut(inode_id).unwrap();
            let offset_usize = offset as usize;

            // Asegurar que el vector tenga suficiente espacio
            if offset_usize + buffer.len() > inode.data.len() {
                inode.data.resize(offset_usize + buffer.len(), 0);
            }

            // Copiar los datos
            inode.data[offset_usize..offset_usize + buffer.len()].copy_from_slice(buffer);

            // Actualizar metadatos
            inode.metadata.size = inode.data.len() as u64;
            inode.metadata.modified_at = 0; // TODO: usar timestamp real

            Ok(buffer.len())
        } else {
            Err(format!("Inode {} not found", inode_id))
        }
    }

    /// Listar directorio
    pub fn list_directory(&self, inode_id: u64) -> Result<Vec<String>, String> {
        if let Some(inode) = self.get_inode(inode_id) {
            if !inode.is_directory() {
                return Err(String::from("Not a directory"));
            }

            let entries: Vec<String> = self.inodes.values()
                .filter(|i| i.parent_id == Some(inode_id))
                .map(|i| i.name.clone())
                .collect();

            Ok(entries)
        } else {
            Err(format!("Inode {} not found", inode_id))
        }
    }

    /// Eliminar un archivo
    pub fn delete_file(&mut self, inode_id: u64) -> Result<(), String> {
        if self.inodes.remove(&inode_id).is_some() {
            Ok(())
        } else {
            Err(format!("Inode {} not found", inode_id))
        }
    }

    /// Obtener el inodo raíz
    pub fn root_inode(&self) -> Option<&VfsInode> {
        self.root_inode_id.and_then(|id| self.get_inode(id))
    }

    /// Obtener número de inodos
    pub fn inode_count(&self) -> usize {
        self.inodes.len()
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Sistema de archivos FAT32
pub struct Fat32FileSystem {
    /// VFS subyacente
    pub vfs: VirtualFileSystem,
    /// Sector de inicio del FAT
    pub fat_start_sector: u64,
    /// Número de sectores del FAT
    pub fat_sector_count: u32,
    /// Sectores por clúster
    pub sectors_per_cluster: u8,
}

impl Fat32FileSystem {
    pub fn new() -> Self {
        Self {
            vfs: VirtualFileSystem::new(),
            fat_start_sector: 0,
            fat_sector_count: 0,
            sectors_per_cluster: 8,
        }
    }

    /// Montar sistema de archivos FAT32
    pub fn mount(&mut self, device_id: u64) -> Result<(), String> {
        // En un sistema real, aquí se:
        // 1. Leería el MBR (Master Boot Record)
        // 2. Leería la tabla de particiones
        // 3. Encontraría la partición FAT32
        // 4. Leería el VBR (Volume Boot Record)
        // 5. Analizaría la estructura FAT32
        // 6. Construiría el árbol de directorios

        self.vfs.initialize()?;
        Ok(())
    }

    /// Obtener el VFS
    pub fn vfs(&mut self) -> &mut VirtualFileSystem {
        &mut self.vfs
    }
}

impl Default for Fat32FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestor de sistemas de archivos
pub struct FileSystemManager {
    /// Sistemas de archivos montados
    pub file_systems: BTreeMap<String, VirtualFileSystem>,
    /// Sistema de archivos FAT32
    pub fat32: Option<Fat32FileSystem>,
    /// Referencia al graph kernel
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl FileSystemManager {
    pub fn new() -> Self {
        Self {
            file_systems: BTreeMap::new(),
            fat32: None,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Montar un sistema de archivos
    pub fn mount(&mut self, mount_point: String, fs_type: FileSystemType, device_id: u64) -> Result<(), String> {
        match fs_type {
            FileSystemType::Fat32 | FileSystemType::CronosFs => {
                if self.fat32.is_none() {
                    let mut fat32 = Fat32FileSystem::new();
                    // No necesitamos pasar el graph kernel aquí, ya que se puede establecer después
                    fat32.mount(device_id)?;
                    self.fat32 = Some(fat32);
                }
                Ok(())
            }
            FileSystemType::Ext4 => {
                Err(String::from("ext4 not implemented yet"))
            }
            FileSystemType::Ntfs => {
                Err(String::from("NTFS not implemented yet"))
            }
            FileSystemType::TmpFs => {
                Err(String::from("TMPFS not implemented yet"))
            }
            FileSystemType::ProcFs => {
                Err(String::from("PROCFS not implemented yet"))
            }
            FileSystemType::SysFs => {
                Err(String::from("SYSFS not implemented yet"))
            }
            FileSystemType::Unknown => {
                Err(String::from("Unknown filesystem type"))
            }
        }
    }

    /// Crear un archivo
    pub fn create_file(&mut self, path: &str, file_type: FileType) -> Result<u64, String> {
        // Parsear la ruta y crear el archivo
        // Por ahora, usar el primer sistema de archivos disponible
        if let Some(ref mut fat32) = self.fat32 {
            let vfs = fat32.vfs();
            let root_id = vfs.root_inode_id.unwrap_or(0);
            vfs.create_file(root_id, String::from(path), file_type)
        } else {
            Err(String::from("No filesystem mounted"))
        }
    }

    /// Leer un archivo
    pub fn read_file(&mut self, path: &str, offset: u64, buffer: &mut [u8]) -> Result<usize, String> {
        if let Some(ref mut fat32) = self.fat32 {
            let vfs = fat32.vfs();
            // Por ahora, buscar por nombre en el directorio raíz
            let root_id = vfs.root_inode_id.unwrap_or(0);
            let entries = vfs.list_directory(root_id)?;
            
            for entry in entries {
                if entry == path {
                    // Encontrar el inodo correspondiente
                    if let Some(inode) = vfs.inodes.values().find(|i| i.name == entry) {
                        return vfs.read_file(inode.id, offset, buffer);
                    }
                }
            }
            Err(format!("File not found: {}", path))
        } else {
            Err(String::from("No filesystem mounted"))
        }
    }

    /// Escribir a un archivo
    pub fn write_file(&mut self, path: &str, offset: u64, buffer: &[u8]) -> Result<usize, String> {
        if let Some(ref mut fat32) = self.fat32 {
            let vfs = fat32.vfs();
            let root_id = vfs.root_inode_id.unwrap_or(0);
            let entries = vfs.list_directory(root_id)?;
            
            for entry in entries {
                if entry == path {
                    if let Some(inode) = vfs.inodes.values().find(|i| i.name == entry) {
                        return vfs.write_file(inode.id, offset, buffer);
                    }
                }
            }
            Err(format!("File not found: {}", path))
        } else {
            Err(String::from("No filesystem mounted"))
        }
    }

    /// Listar directorio
    pub fn list_directory(&mut self, path: &str) -> Result<Vec<String>, String> {
        if let Some(ref mut fat32) = self.fat32 {
            let vfs = fat32.vfs();
            let root_id = vfs.root_inode_id.unwrap_or(0);
            vfs.list_directory(root_id)
        } else {
            Err(String::from("No filesystem mounted"))
        }
    }

    /// Obtener número de sistemas de archivos
    pub fn fs_count(&self) -> usize {
        self.file_systems.len()
    }
}

impl Default for FileSystemManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del sistema de archivos
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemError {
    FileSystemNotFound,
    FileNotFound,
    DirectoryNotFound,
    PermissionDenied,
    DiskFull,
    InvalidPath,
    NotADirectory,
    NotAFile,
    AlreadyExists,
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::FileSystemNotFound => write!(f, "Filesystem not found"),
            FileSystemError::FileNotFound => write!(f, "File not found"),
            FileSystemError::DirectoryNotFound => write!(f, "Directory not found"),
            FileSystemError::PermissionDenied => write!(f, "Permission denied"),
            FileSystemError::DiskFull => write!(f, "Disk full"),
            FileSystemError::InvalidPath => write!(f, "Invalid path"),
            FileSystemError::NotADirectory => write!(f, "Not a directory"),
            FileSystemError::NotAFile => write!(f, "Not a file"),
            FileSystemError::AlreadyExists => write!(f, "Already exists"),
        }
    }
}
