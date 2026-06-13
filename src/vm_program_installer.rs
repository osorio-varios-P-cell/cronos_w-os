//! VM Program Installer para CRONOS W-OS
//!
//! Este módulo implementa instalación de programas en VMs,
//! permitiendo automatizar la instalación de software en Linux, Windows y Android

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Tipo de sistema operativo de la VM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmOsType {
    /// Linux
    Linux,
    /// Windows
    Windows,
    /// Android
    Android,
}

/// Tipo de gestor de paquetes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerType {
    /// APT (Debian/Ubuntu)
    Apt,
    /// YUM/DNF (Fedora/CentOS)
    Yum,
    /// Pacman (Arch Linux)
    Pacman,
    /// Snap
    Snap,
    /// Flatpak
    Flatpak,
    /// Winget (Windows)
    Winget,
    /// Chocolatey (Windows)
    Chocolatey,
    /// ADB (Android)
    Adb,
    /// Custom
    Custom,
}

/// Estado de instalación
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationState {
    /// Pendiente
    Pending,
    /// Descargando
    Downloading,
    /// Instalando
    Installing,
    /// Configurando
    Configuring,
    /// Completado
    Completed,
    /// Fallido
    Failed,
}

/// Configuración de instalación de programa
#[derive(Debug, Clone)]
pub struct ProgramInstallConfig {
    /// ID único de la instalación
    pub install_id: u64,
    /// ID de la VM
    pub vm_id: u64,
    /// Tipo de sistema operativo
    pub os_type: VmOsType,
    /// Tipo de gestor de paquetes
    pub package_manager: PackageManagerType,
    /// Nombre del paquete/programa
    pub package_name: String,
    /// Versión específica (opcional)
    pub version: Option<String>,
    /// Argumentos adicionales
    pub additional_args: Vec<String>,
    /// Habilitar instalación silenciosa
    pub silent_install: bool,
    /// Habilitar auto-confirmación
    pub auto_confirm: bool,
}

impl ProgramInstallConfig {
    pub fn new(install_id: u64, vm_id: u64, os_type: VmOsType, package_manager: PackageManagerType, package_name: String) -> Self {
        Self {
            install_id,
            vm_id,
            os_type,
            package_manager,
            package_name,
            version: None,
            additional_args: Vec::new(),
            silent_install: true,
            auto_confirm: true,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.additional_args = args;
        self
    }
}

/// Resultado de instalación
#[derive(Debug, Clone)]
pub struct InstallationResult {
    /// ID de la instalación
    pub install_id: u64,
    /// Estado de la instalación
    pub state: InstallationState,
    /// Tiempo de instalación (ms)
    pub install_time_ms: u64,
    /// Mensaje de error (si falló)
    pub error_message: Option<String>,
    /// Ruta de instalación
    pub install_path: Option<String>,
}

/// Tarea de instalación de programa
pub struct ProgramInstallTask {
    /// Configuración de la instalación
    pub config: ProgramInstallConfig,
    /// Estado actual
    pub state: InstallationState,
    /// Capability de esta tarea
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Resultado de instalación
    pub result: Option<InstallationResult>,
}

impl ProgramInstallTask {
    pub fn new(config: ProgramInstallConfig) -> Self {
        Self {
            config,
            state: InstallationState::Pending,
            capability_id: None,
            graph_node_id: None,
            result: None,
        }
    }

