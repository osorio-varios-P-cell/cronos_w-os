//! Filesystem Integration Module
//! 
//! This module integrates all filesystems with the VFS layer.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de filesystem
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsType {
    /// FAT32
    Fat32,
    /// EXT4
    Ext4,
    /// NTFS
    Ntfs,
    /// BTRFS
    Btrfs,
    /// XFS
    Xfs,
    /// TMPFS (memoria)
    Tmpfs,
    /// PROCFS (procesos)
    Procfs,
    /// SYSFS (sistema)
    Sysfs,
    /// Devtmpfs (dispositivos)
    Devtmpfs,
}

impl FsType {
    /// Crear desde string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fat32" | "vfat" => Some(FsType::Fat32),
            "ext4" => Some(FsType::Ext4),
            "ntfs" => Some(FsType::Ntfs),
            "btrfs" => Some(FsType::Btrfs),
            "xfs" => Some(FsType::Xfs),
            "tmpfs" => Some(FsType::Tmpfs),
            "proc" | "procfs" => Some(FsType::Procfs),
            "sysfs" => Some(FsType::Sysfs),
            "devtmpfs" => Some(FsType::Devtmpfs),
            _ => None,
        }
    }

    /// Convertir a string
    pub fn to_str(&self) -> &'static str {
        match self {
            FsType::Fat32 => "fat32",
            FsType::Ext4 => "ext4",
            FsType::Ntfs => "ntfs",
            FsType::Btrfs => "btrfs",
            FsType::Xfs => "xfs",
            FsType::Tmpfs => "tmpfs",
            FsType::Procfs => "proc",
            FsType::Sysfs => "sysfs",
            FsType::Devtmpfs => "devtmpfs",
        }
    }
}

/// Descriptor de filesystem
#[derive(Debug, Clone)]
pub struct FsDescriptor {
    /// Tipo de filesystem
    pub fs_type: FsType,
    /// Nombre del dispositivo
    pub device: String,
    /// Montado
    pub mounted: bool,
    /// Punto de montaje
    pub mount_point: String,
}

impl FsDescriptor {
    /// Crear nuevo descriptor
    pub fn new(fs_type: FsType, device: String) -> Self {
        Self {
            fs_type,
            device,
            mounted: false,
            mount_point: String::new(),
        }
    }

    /// Marcar como montado
    pub fn set_mounted(&mut self, mount_point: String) {
        self.mounted = true;
        self.mount_point = mount_point;
    }

    /// Marcar como desmontado
    pub fn set_unmounted(&mut self) {
        self.mounted = false;
        self.mount_point.clear();
    }
}

/// Operaciones de filesystem
pub trait FsOperations {
    /// Montar filesystem
    fn mount(&mut self, mount_point: String) -> Result<(), String>;
    
    /// Desmontar filesystem
    fn unmount(&mut self) -> Result<(), String>;
    
    /// Leer archivo
    fn read(&self, path: &str) -> Result<Vec<u8>, String>;
    
    /// Escribir archivo
    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), String>;
    
    /// Crear directorio
    fn mkdir(&mut self, path: &str) -> Result<(), String>;
    
    /// Eliminar archivo
    fn unlink(&mut self, path: &str) -> Result<(), String>;
    
    /// Listar directorio
    fn readdir(&self, path: &str) -> Result<Vec<String>, String>;
    
    /// Obtener estadísticas
    fn stat(&self, path: &str) -> Result<FsStats, String>;
}

/// Estadísticas de filesystem
#[derive(Debug, Clone)]
pub struct FsStats {
    /// Tamaño total
    pub total_size: u64,
    /// Espacio libre
    pub free_space: u64,
    /// Espacio usado
    pub used_space: u64,
    /// Número de archivos
    pub file_count: u64,
    /// Número de directorios
    pub dir_count: u64,
}

impl FsStats {
    /// Crear nuevas estadísticas
    pub fn new(total_size: u64, free_space: u64) -> Self {
        let used_space = total_size - free_space;
        Self {
            total_size,
            free_space,
            used_space,
            file_count: 0,
            dir_count: 0,
        }
    }

    /// Calcular porcentaje usado
    pub fn used_percent(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.used_space as f64) / (self.total_size as f64) * 100.0
        }
    }
}

