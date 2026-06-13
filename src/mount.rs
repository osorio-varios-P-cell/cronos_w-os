//! Mount/Unmount System Module
//! 
//! This module implements the mount/unmount system for filesystems.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Opciones de montaje
#[derive(Debug, Clone)]
pub struct MountOptions {
    /// Solo lectura
    pub read_only: bool,
    /// No ejecutar setuid
    pub no_suid: bool,
    /// No ejecutar dispositivos
    pub no_dev: bool,
    /// No ejecutar programas
    pub no_exec: bool,
    /// Síncrono
    pub sync: bool,
    /// Datos asíncronos
    pub async_data: bool,
}

impl MountOptions {
    /// Crear nuevas opciones por defecto
    pub fn new() -> Self {
        Self {
            read_only: false,
            no_suid: false,
            no_dev: false,
            no_exec: false,
            sync: false,
            async_data: false,
        }
    }

    /// Crear opciones de solo lectura
    pub fn read_only() -> Self {
        Self {
            read_only: true,
            no_suid: false,
            no_dev: false,
            no_exec: false,
            sync: false,
            async_data: false,
        }
    }
}

impl Default for MountOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Entrada de montaje
#[derive(Debug, Clone)]
pub struct MountEntry {
    /// ID del montaje
    pub mount_id: u32,
    /// Dispositivo fuente
    pub source: String,
    /// Tipo de filesystem
    pub fstype: String,
    /// Punto de montaje
    pub target: String,
    /// Opciones de montaje
    pub options: MountOptions,
    /// Montado
    pub mounted: bool,
}

impl MountEntry {
    /// Crear nueva entrada de montaje
    pub fn new(mount_id: u32, source: String, fstype: String, target: String, options: MountOptions) -> Self {
        Self {
            mount_id,
            source,
            fstype,
            target,
            options,
            mounted: false,
        }
    }

    /// Marcar como montado
    pub fn set_mounted(&mut self) {
        self.mounted = true;
    }

    /// Marcar como desmontado
    pub fn set_unmounted(&mut self) {
        self.mounted = false;
    }
}

/// Sistema de montaje
pub struct MountSystem {
    /// Entradas de montaje
    pub mounts: Vec<MountEntry>,
    /// Siguiente ID de montaje
    pub next_mount_id: u32,
}

impl MountSystem {
    /// Crear nuevo sistema de montaje
    pub fn new() -> Self {
        Self {
            mounts: Vec::new(),
            next_mount_id: 1,
        }
    }

    /// Montar filesystem
    pub fn mount(&mut self, source: String, fstype: String, target: String, options: MountOptions) -> Result<u32, String> {
        // Verificar si el punto de montaje ya existe
        for mount in &self.mounts {
            if mount.target == target && mount.mounted {
                return Err(format!("Mount point {} already in use", target));
            }
        }

        let mount_id = self.next_mount_id;
        let mut entry = MountEntry::new(mount_id, source, fstype, target, options);
        entry.set_mounted();
        
        self.mounts.push(entry);
        self.next_mount_id += 1;
        
        Ok(mount_id)
    }

    /// Desmontar filesystem
    pub fn unmount(&mut self, target: &str) -> Result<(), String> {
        let index = self.mounts.iter()
            .position(|m| m.target == target && m.mounted)
            .ok_or_else(|| String::from("Mount point not found or not mounted"))?;
        
        self.mounts[index].set_unmounted();
        self.mounts.remove(index);
        
        Ok(())
    }

    /// Obtener entrada de montaje por punto de montaje
    pub fn get_mount(&self, target: &str) -> Option<&MountEntry> {
        self.mounts.iter()
            .find(|m| m.target == target && m.mounted)
    }

    /// Obtener entrada de montaje por ID
    pub fn get_mount_by_id(&self, mount_id: u32) -> Option<&MountEntry> {
        self.mounts.iter()
            .find(|m| m.mount_id == mount_id)
    }

    /// Listar todos los montajes
    pub fn list_mounts(&self) -> Vec<&MountEntry> {
        self.mounts.iter()
            .filter(|m| m.mounted)
            .collect()
    }

    /// Verificar si un punto de montaje está en uso
    pub fn is_mounted(&self, target: &str) -> bool {
        self.mounts.iter()
            .any(|m| m.target == target && m.mounted)
    }

    /// Obtener número de montajes activos
    pub fn active_mount_count(&self) -> usize {
        self.mounts.iter()
            .filter(|m| m.mounted)
            .count()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Mount System Status\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("Active Mounts: {}\n\n", self.active_mount_count()));
        
        for mount in self.list_mounts() {
            report.push_str(&format!("Mount ID: {}\n", mount.mount_id));
            report.push_str(&format!("  Source: {}\n", mount.source));
            report.push_str(&format!("  Type: {}\n", mount.fstype));
            report.push_str(&format!("  Target: {}\n", mount.target));
            report.push_str(&format!("  Read-only: {}\n", mount.options.read_only));
            report.push_str(&format!("  Sync: {}\n\n", mount.options.sync));
        }
        
        report
    }
}

impl Default for MountSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de montaje
pub struct MountUtils;

impl MountUtils {
    /// Verificar si una ruta es válida como punto de montaje
    pub fn is_valid_mount_point(path: &str) -> bool {
        // Una ruta válida debe ser absoluta
        if !path.starts_with('/') {
            return false;
        }
        
        // No debe estar vacía
        if path.is_empty() {
            return false;
        }
        
        true
    }

    /// Normalizar ruta de montaje
    pub fn normalize_mount_point(path: &str) -> String {
        let mut result = String::from(path);
        
        // Asegurar que termina con /
        if !result.ends_with('/') {
            result.push('/');
        }
        
        result
    }

    /// Verificar si un tipo de filesystem es soportado
    pub fn is_supported_fstype(fstype: &str) -> bool {
        matches!(fstype, 
            "ext4" | "ext3" | "ext2" | "xfs" | "btrfs" | 
            "fat32" | "ntfs" | "iso9660" | "proc" | "sysfs" | 
            "tmpfs" | "devtmpfs" | "cgroup" | "overlay"
        )
    }
}
