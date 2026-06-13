//! File Permissions Module
//! 
//! This module implements file permissions and access control.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Permisos de archivo
#[derive(Debug, Clone, Copy)]
pub struct FilePermissions {
    /// Permisos del propietario (rwx)
    pub owner: u8,
    /// Permisos del grupo (rwx)
    pub group: u8,
    /// Permisos de otros (rwx)
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

    /// Permisos de archivo regular (644)
    pub fn regular_file() -> Self {
        Self::new(0o6, 0o4, 0o4)
    }

    /// Permisos de directorio (755)
    pub fn directory() -> Self {
        Self::new(0o7, 0o5, 0o5)
    }

    /// Permisos ejecutables (755)
    pub fn executable() -> Self {
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

    /// Verificar si el grupo puede leer
    pub fn group_can_read(&self) -> bool {
        self.group & 0o4 != 0
    }

    /// Verificar si el grupo puede escribir
    pub fn group_can_write(&self) -> bool {
        self.group & 0o2 != 0
    }

    /// Verificar si el grupo puede ejecutar
    pub fn group_can_execute(&self) -> bool {
        self.group & 0o1 != 0
    }

    /// Verificar si otros pueden leer
    pub fn other_can_read(&self) -> bool {
        self.other & 0o4 != 0
    }

    /// Verificar si otros pueden escribir
    pub fn other_can_write(&self) -> bool {
        self.other & 0o2 != 0
    }

    /// Verificar si otros pueden ejecutar
    pub fn other_can_execute(&self) -> bool {
        self.other & 0o1 != 0
    }

    /// Convertir a modo octal
    pub fn to_mode(&self) -> u32 {
        ((self.owner as u32) << 6) | ((self.group as u32) << 3) | (self.other as u32)
    }

    /// Crear desde modo octal
    pub fn from_mode(mode: u32) -> Self {
        Self {
            owner: ((mode >> 6) & 0o7) as u8,
            group: ((mode >> 3) & 0o7) as u8,
            other: (mode & 0o7) as u8,
        }
    }
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self::default()
    }
}

/// Información de propietario
#[derive(Debug, Clone)]
pub struct OwnerInfo {
    /// UID del propietario
    pub uid: u32,
    /// GID del grupo
    pub gid: u32,
}

impl OwnerInfo {
    /// Crear nueva información de propietario
    pub fn new(uid: u32, gid: u32) -> Self {
        Self { uid, gid }
    }

    /// Root (uid=0, gid=0)
    pub fn root() -> Self {
        Self::new(0, 0)
    }

    /// Verificar si es root
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
}

impl Default for OwnerInfo {
    fn default() -> Self {
        Self::root()
    }
}

/// Control de acceso
pub struct AccessControl {
    /// Permisos del archivo
    pub permissions: FilePermissions,
    /// Información del propietario
    pub owner: OwnerInfo,
}

impl AccessControl {
    /// Crear nuevo control de acceso
    pub fn new(permissions: FilePermissions, owner: OwnerInfo) -> Self {
        Self {
            permissions,
            owner,
        }
    }

    /// Verificar si un UID puede leer
    pub fn can_read(&self, uid: u32, gid: u32) -> bool {
        if self.owner.is_root() || uid == 0 {
            return true;
        }

        if uid == self.owner.uid {
            return self.permissions.owner_can_read();
        }

        if gid == self.owner.gid {
            return self.permissions.group_can_read();
        }

        self.permissions.other_can_read()
    }

    /// Verificar si un UID puede escribir
    pub fn can_write(&self, uid: u32, gid: u32) -> bool {
        if self.owner.is_root() || uid == 0 {
            return true;
        }

        if uid == self.owner.uid {
            return self.permissions.owner_can_write();
        }

        if gid == self.owner.gid {
            return self.permissions.group_can_write();
        }

        self.permissions.other_can_write()
    }

    /// Verificar si un UID puede ejecutar
    pub fn can_execute(&self, uid: u32, gid: u32) -> bool {
        if self.owner.is_root() || uid == 0 {
            return true;
        }

        if uid == self.owner.uid {
            return self.permissions.owner_can_execute();
        }

        if gid == self.owner.gid {
            return self.permissions.group_can_execute();
        }

        self.permissions.other_can_execute()
    }

    /// Cambiar permisos
    pub fn chmod(&mut self, mode: u32) {
        self.permissions = FilePermissions::from_mode(mode);
    }

    /// Cambiar propietario
    pub fn chown(&mut self, uid: u32, gid: u32) {
        self.owner = OwnerInfo::new(uid, gid);
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new(FilePermissions::default(), OwnerInfo::default())
    }
}

/// Gestor de permisos
pub struct PermissionManager {
    /// Controles de acceso
    pub access_controls: Vec<AccessControl>,
}

impl PermissionManager {
    /// Crear nuevo gestor de permisos
    pub fn new() -> Self {
        Self {
            access_controls: Vec::new(),
        }
    }

    /// Agregar control de acceso
    pub fn add(&mut self, access_control: AccessControl) {
        self.access_controls.push(access_control);
    }

    /// Obtener control de acceso por índice
    pub fn get(&self, index: usize) -> Option<&AccessControl> {
        self.access_controls.get(index)
    }

    /// Obtener control de acceso mutable por índice
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AccessControl> {
        self.access_controls.get_mut(index)
    }

    /// Verificar permisos de lectura
    pub fn check_read(&self, index: usize, uid: u32, gid: u32) -> bool {
        if let Some(ac) = self.get(index) {
            ac.can_read(uid, gid)
        } else {
            false
        }
    }

    /// Verificar permisos de escritura
    pub fn check_write(&self, index: usize, uid: u32, gid: u32) -> bool {
        if let Some(ac) = self.get(index) {
            ac.can_write(uid, gid)
        } else {
            false
        }
    }

    /// Verificar permisos de ejecución
    pub fn check_execute(&self, index: usize, uid: u32, gid: u32) -> bool {
        if let Some(ac) = self.get(index) {
            ac.can_execute(uid, gid)
        } else {
            false
        }
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Permission Manager Status\n");
        report.push_str("=========================\n\n");
        
        report.push_str(&format!("Access Controls: {}\n\n", self.access_controls.len()));
        
        for (i, ac) in self.access_controls.iter().enumerate() {
            report.push_str(&format!("Index: {}\n", i));
            report.push_str(&format!("  Owner UID: {}\n", ac.owner.uid));
            report.push_str(&format!("  Owner GID: {}\n", ac.owner.gid));
            report.push_str(&format!("  Mode: {:o}\n", ac.permissions.to_mode()));
            report.push_str(&format!("  Owner: rwx = {}{}{}\n", 
                if ac.permissions.owner_can_read() { "r" } else { "-" },
                if ac.permissions.owner_can_write() { "w" } else { "-" },
                if ac.permissions.owner_can_execute() { "x" } else { "-" }
            ));
            report.push_str(&format!("  Group: rwx = {}{}{}\n", 
                if ac.permissions.group_can_read() { "r" } else { "-" },
                if ac.permissions.group_can_write() { "w" } else { "-" },
                if ac.permissions.group_can_execute() { "x" } else { "-" }
            ));
            report.push_str(&format!("  Other: rwx = {}{}{}\n\n", 
                if ac.permissions.other_can_read() { "r" } else { "-" },
                if ac.permissions.other_can_write() { "w" } else { "-" },
                if ac.permissions.other_can_execute() { "x" } else { "-" }
            ));
        }
        
        report
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
