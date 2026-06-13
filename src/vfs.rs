//! VFS (Virtual File System) Layer Module
//! 
//! This module implements a unified VFS layer for filesystem abstraction.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de archivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Archivo regular
    Regular,
    /// Directorio
    Directory,
    /// Dispositivo de caracteres
    CharDevice,
    /// Dispositivo de bloques
    BlockDevice,
    /// FIFO (named pipe)
    Fifo,
    /// Socket
    Socket,
    /// Enlace simbólico
    Symlink,
    /// FASE 16: Partición Externa (NTFS/APFS)
    ExternalPartition,
}

/// Permisos de archivo
#[derive(Debug, Clone, Copy)]
pub struct FilePermissions {
    /// Permisos del propietario
    pub owner: u8,
    /// Permisos del grupo
    pub group: u8,
    /// Permisos de otros
    pub other: u8,
}

impl FilePermissions {
    /// Crear nuevos permisos
    pub fn new(owner: u8, group: u8, other: u8) -> Self {
        Self { owner, group, other }
    }

    /// Permisos por defecto (755)
    pub fn default() -> Self {
        Self::new(0o7, 0o5, 0o5)
    }

    /// Verificar si el propietario puede leer
    pub fn owner_can_read(&self) -> bool {
        self.owner & 0o4 != 0
    }

    /// Verificar si el propietario puede escribir
    pub fn owner_can_write(&self) -> bool {
        self.owner & 0o2 != 0
    }

    /// Verificar si el propietario puede ejecutar
    pub fn owner_can_execute(&self) -> bool {
        self.owner & 0o1 != 0
    }
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self::default()
    }
}

/// Inodo VFS
#[derive(Debug, Clone)]
pub struct VfsInode {
    /// Número de inodo
    pub inode_number: u64,
    /// Tipo de archivo
    pub file_type: FileType,
    /// Permisos
    pub permissions: FilePermissions,
    /// UID del propietario
    pub uid: u32,
    /// GID del grupo
    pub gid: u32,
    /// Tamaño del archivo
    pub size: u64,
    /// Número de bloques
    pub blocks: u64,
    /// Tiempo de acceso
    pub atime: u64,
    /// Tiempo de modificación
    pub mtime: u64,
    /// Tiempo de cambio
    pub ctime: u64,
    /// Número de enlaces
    pub nlink: u32,
}

impl VfsInode {
    /// Crear nuevo inodo
    pub fn new(inode_number: u64, file_type: FileType) -> Self {
        Self {
            inode_number,
            file_type,
            permissions: FilePermissions::default(),
            uid: 0,
            gid: 0,
            size: 0,
            blocks: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 1,
        }
    }

    /// Es directorio
    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }

    /// Es archivo regular
    pub fn is_regular_file(&self) -> bool {
        self.file_type == FileType::Regular
    }
}

/// Entrada de directorio
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Número de inodo
    pub inode: u64,
    /// Tipo de archivo
    pub file_type: FileType,
    /// Nombre del archivo
    pub name: String,
}

impl DirEntry {
    /// Crear nueva entrada
    pub fn new(inode: u64, file_type: FileType, name: String) -> Self {
        Self { inode, file_type, name }
    }
}

/// Superbloque VFS
#[derive(Debug, Clone)]
pub struct VfsSuperblock {
    /// Identificador del filesystem
    pub fs_id: u32,
    /// Nombre del filesystem
    pub fs_name: String,
    /// Tamaño total
    pub total_size: u64,
    /// Espacio libre
    pub free_space: u64,
    /// Número total de inodos
    pub total_inodes: u64,
    /// Inodos libres
    pub free_inodes: u64,
}

impl VfsSuperblock {
    /// Crear nuevo superbloque
    pub fn new(fs_id: u32, fs_name: String, total_size: u64) -> Self {
        Self {
            fs_id,
            fs_name,
            total_size,
            free_space: total_size,
            total_inodes: 65536,
            free_inodes: 65536,
        }
    }
}

/// Operaciones VFS
pub trait VfsOperations {
    /// Leer archivo
    fn read(&self, inode: u64, offset: u64, buffer: &mut [u8]) -> Result<usize, String>;
    
    /// Escribir archivo
    fn write(&mut self, inode: u64, offset: u64, data: &[u8]) -> Result<usize, String>;
    
    /// Crear archivo
    fn create(&mut self, parent: u64, name: &str, file_type: FileType) -> Result<u64, String>;
    
    /// Eliminar archivo
    fn unlink(&mut self, parent: u64, name: &str) -> Result<(), String>;
    
    /// Crear directorio
    fn mkdir(&mut self, parent: u64, name: &str) -> Result<u64, String>;
    
    /// Eliminar directorio
    fn rmdir(&mut self, parent: u64, name: &str) -> Result<(), String>;
    
    /// Leer directorio
    fn readdir(&self, inode: u64) -> Result<Vec<DirEntry>, String>;
    
    /// Obtener inodo
    fn lookup(&self, parent: u64, name: &str) -> Result<VfsInode, String>;
    
    /// Obtener atributos
    fn getattr(&self, inode: u64) -> Result<VfsInode, String>;
    
    /// Establecer atributos
    fn setattr(&mut self, inode: u64, attrs: VfsInode) -> Result<(), String>;
}

/// Montaje VFS
#[derive(Debug, Clone)]
pub struct VfsMount {
    /// Punto de montaje
    pub mount_point: String,
    /// Superbloque
    pub superblock: VfsSuperblock,
    /// Raíz del filesystem
    pub root_inode: u64,
}