/// Sistema de integración de filesystems
pub struct FsIntegration {
    /// Descriptores de filesystems
    pub filesystems: Vec<FsDescriptor>,
    /// Siguiente ID de filesystem
    pub next_fs_id: u32,
}

impl FsIntegration {
    /// Crear nuevo sistema de integración
    pub fn new() -> Self {
        Self {
            filesystems: Vec::new(),
            next_fs_id: 1,
        }
    }

    /// Registrar filesystem
    pub fn register(&mut self, fs_type: FsType, device: String) -> Result<u32, String> {
        let descriptor = FsDescriptor::new(fs_type, device);
        let fs_id = self.next_fs_id;
        self.filesystems.push(descriptor);
        self.next_fs_id += 1;
        Ok(fs_id)
    }

    /// Montar filesystem
    pub fn mount(&mut self, fs_id: u32, mount_point: String) -> Result<(), String> {
        let descriptor = self.filesystems.get_mut(fs_id as usize)
            .ok_or_else(|| String::from("Filesystem not found"))?;
        
        if descriptor.mounted {
            return Err(String::from("Filesystem already mounted"));
        }
        
        descriptor.set_mounted(mount_point);
        Ok(())
    }

    /// Desmontar filesystem
    pub fn unmount(&mut self, fs_id: u32) -> Result<(), String> {
        let descriptor = self.filesystems.get_mut(fs_id as usize)
            .ok_or_else(|| String::from("Filesystem not found"))?;
        
        if !descriptor.mounted {
            return Err(String::from("Filesystem not mounted"));
        }
        
        descriptor.set_unmounted();
        Ok(())
    }

    /// Obtener descriptor por ID
    pub fn get(&self, fs_id: u32) -> Option<&FsDescriptor> {
        self.filesystems.get(fs_id as usize)
    }

    /// Obtener descriptor por punto de montaje
    pub fn get_by_mount_point(&self, mount_point: &str) -> Option<&FsDescriptor> {
        self.filesystems.iter()
            .find(|fs| fs.mounted && fs.mount_point == mount_point)
    }

    /// Listar filesystems montados
    pub fn list_mounted(&self) -> Vec<&FsDescriptor> {
        self.filesystems.iter()
            .filter(|fs| fs.mounted)
            .collect()
    }

    /// Obtener número de filesystems
    pub fn fs_count(&self) -> usize {
        self.filesystems.len()
    }

    /// Obtener número de filesystems montados
    pub fn mounted_count(&self) -> usize {
        self.filesystems.iter()
            .filter(|fs| fs.mounted)
            .count()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Filesystem Integration Status\n");
        report.push_str("==============================\n\n");
        
        report.push_str(&format!("Total Filesystems: {}\n", self.fs_count()));
        report.push_str(&format!("Mounted Filesystems: {}\n\n", self.mounted_count()));
        
        for (i, fs) in self.filesystems.iter().enumerate() {
            report.push_str(&format!("ID: {}\n", i));
            report.push_str(&format!("  Type: {}\n", fs.fs_type.to_str()));
            report.push_str(&format!("  Device: {}\n", fs.device));
            report.push_str(&format!("  Mounted: {}\n", fs.mounted));
            if fs.mounted {
                report.push_str(&format!("  Mount Point: {}\n", fs.mount_point));
            }
            report.push('\n');
        }
        
        report
    }
}

impl Default for FsIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de filesystem
pub struct FsUtils;

impl FsUtils {
    /// Verificar si un tipo de filesystem es soportado
    pub fn is_supported(fs_type: FsType) -> bool {
        matches!(fs_type,
            FsType::Fat32 | FsType::Ext4 | FsType::Ntfs |
            FsType::Btrfs | FsType::Xfs | FsType::Tmpfs |
            FsType::Procfs | FsType::Sysfs | FsType::Devtmpfs
        )
    }

    /// Verificar si un punto de montaje es válido
    pub fn is_valid_mount_point(path: &str) -> bool {
        path.starts_with('/') && !path.is_empty()
    }

    /// Normalizar ruta de archivo
    pub fn normalize_path(path: &str) -> String {
        let mut result = String::new();
        let mut components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        for component in components {
            if component == "." {
                continue;
            } else if component == ".." {
                // Manejar ".."
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
}
