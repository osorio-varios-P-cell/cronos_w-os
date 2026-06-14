//! Virtualización General para CRONOS W-OS
//!
//! Este módulo implementa la virtualización de sistemas operativos (Windows/Linux)
//! usando QEMU/KVM adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Tipo de sistema operativo a virtualizar
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsType {
    /// Windows
    Windows,
    /// Linux
    Linux,
    /// macOS (Apple)
    Mac,
    /// Android (Subsystem)
    Android,
    /// Otro sistema
    Other(String),
}

/// Estado de la VM
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmState {
    /// Detenida
    Stopped,
    /// Inicializada
    Initialized,
    /// Iniciando
    Starting,
    /// Ejecutándose
    Running,
    /// Pausada
    Paused,
    /// Apagándose
    ShuttingDown,
    /// Error
    Error(String),
}

/// Configuración de la VM
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Nombre de la VM
    pub name: String,
    /// Tipo de sistema operativo
    pub os_type: OsType,
    /// Memoria en MB
    pub memory_mb: u32,
    /// Número de CPUs
    pub cpu_count: u32,
    /// Tamaño del disco en GB
    pub disk_size_gb: u32,
    /// Ruta al ISO de instalación
    pub iso_path: String,
    /// Ruta al disco virtual
    pub disk_image_path: String,
    /// Habilitar aceleración KVM
    pub enable_kvm: bool,
    /// Habilitar aceleración de hardware 3D
    pub enable_3d_acceleration: bool,
    /// Habilitar red
    pub enable_network: bool,
    /// Habilitar USB
    pub enable_usb: bool,
    /// Habilitar audio
    pub enable_audio: bool,
    /// FASE 16: Habilitar modo fluido (Seamless Mode)
    pub seamless_mode: bool,
}

impl VmConfig {
    pub fn new(name: String, os_type: OsType, disk_image_path: String) -> Self {
        Self {
            name,
            os_type,
            memory_mb: 4096,
            cpu_count: 4,
            disk_size_gb: 64,
            iso_path: String::new(),
            disk_image_path,
            enable_kvm: true,
            enable_3d_acceleration: true,
            enable_network: true,
            enable_usb: true,
            enable_audio: true,
            seamless_mode: false,
        }
    }

    pub fn with_memory(mut self, memory_mb: u32) -> Self {
        self.memory_mb = memory_mb;
        self
    }

    pub fn with_cpus(mut self, cpu_count: u32) -> Self {
        self.cpu_count = cpu_count;
        self
    }

    pub fn with_iso(mut self, iso_path: String) -> Self {
        self.iso_path = iso_path;
        self
    }

    pub fn with_kvm(mut self, enable: bool) -> Self {
        self.enable_kvm = enable;
        self
    }

    pub fn with_3d_acceleration(mut self, enable: bool) -> Self {
        self.enable_3d_acceleration = enable;
        self
    }

    pub fn with_network(mut self, enable: bool) -> Self {
        self.enable_network = enable;
        self
    }

    pub fn with_usb(mut self, enable: bool) -> Self {
        self.enable_usb = enable;
        self
    }

    pub fn with_audio(mut self, enable: bool) -> Self {
        self.enable_audio = enable;
        self
    }

    pub fn with_seamless_mode(mut self, enable: bool) -> Self {
        self.seamless_mode = enable;
        self
    }
}

/// Máquina Virtual
pub struct VirtualMachine {
    /// Configuración de la VM
    pub config: VmConfig,
    /// Estado actual
    pub state: VmState,
    /// Capability que representa esta VM en el grafo
    pub capability: Option<CapabilityId>,
    /// PID del proceso QEMU (si está ejecutándose)
    pub qemu_pid: Option<u32>,
    /// Uso de recursos
    pub resource_usage: VmResourceUsage,
    /// FASE 16: Portapapeles compartido (Host <-> Guest)
    pub guest_clipboard: String,
}