impl VfsMount {
    /// Crear nuevo montaje
    pub fn new(mount_point: String, superblock: VfsSuperblock, root_inode: u64) -> Self {
        Self {
            mount_point,
            superblock,
            root_inode,
        }
    }
}

/// Sistema VFS
pub struct VfsSystem {
    /// Montajes activos
    pub mounts: Vec<VfsMount>,
    /// Siguiente ID de filesystem
    pub next_fs_id: u32,
}

impl VfsSystem {
    /// Crear nuevo sistema VFS
    pub fn new() -> Self {
        Self {
            mounts: Vec::new(),
            next_fs_id: 1,
        }
    }

    /// FASE 16: Auto-detección y montaje de particiones de otros OS
    pub fn auto_mount_external(&mut self) -> Result<u32, String> {
        let mut count = 0;
        // 1. Escanear tablas de particiones (GPT/MBR)
        // 2. Identificar GUIDs de NTFS (Windows) y APFS (Apple)
        // 3. Crear montajes en /mnt/windows y /mnt/macos

        // Simulación de montaje de disco Windows real
        let sb_win = VfsSuperblock::new(10, String::from("Windows-SSD"), 500 * 1024 * 1024 * 1024);
        self.mount(String::from("/mnt/windows"), sb_win, 1).map(|_| { count += 1 });

        Ok(count)
    }

    /// Montar filesystem
    pub fn mount(&mut self, mount_point: String, superblock: VfsSuperblock, root_inode: u64) -> Result<(), String> {
        // Verificar si el punto de montaje ya existe
        for mount in &self.mounts {
            if mount.mount_point == mount_point {
                return Err(format!("Mount point {} already exists", mount_point));
            }
        }

        let mount = VfsMount::new(mount_point, superblock, root_inode);
        self.mounts.push(mount);
        Ok(())
    }

    /// Desmontar filesystem
    pub fn unmount(&mut self, mount_point: &str) -> Result<(), String> {
        let index = self.mounts.iter()
            .position(|m| m.mount_point == mount_point)
            .ok_or_else(|| String::from("Mount point not found"))?;
        
        self.mounts.remove(index);
        Ok(())
    }

    /// Obtener montaje por ruta
    pub fn get_mount(&self, path: &str) -> Option<&VfsMount> {
        // Encontrar el montaje más largo que coincida con la ruta
        let mut best_match = None;
        let mut best_len = 0;

        for mount in &self.mounts {
            let mount_len = mount.mount_point.len();
            if path.starts_with(&mount.mount_point) && mount_len > best_len {
                best_match = Some(mount);
                best_len = mount_len;
            }
        }

        best_match
    }

    /// Resolver ruta a inodo
    pub fn resolve_path(&self, path: &str) -> Result<(u64, &VfsMount), String> {
        let mount = self.get_mount(path)
            .ok_or_else(|| String::from("No mount found for path"))?;
        
        // En un sistema real, esto resolvería la ruta relativa al punto de montaje
        // Para este ejemplo, retornamos el inodo raíz
        Ok((mount.root_inode, mount))
    }

    /// Obtener número de montajes
    pub fn mount_count(&self) -> usize {
        self.mounts.len()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("VFS System Status\n");
        report.push_str("================\n\n");
        
        report.push_str(&format!("Mounts: {}\n\n", self.mount_count()));
        
        for mount in &self.mounts {
            report.push_str(&format!("Mount Point: {}\n", mount.mount_point));
            report.push_str(&format!("  FS ID: {}\n", mount.superblock.fs_id));
            report.push_str(&format!("  FS Name: {}\n", mount.superblock.fs_name));
            report.push_str(&format!("  Total Size: {} bytes\n", mount.superblock.total_size));
            report.push_str(&format!("  Free Space: {} bytes\n", mount.superblock.free_space));
            report.push_str(&format!("  Total Inodes: {}\n", mount.superblock.total_inodes));
            report.push_str(&format!("  Free Inodes: {}\n", mount.superblock.free_inodes));
            report.push_str(&format!("  Root Inode: {}\n\n", mount.root_inode));
        }
        
        report
    }
}

impl Default for VfsSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades VFS
pub struct VfsUtils;

impl VfsUtils {
    /// Normalizar ruta
    pub fn normalize_path(path: &str) -> String {
        let mut result = String::new();
        let mut components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        for component in components {
            if component == "." {
                continue;
            } else if component == ".." {
                // En un sistema real, esto manejaría ".."
                continue;
            } else {
                result.push('/');
                result.push_str(component);
            }
        }
        
        if result.is_empty() {
            String::from("/")
        } else {
            result
        }
    }

    /// Obtener directorio padre
    pub fn parent_directory(path: &str) -> String {
        let normalized = Self::normalize_path(path);
        if normalized == "/" {
            return String::from("/");
        }
        
        let last_slash = normalized.rfind('/');
        match last_slash {
            Some(0) => String::from("/"),
            Some(pos) => String::from(&normalized[..pos]),
            None => String::from("/"),
        }
    }

    /// Obtener nombre base
    pub fn basename(path: &str) -> String {
        let normalized = Self::normalize_path(path);
        if normalized == "/" {
            return String::from("/");
        }
        
        let last_slash = normalized.rfind('/');
        match last_slash {
            Some(pos) => String::from(&normalized[pos + 1..]),
            None => normalized,
        }
    }
}
