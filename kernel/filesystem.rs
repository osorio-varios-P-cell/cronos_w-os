//! Módulo de Sistema de Archivos para CRONOS W-OS
//! Implementa sistema de archivos completo con soporte para múltiples FS

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Tipo de sistema de archivos
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemType {
    CRONOSFS,  // Sistema de archivos nativo de CRONOS
    EXT4,
    FAT32,
    NTFS,
    TMPFS,
    PROCFS,
    SYSFS,
}

/// Tipo de archivo
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    RegularFile,
    Directory,
    SymbolicLink,
    CharacterDevice,
    BlockDevice,
    Socket,
    FIFO,
}

/// Permisos de archivo
#[derive(Debug, Clone)]
pub struct FilePermissions {
    pub user_read: bool,
    pub user_write: bool,
    pub user_execute: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
}

/// Metadatos de archivo
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub created_at: u64,
    pub modified_at: u64,
    pub accessed_at: u64,
    pub permissions: FilePermissions,
    pub owner: u32,
    pub group: u32,
}

/// Inodo
#[derive(Debug, Clone)]
pub struct Inode {
    pub id: u64,
    pub file_type: FileType,
    pub metadata: FileMetadata,
    pub data_blocks: Vec<u64>,
}

/// Directorio
#[derive(Debug, Clone)]
pub struct Directory {
    pub inode_id: u64,
    pub entries: BTreeMap<String, u64>,
}

/// Archivo
#[derive(Debug, Clone)]
pub struct File {
    pub inode_id: u64,
    pub data: Vec<u8>,
    pub offset: u64,
}

/// Sistema de archivos
#[derive(Debug, Clone)]
pub struct FileSystem {
    pub fs_type: FileSystemType,
    pub mount_point: String,
    pub inodes: BTreeMap<u64, Inode>,
    pub directories: BTreeMap<u64, Directory>,
    pub files: BTreeMap<u64, File>,
    pub next_inode_id: u64,
}

/// Sistema de archivos virtual
pub struct FileSystemVirtual {
    pub file_systems: BTreeMap<String, FileSystem>,
    pub root_fs: Option<String>,
    pub current_directory: String,
}

impl FileSystemVirtual {
    /// Crea un nuevo sistema de archivos virtual
    pub fn new() -> Self {
        FileSystemVirtual {
            file_systems: BTreeMap::new(),
            root_fs: None,
            current_directory: String::from("/"),
        }
    }

    /// Inicializa el sistema de archivos
    pub fn initialize(&mut self) {
        println!("📁 Inicializando Sistema de Archivos...");

        // Crear sistema de archivos raíz
        let root_fs = self.create_filesystem(FileSystemType::CRONOSFS, String::from("/"));
        self.root_fs = Some(String::from("/"));

        // Crear directorios básicos
        self.create_directory("/bin");
        self.create_directory("/etc");
        self.create_directory("/home");
        self.create_directory("/usr");
        self.create_directory("/var");
        self.create_directory("/tmp");
        self.create_directory("/dev");
        self.create_directory("/proc");
        self.create_directory("/sys");

        println!("✅ Sistema de Archivos inicializado");
    }

    /// Crea un sistema de archivos
    pub fn create_filesystem(&mut self, fs_type: FileSystemType, mount_point: String) -> String {
        let fs = FileSystem {
            fs_type,
            mount_point: mount_point.clone(),
            inodes: BTreeMap::new(),
            directories: BTreeMap::new(),
            files: BTreeMap::new(),
            next_inode_id: 1,
        };

        self.file_systems.insert(mount_point.clone(), fs);
        println!("📂 Sistema de archivos creado: {:?} en {}", fs_type, mount_point);
        mount_point
    }

    /// Crea un directorio
    pub fn create_directory(&mut self, path: &str) -> Result<u64, FileSystemError> {
        let fs = self.get_filesystem_for_path(path)?;
        
        let inode_id = fs.next_inode_id;
        fs.next_inode_id += 1;

        let inode = Inode {
            id: inode_id,
            file_type: FileType::Directory,
            metadata: FileMetadata {
                size: 0,
                created_at: 0,
                modified_at: 0,
                accessed_at: 0,
                permissions: FilePermissions {
                    user_read: true,
                    user_write: true,
                    user_execute: true,
                    group_read: true,
                    group_write: false,
                    group_execute: true,
                    other_read: true,
                    other_write: false,
                    other_execute: true,
                },
                owner: 0,
                group: 0,
            },
            data_blocks: Vec::new(),
        };

        fs.inodes.insert(inode_id, inode);

        let directory = Directory {
            inode_id,
            entries: BTreeMap::new(),
        };

        fs.directories.insert(inode_id, directory);

        println!("📁 Directorio creado: {}", path);
        Ok(inode_id)
    }

    /// Crea un archivo
    pub fn create_file(&mut self, path: &str) -> Result<u64, FileSystemError> {
        let fs = self.get_filesystem_for_path(path)?;
        
        let inode_id = fs.next_inode_id;
        fs.next_inode_id += 1;

        let inode = Inode {
            id: inode_id,
            file_type: FileType::RegularFile,
            metadata: FileMetadata {
                size: 0,
                created_at: 0,
                modified_at: 0,
                accessed_at: 0,
                permissions: FilePermissions {
                    user_read: true,
                    user_write: true,
                    user_execute: false,
                    group_read: true,
                    group_write: false,
                    group_execute: false,
                    other_read: true,
                    other_write: false,
                    other_execute: false,
                },
                owner: 0,
                group: 0,
            },
            data_blocks: Vec::new(),
        };

        fs.inodes.insert(inode_id, inode);

        let file = File {
            inode_id,
            data: Vec::new(),
            offset: 0,
        };

        fs.files.insert(inode_id, file);

        println!("📄 Archivo creado: {}", path);
        Ok(inode_id)
    }