/// Uso de recursos de la VM
#[derive(Debug, Clone)]
pub struct VmResourceUsage {
    /// Uso de CPU (%)
    pub cpu_usage: f32,
    /// Uso de memoria (MB)
    pub memory_usage_mb: u32,
    /// Uso de disco (GB)
    pub disk_usage_gb: u32,
    /// Uso de red (KB/s)
    pub network_usage_kbps: u32,
}

impl Default for VmResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage_mb: 0,
            disk_usage_gb: 0,
            network_usage_kbps: 0,
        }
    }
}

impl VirtualMachine {
    pub fn new(config: VmConfig) -> Self {
        Self {
            config,
            state: VmState::Stopped,
            capability: None,
            qemu_pid: None,
            resource_usage: VmResourceUsage::default(),
            guest_clipboard: String::new(),
        }
    }

    /// Sincronizar portapapeles con el sistema anfitrión
    pub fn sync_clipboard(&mut self, host_clipboard: &str) {
        self.guest_clipboard = String::from(host_clipboard);
        // En un sistema real, esto se enviaría vía guest-agent
    }

    /// Configurar EPT (Extended Page Tables) en el grafo de recursos
    pub fn setup_ept(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if let Some(vm_node) = self.capability.map(|id| NodeId(id.0)) { // Simplificación de ID
            let ept_node = graph_kernel.create_node(
                NodeType::MemoryRegion,
                format!("ept_{}", self.config.name),
            );

            // Vincular EPT a la VM como una arista de mapeo virtual (estilo 2024)
            graph_kernel.create_edge(vm_node, ept_node, EdgeType::VirtualMapping);
            Ok(())
        } else {
            Err(String::from("VM node not found in graph"))
        }
    }



    /// Iniciar la VM
    pub fn start(&mut self) -> Result<(), String> {
        if self.state != VmState::Stopped {
            return Err(format!("VM no está en estado Stopped, estado actual: {:?}", self.state));
        }

        self.state = VmState::Starting;

        // Generar comando QEMU
        let qemu_command = self.generate_qemu_command();

        // En un sistema real, aquí se ejecutaría el comando QEMU
        // Por ahora, simulamos el inicio
        self.state = VmState::Running;
        self.qemu_pid = Some(12345); // PID simulado

        Ok(())
    }

    /// FASE 28: Gestión directa de estructuras VMCS/VMCB (Sovereign Hypervisor)
    /// En lugar de usar comandos externos, CRONOS gestiona el estado de la CPU virtual
    pub fn setup_vm_context(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        // Si es macOS, registramos el nodo de hardware SMC (Apple) con emulación de registros
        if self.config.os_type == OsType::Mac {
            let smc_node = graph_kernel.create_node(
                NodeType::HardwareDevice(crate::graph_kernel::HardwareType::Acpi),
                String::from("apple_smc_controller_v2"),
            );

            // FASE 28: Inyectar llaves SMC (OSK) para macOS real
            graph_kernel.invoke_node_operation_mut::<(), _, _>(smc_node, |node| {
                node.set_metadata(String::from("smc_version"), String::from("2.0"));
                node.set_metadata(String::from("apple_osk_valid"), String::from("true"));
            });

            if let Some(vm_node) = self.capability.map(|id| NodeId(id.0)) {
                graph_kernel.create_edge(vm_node, smc_node, EdgeType::Dependency);
            }
        }
        // En hardware real, aquí configuraríamos la estructura de control de VM (VMCS)
        // vinculándola como un nodo de hardware crítico en el GraphKernel.
        self.state = VmState::Initialized;
        Ok(())
    }

    /// Detener la VM
    pub fn stop(&mut self) -> Result<(), String> {
        if self.state != VmState::Running && self.state != VmState::Paused {
            return Err(format!("VM no está en estado Running o Paused, estado actual: {:?}", self.state));
        }

        self.state = VmState::ShuttingDown;

        // En un sistema real, aquí se enviaría señal SIGTERM al proceso QEMU
        self.state = VmState::Stopped;
        self.qemu_pid = None;

        Ok(())
    }