    /// Inicializar la tarea en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != InstallationState::Pending {
            return Err(format!("Tarea ya inicializada, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("install_task_{}", self.config.install_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = InstallationState::Pending;
        Ok(())
    }

    /// Ejecutar la instalación
    pub fn execute(&mut self) -> Result<(), String> {
        if self.state != InstallationState::Pending {
            return Err(format!("Tarea no está en estado Pending, estado actual: {:?}", self.state));
        }

        self.state = InstallationState::Downloading;

        // Generar comando de instalación según el tipo de gestor de paquetes
        let install_command = match self.config.package_manager {
            PackageManagerType::Apt => {
                let confirm = if self.config.auto_confirm { "-y" } else { "" };
                format!("apt install {} {}", confirm, self.config.package_name)
            }
            PackageManagerType::Yum => {
                let confirm = if self.config.auto_confirm { "-y" } else { "" };
                format!("yum install {} {}", confirm, self.config.package_name)
            }
            PackageManagerType::Pacman => {
                let confirm = if self.config.auto_confirm { "--noconfirm" } else { "" };
                format!("pacman -S {} {}", confirm, self.config.package_name)
            }
            PackageManagerType::Snap => {
                format!("snap install {}", self.config.package_name)
            }
            PackageManagerType::Flatpak => {
                format!("flatpak install {}", self.config.package_name)
            }
            PackageManagerType::Winget => {
                let silent = if self.config.silent_install { "--silent" } else { "" };
                format!("winget install {} {}", silent, self.config.package_name)
            }
            PackageManagerType::Chocolatey => {
                let silent = if self.config.silent_install { "-y" } else { "" };
                format!("choco install {} {}", silent, self.config.package_name)
            }
            PackageManagerType::Adb => {
                format!("adb install {}", self.config.package_name)
            }
            PackageManagerType::Custom => {
                self.config.package_name.clone()
            }
        };

        self.state = InstallationState::Installing;

        // Simular instalación
        let result = InstallationResult {
            install_id: self.config.install_id,
            state: InstallationState::Completed,
            install_time_ms: 5000,
            error_message: None,
            install_path: Some(format!("/usr/bin/{}", self.config.package_name)),
        };

        self.result = Some(result);
        self.state = InstallationState::Completed;
        Ok(())
    }

    /// Obtener el resultado de instalación
    pub fn get_result(&self) -> Option<&InstallationResult> {
        self.result.as_ref()
    }

    /// Verificar si la instalación está completada
    pub fn is_completed(&self) -> bool {
        self.state == InstallationState::Completed
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &InstallationState {
        &self.state
    }
}

/// Integración VM Program Installer para CRONOS W-OS
pub struct CronosVmProgramInstaller {
    /// Tareas de instalación (keyed by install_id)
    pub tasks: BTreeMap<u64, ProgramInstallTask>,
    /// Estado del módulo
    pub state: InstallationState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del módulo
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de tarea
    pub next_install_id: u64,
}

impl CronosVmProgramInstaller {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            state: InstallationState::Pending,
            graph_kernel: None,
            capability_id: None,
            next_install_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = InstallationState::Pending;
    }

    /// Crear una nueva tarea de instalación
    pub fn create_install_task(&mut self, config: ProgramInstallConfig) -> Result<u64, String> {
        if self.graph_kernel.is_none() {
            return Err(String::from("VM Program Installer no inicializado. Llamar a set_graph_kernel primero."));
        }

        let install_id = config.install_id;
        let mut task = ProgramInstallTask::new(config);

        // Inicializar la tarea en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                task.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.tasks.insert(install_id, task);
        self.next_install_id = install_id + 1;

        Ok(install_id)
    }

    /// Crear una tarea de instalación predeterminada
    pub fn create_default_install(&mut self, vm_id: u64, os_type: VmOsType, package_manager: PackageManagerType, package_name: String) -> Result<u64, String> {
        let install_id = self.next_install_id;
        let config = ProgramInstallConfig::new(install_id, vm_id, os_type, package_manager, package_name);
        self.create_install_task(config)
    }

    /// Obtener una tarea por ID
    pub fn get_task(&self, install_id: u64) -> Option<&ProgramInstallTask> {
        self.tasks.get(&install_id)
    }

    /// Obtener una tarea mutable por ID
    pub fn get_task_mut(&mut self, install_id: u64) -> Option<&mut ProgramInstallTask> {
        self.tasks.get_mut(&install_id)
    }

    /// Ejecutar una tarea de instalación
    pub fn execute_install(&mut self, install_id: u64) -> Result<(), String> {
        if let Some(task) = self.get_task_mut(install_id) {
            task.execute()
        } else {
            Err(format!("Tarea con ID {} no encontrada", install_id))
        }
    }

    /// Instalar programa en VM Linux
    pub fn install_linux_package(&mut self, vm_id: u64, package_manager: PackageManagerType, package_name: String) -> Result<u64, String> {
        let install_id = self.next_install_id;
        let config = ProgramInstallConfig::new(install_id, vm_id, VmOsType::Linux, package_manager, package_name);
        self.create_install_task(config)?;
        self.execute_install(install_id)?;
        Ok(install_id)
    }

    /// Instalar programa en VM Windows
    pub fn install_windows_program(&mut self, vm_id: u64, package_manager: PackageManagerType, program_name: String) -> Result<u64, String> {
        let install_id = self.next_install_id;
        let config = ProgramInstallConfig::new(install_id, vm_id, VmOsType::Windows, package_manager, program_name);
        self.create_install_task(config)?;
        self.execute_install(install_id)?;
        Ok(install_id)
    }

    /// Instalar APK en VM Android
    pub fn install_android_apk(&mut self, vm_id: u64, apk_path: String) -> Result<u64, String> {
        let install_id = self.next_install_id;
        let config = ProgramInstallConfig::new(install_id, vm_id, VmOsType::Android, PackageManagerType::Adb, apk_path);
        self.create_install_task(config)?;
        self.execute_install(install_id)?;
        Ok(install_id)
    }

    /// Obtener número de tareas
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Obtener número de tareas completadas
    pub fn completed_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_completed()).count()
    }

    /// Listar todas las tareas
    pub fn list_tasks(&self) -> Vec<&ProgramInstallTask> {
        self.tasks.values().collect()
    }

    /// Obtener tareas por VM
    pub fn get_tasks_by_vm(&self, vm_id: u64) -> Vec<&ProgramInstallTask> {
        self.tasks.values()
            .filter(|t| t.config.vm_id == vm_id)
            .collect()
    }

    /// Obtener tareas por tipo de OS
    pub fn get_tasks_by_os_type(&self, os_type: VmOsType) -> Vec<&ProgramInstallTask> {
        self.tasks.values()
            .filter(|t| t.config.os_type == os_type)
            .collect()
    }

    /// Obtener el estado del módulo
    pub fn state(&self) -> &InstallationState {
        &self.state
    }
}

impl Default for CronosVmProgramInstaller {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración VM Program Installer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmProgramInstallerError {
    TaskNotFound,
    TaskAlreadyCompleted,
    TaskNotPending,
    InvalidConfig,
    InstallationFailed,
    PackageManagerNotSupported,
    VmNotFound,
}

impl fmt::Display for VmProgramInstallerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmProgramInstallerError::TaskNotFound => write!(f, "Task not found"),
            VmProgramInstallerError::TaskAlreadyCompleted => write!(f, "Task already completed"),
            VmProgramInstallerError::TaskNotPending => write!(f, "Task not pending"),
            VmProgramInstallerError::InvalidConfig => write!(f, "Invalid configuration"),
            VmProgramInstallerError::InstallationFailed => write!(f, "Installation failed"),
            VmProgramInstallerError::PackageManagerNotSupported => write!(f, "Package manager not supported"),
            VmProgramInstallerError::VmNotFound => write!(f, "VM not found"),
        }
    }
}