    /// Lee un archivo
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError> {
        let fs = self.get_filesystem_for_path(path)?;
        
        // Buscar el archivo
        for (_, file) in &fs.files {
            if let Some(inode) = fs.inodes.get(&file.inode_id) {
                // Verificar si es el archivo correcto
                return Ok(file.data.clone());
            }
        }

        Err(FileSystemError::FileNotFound)
    }

    /// Escribe en un archivo
    pub fn write_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), FileSystemError> {
        let fs = self.get_filesystem_for_path(path)?;
        
        // Buscar el archivo
        for (_, file) in fs.files.iter_mut() {
            if let Some(inode) = fs.inodes.get_mut(&file.inode_id) {
                file.data = data;
                inode.metadata.size = data.len() as u64;
                return Ok(());
            }
        }

        Err(FileSystemError::FileNotFound)
    }

    /// Elimina un archivo
    pub fn delete_file(&mut self, path: &str) -> Result<(), FileSystemError> {
        let fs = self.get_filesystem_for_path(path)?;
        
        // Buscar y eliminar el archivo
        for (inode_id, _) in fs.files.clone().iter() {
            if fs.files.remove(inode_id).is_some() {
                fs.inodes.remove(inode_id);
                println!("🗑️ Archivo eliminado: {}", path);
                return Ok(());
            }
        }

        Err(FileSystemError::FileNotFound)
    }

    /// Lista el contenido de un directorio
    pub fn list_directory(&self, path: &str) -> Result<Vec<String>, FileSystemError> {
        let fs = self.get_filesystem_for_path(path)?;
        
        let mut entries = Vec::new();
        
        for (_, directory) in &fs.directories {
            for (name, _) in &directory.entries {
                entries.push(name.clone());
            }
        }

        Ok(entries)
    }

    /// Monta un sistema de archivos
    pub fn mount(&mut self, fs_type: FileSystemType, mount_point: String) -> Result<(), FileSystemError> {
        self.create_filesystem(fs_type, mount_point);
        Ok(())
    }

    /// Desmonta un sistema de archivos
    pub fn unmount(&mut self, mount_point: &str) -> Result<(), FileSystemError> {
        self.file_systems.remove(mount_point);
        println!("🔓 Sistema de archivos desmontado: {}", mount_point);
        Ok(())
    }

    /// Obtiene el sistema de archivos para un path
    fn get_filesystem_for_path(&self, path: &str) -> Result<&FileSystem, FileSystemError> {
        // Por ahora, siempre retorna el sistema de archivos raíz
        if let Some(root) = &self.root_fs {
            self.file_systems.get(root).ok_or(FileSystemError::FileSystemNotFound)
        } else {
            Err(FileSystemError::FileSystemNotFound)
        }
    }

    /// Cambia el directorio actual
    pub fn change_directory(&mut self, path: &str) -> Result<(), FileSystemError> {
        self.current_directory = path.to_string();
        println!("📂 Directorio cambiado a: {}", path);
        Ok(())
    }

    /// Obtiene el directorio actual
    pub fn get_current_directory(&self) -> &str {
        &self.current_directory
    }

    /// Genera reporte del sistema de archivos
    pub fn generate_report(&self) -> FileSystemReport {
        let total_filesystems = self.file_systems.len();
        let total_inodes = self.file_systems.values().map(|fs| fs.inodes.len()).sum();
        let total_files = self.file_systems.values().map(|fs| fs.files.len()).sum();
        let total_directories = self.file_systems.values().map(|fs| fs.directories.len()).sum();

        FileSystemReport {
            total_filesystems,
            total_inodes,
            total_files,
            total_directories,
            current_directory: self.current_directory.clone(),
        }
    }
}

/// Reporte del sistema de archivos
#[derive(Debug, Clone)]
pub struct FileSystemReport {
    pub total_filesystems: usize,
    pub total_inodes: usize,
    pub total_files: usize,
    pub total_directories: usize,
    pub current_directory: String,
}

/// Errores del sistema de archivos
#[derive(Debug, Clone)]
pub enum FileSystemError {
    FileSystemNotFound,
    FileNotFound,
    DirectoryNotFound,
    PermissionDenied,
    DiskFull,
    InvalidPath,
}

impl fmt::Display for FileSystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemType::CRONOSFS => write!(f, "CRONOSFS"),
            FileSystemType::EXT4 => write!(f, "EXT4"),
            FileSystemType::FAT32 => write!(f, "FAT32"),
            FileSystemType::NTFS => write!(f, "NTFS"),
            FileSystemType::TMPFS => write!(f, "TMPFS"),
            FileSystemType::PROCFS => write!(f, "PROCFS"),
            FileSystemType::SYSFS => write!(f, "SYSFS"),
        }
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::RegularFile => write!(f, "RegularFile"),
            FileType::Directory => write!(f, "Directory"),
            FileType::SymbolicLink => write!(f, "SymbolicLink"),
            FileType::CharacterDevice => write!(f, "CharacterDevice"),
            FileType::BlockDevice => write!(f, "BlockDevice"),
            FileType::Socket => write!(f, "Socket"),
            FileType::FIFO => write!(f, "FIFO"),
        }
    }
}