    /// Pausar la VM
    pub fn pause(&mut self) -> Result<(), String> {
        if self.state != VmState::Running {
            return Err(format!("VM no está en estado Running, estado actual: {:?}", self.state));
        }

        // En un sistema real, aquí se enviaría señal SIGSTOP al proceso QEMU
        self.state = VmState::Paused;

        Ok(())
    }

    /// Reanudar la VM
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != VmState::Paused {
            return Err(format!("VM no está en estado Paused, estado actual: {:?}", self.state));
        }

        // En un sistema real, aquí se enviaría señal SIGCONT al proceso QEMU
        self.state = VmState::Running;

        Ok(())
    }


    /// Generar el comando QEMU para iniciar la VM (Fallback para entornos sin VT-x)
    fn generate_qemu_command(&self) -> String {
        let mut command = String::from("qemu-system-x86_64");

        // Memoria
        command.push_str(&format!(" -m {}", self.config.memory_mb));

        // CPUs
        command.push_str(&format!(" -smp {}", self.config.cpu_count));

        // KVM
        if self.config.enable_kvm {
            command.push_str(" -enable-kvm");
        }

        // Disco
        command.push_str(&format!(" -drive file={},format=qcow2", self.config.disk_image_path));

        // ISO de instalación
        if !self.config.iso_path.is_empty() {
            command.push_str(&format!(" -cdrom {}", self.config.iso_path));
        }

        // Red
        if self.config.enable_network {
            command.push_str(" -net nic,model=virtio -net user");
        }

        // USB
        if self.config.enable_usb {
            command.push_str(" -usb -device usb-tablet");
        }

        // Audio
        if self.config.enable_audio {
            command.push_str(" -soundhw hda");
        }

        // Aceleración 3D
        if self.config.enable_3d_acceleration {
            command.push_str(" -vga virtio -display gtk,gl=on");
        } else {
            command.push_str(" -vga std");
        }

        // Configuración específica según tipo de OS
        match self.config.os_type {
            OsType::Windows => {
                command.push_str(" -machine type=pc,accel=kvm");
                command.push_str(" -device qemu-xhci");
            }
            OsType::Linux => {
                command.push_str(" -machine type=q35,accel=kvm");
            }
            OsType::Mac => {
                // Configuración para macOS (Basado en patrones de OpenCore/OSX-KVM)
                command.push_str(" -machine q35,accel=kvm -device isa-applesmc,osk=\"...\"");
                command.push_str(" -cpu Penryn,vendor=GenuineIntel");
            }
            OsType::Android => {
                command.push_str(" -machine type=q35,accel=kvm -device virtio-vga-gl");
            }
            OsType::Other(_) => {
                command.push_str(" -machine type=pc");
            }
        }

        // Nombre de la VM
        command.push_str(&format!(" -name {}", self.config.name));

        command
    }

    /// Actualizar métricas de uso de recursos
    pub fn update_resource_usage(&mut self) {
        // En un sistema real, aquí se leerían las métricas del proceso QEMU
        // Por ahora, usamos valores simulados
        if self.state == VmState::Running {
            self.resource_usage.cpu_usage = 35.0;
            self.resource_usage.memory_usage_mb = self.config.memory_mb / 2;
            self.resource_usage.disk_usage_gb = self.config.disk_size_gb / 4;
            self.resource_usage.network_usage_kbps = 50;
        } else {
            self.resource_usage = VmResourceUsage::default();
        }
    }

    /// Obtener el uso de recursos
    pub fn resource_usage(&self) -> &VmResourceUsage {
        &self.resource_usage
    }

    /// Verificar si la VM está ejecutándose
    pub fn is_running(&self) -> bool {
        self.state == VmState::Running
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &VmState {
        &self.state
    }
}

/// Gestor de virtualización
pub struct VirtualizationManager {
    /// VMs registradas
    pub vms: Vec<VirtualMachine>,
    /// Capability del graph kernel para registrar VMs como nodos
    graph_kernel_capability: Option<CapabilityId>,
}

impl VirtualizationManager {
    pub fn new() -> Self {
        Self {
            vms: Vec::new(),
            graph_kernel_capability: None,
        }
    }

    /// Crear una nueva VM
    pub fn create_vm(&mut self, config: VmConfig) -> Result<u32, String> {
        let vm_id = self.vms.len() as u32;
        let mut vm = VirtualMachine::new(config);
        
        // En un sistema completo, aquí se registraría la VM como nodo en el grafo
        // usando invoke_capability con el graph_kernel
        
        self.vms.push(vm);
        Ok(vm_id)
    }

    /// Crear una VM Windows con configuración predeterminada
    pub fn create_windows_vm(&mut self, name: String, disk_image_path: String) -> Result<u32, String> {
        let config = VmConfig::new(name, OsType::Windows, disk_image_path)
            .with_memory(8192)
            .with_cpus(8)
            .with_3d_acceleration(true);
        self.create_vm(config)
    }

    /// Crear una VM Linux con configuración predeterminada
    pub fn create_linux_vm(&mut self, name: String, disk_image_path: String) -> Result<u32, String> {
        let config = VmConfig::new(name, OsType::Linux, disk_image_path)
            .with_memory(4096)
            .with_cpus(4)
            .with_3d_acceleration(false);
        self.create_vm(config)
    }

    /// Obtener una VM por ID
    pub fn get_vm(&self, vm_id: u32) -> Option<&VirtualMachine> {
        self.vms.get(vm_id as usize)
    }

    /// Obtener una VM mutable por ID
    pub fn get_vm_mut(&mut self, vm_id: u32) -> Option<&mut VirtualMachine> {
        self.vms.get_mut(vm_id as usize)
    }

    /// Iniciar una VM
    pub fn start_vm(&mut self, vm_id: u32) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.start()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Detener una VM
    pub fn stop_vm(&mut self, vm_id: u32) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.stop()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Pausar una VM
    pub fn pause_vm(&mut self, vm_id: u32) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.pause()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Reanudar una VM
    pub fn resume_vm(&mut self, vm_id: u32) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.resume()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Actualizar métricas de todas las VMs
    pub fn update_all_metrics(&mut self) {
        for vm in &mut self.vms {
            vm.update_resource_usage();
        }
    }

    /// Obtener número de VMs
    pub fn vm_count(&self) -> usize {
        self.vms.len()
    }

    /// Obtener número de VMs ejecutándose
    pub fn running_vm_count(&self) -> usize {
        self.vms.iter().filter(|vm| vm.is_running()).count()
    }

    /// Listar todas las VMs
    pub fn list_vms(&self) -> &[VirtualMachine] {
        &self.vms
    }
}

impl Default for VirtualizationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de virtualización
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualizationError {
    VmNotFound,
    VmAlreadyRunning,
    VmNotRunning,
    InvalidConfig,
    QemuNotAvailable,
    DiskImageNotFound,
    IsoNotFound,
    InsufficientResources,
}

impl fmt::Display for VirtualizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualizationError::VmNotFound => write!(f, "VM not found"),
            VirtualizationError::VmAlreadyRunning => write!(f, "VM is already running"),
            VirtualizationError::VmNotRunning => write!(f, "VM is not running"),
            VirtualizationError::InvalidConfig => write!(f, "Invalid configuration"),
            VirtualizationError::QemuNotAvailable => write!(f, "QEMU not available"),
            VirtualizationError::DiskImageNotFound => write!(f, "Disk image not found"),
            VirtualizationError::IsoNotFound => write!(f, "ISO image not found"),
            VirtualizationError::InsufficientResources => write!(f, "Insufficient system resources"),
        }
    }
}
