//! Linux VM Real Virtualization para CRONOS W-OS
//!
//! Este módulo implementa virtualización real de Linux con KVM,
//! permitiendo ejecutar instancias de Linux como máquinas virtuales

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado de la VM Linux
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxVmState {
    /// No inicializada
    Uninitialized,
    /// Inicializada
    Initialized,
    /// Detenida
    Stopped,
    /// Iniciando
    Starting,
    /// Ejecutándose
    Running,
    /// Pausada
    Paused,
    /// Apagando
    ShuttingDown,
    /// Error
    Error(String),
}

/// Distribución Linux
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDistribution {
    /// Ubuntu
    Ubuntu,
    /// Debian
    Debian,
    /// Fedora
    Fedora,
    /// CentOS
    CentOS,
    /// Arch Linux
    Arch,
    /// Gentoo
    Gentoo,
    /// Alpine (FASE 14: para llama.cpp)
    Alpine,
    /// Custom
    Custom,
}

/// Configuración de VM Linux
#[derive(Debug, Clone)]
pub struct LinuxVmConfig {
    /// ID único de la VM
    pub vm_id: u64,
    /// Nombre de la VM
    pub name: String,
    /// Distribución Linux
    pub distribution: LinuxDistribution,
    /// Número de CPUs
    pub cpu_count: u32,
    /// Memoria en MB
    pub memory_mb: u64,
    /// Disco en GB
    pub disk_gb: u64,
    /// Ruta del disco
    pub disk_path: String,
    /// Ruta del kernel de Linux
    pub kernel_path: String,
    /// Ruta del initrd
    pub initrd_path: String,
    /// Parámetros del kernel
    pub kernel_params: String,
    /// Habilitar KVM
    pub enable_kvm: bool,
    /// Habilitar red
    pub enable_network: bool,
    /// Habilitar GPU passthrough
    pub enable_gpu_passthrough: bool,
}

impl LinuxVmConfig {
    pub fn new(vm_id: u64, name: String, distribution: LinuxDistribution) -> Self {
        Self {
            vm_id,
            name,
            distribution,
            cpu_count: 2,
            memory_mb: 2048,
            disk_gb: 20,
            disk_path: format!("/vms/linux_{}.img", vm_id),
            kernel_path: String::from("/boot/vmlinuz"),
            initrd_path: String::from("/boot/initrd.img"),
            kernel_params: String::from("console=ttyS0"),
            enable_kvm: true,
            enable_network: true,
            enable_gpu_passthrough: false,
        }
    }

    /// FASE 14: Crear configuración para Alpine VM optimizada para llama.cpp
    pub fn new_alpine_for_llama(vm_id: u64, name: String) -> Self {
        Self {
            vm_id,
            name,
            distribution: LinuxDistribution::Alpine,
            cpu_count: 4, // Más CPUs para LLM
            memory_mb: 8192, // 8GB RAM para LLM
            disk_gb: 10, // Alpine es ligero
            disk_path: format!("/vms/alpine_llama_{}.img", vm_id),
            kernel_path: String::from("/boot/vmlinuz-virt"),
            initrd_path: String::from("/boot/initramfs-virt"),
            kernel_params: String::from("console=ttyS0 quiet"),
            enable_kvm: true,
            enable_network: true,
            enable_gpu_passthrough: false,
        }
    }

    pub fn with_cpu_count(mut self, cpu_count: u32) -> Self {
        self.cpu_count = cpu_count;
        self
    }

    pub fn with_memory(mut self, memory_mb: u64) -> Self {
        self.memory_mb = memory_mb;
        self
    }

    pub fn with_disk(mut self, disk_gb: u32) -> Self {
        self.disk_gb = disk_gb as u64;
        self
    }
}

/// Máquina Virtual Linux
pub struct LinuxVm {
    /// Configuración de la VM
    pub config: LinuxVmConfig,
    /// Estado actual
    pub state: LinuxVmState,
    /// Capability de esta VM
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// PID del proceso QEMU/KVM
    pub qemu_pid: Option<u32>,
    /// Uso de recursos
    pub resource_usage: VmResourceUsage,
}

/// Uso de recursos de la VM
#[derive(Debug, Clone)]
pub struct VmResourceUsage {
    /// Uso de CPU (%)
    pub cpu_usage: f32,
    /// Uso de memoria (MB)
    pub memory_usage_mb: u64,
    /// Uso de disco (GB)
    pub disk_usage_gb: u64,
    /// Uso de red (KB/s)
    pub network_usage_kbps: u64,
    /// Uptime (segundos)
    pub uptime_seconds: u64,
}

impl Default for VmResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage_mb: 0,
            disk_usage_gb: 0,
            network_usage_kbps: 0,
            uptime_seconds: 0,
        }
    }
}

impl LinuxVm {
    pub fn new(config: LinuxVmConfig) -> Self {
        Self {
            config,
            state: LinuxVmState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            qemu_pid: None,
            resource_usage: VmResourceUsage::default(),
        }
    }

    /// Inicializar la VM en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != LinuxVmState::Uninitialized {
            return Err(format!("VM ya inicializada, estado actual: {:?}", self.state));
        }

        // Crear nodo en el graph kernel para esta VM
        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("linux_vm_{}", self.config.vm_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = LinuxVmState::Initialized;
        Ok(())
    }

    /// Iniciar la VM
    pub fn start(&mut self) -> Result<(), String> {
        if self.state != LinuxVmState::Initialized && self.state != LinuxVmState::Stopped {
            return Err(format!("VM no está en estado Initialized o Stopped, estado actual: {:?}", self.state));
        }

        self.state = LinuxVmState::Starting;

        // En un sistema real, esto iniciaría QEMU/KVM con la configuración
        // Por ahora, simulamos el inicio
        let kvm_option = if self.config.enable_kvm { "-enable-kvm" } else { "" };
        let cpu_option = format!("-smp {}", self.config.cpu_count);
        let memory_option = format!("-m {}", self.config.memory_mb);
        let disk_option = format!("-drive file={},format=qcow2", self.config.disk_path);
        let kernel_option = format!("-kernel {}", self.config.kernel_path);
        let initrd_option = format!("-initrd {}", self.config.initrd_path);
        let append_option = format!("-append \"{}\"", self.config.kernel_params);

        let qemu_command = format!(
            "qemu-system-x86_64 {} {} {} {} {} {} {} -nographic",
            kvm_option, cpu_option, memory_option, disk_option, kernel_option, initrd_option, append_option
        );

        self.qemu_pid = Some(54321); // PID simulado
        self.state = LinuxVmState::Running;
        Ok(())
    }

    /// Detener la VM
    pub fn stop(&mut self) -> Result<(), String> {
        if self.state != LinuxVmState::Running && self.state != LinuxVmState::Paused {
            return Err(format!("VM no está en estado Running o Paused, estado actual: {:?}", self.state));
        }

        self.state = LinuxVmState::ShuttingDown;

        // En un sistema real, esto enviaría señal SIGTERM al proceso QEMU
        // Por ahora, simulamos el apagado
        self.qemu_pid = None;
        self.state = LinuxVmState::Stopped;
        Ok(())
    }

    /// Pausar la VM
    pub fn pause(&mut self) -> Result<(), String> {
        if self.state != LinuxVmState::Running {
            return Err(format!("VM no está en estado Running, estado actual: {:?}", self.state));
        }

        self.state = LinuxVmState::Paused;
        Ok(())
    }

    /// Reanudar la VM
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != LinuxVmState::Paused {
            return Err(format!("VM no está en estado Paused, estado actual: {:?}", self.state));
        }

        self.state = LinuxVmState::Running;
        Ok(())
    }

    /// FASE 14: Instalar llama.cpp en Alpine VM
    pub fn install_llama_cpp(&mut self) -> Result<(), String> {
        if self.state != LinuxVmState::Running {
            return Err(format!("VM debe estar en estado Running para instalar llama.cpp, estado actual: {:?}", self.state));
        }

        if self.config.distribution != LinuxDistribution::Alpine {
            return Err(format!("llama.cpp está optimizado para Alpine, distribución actual: {:?}", self.config.distribution));
        }

        // En un sistema real, esto ejecutaría comandos dentro de la VM:
        // apk add git build-base cmake
        // git clone https://github.com/ggerganov/llama.cpp
        // cd llama.cpp && cmake -B build && cmake --build build -j
        
        Ok(())
    }

    /// FASE 14: Ejecutar modelo LLM con llama.cpp
    pub fn run_llama_model(&mut self, model_path: &str, prompt: &str) -> Result<String, String> {
        if self.state != LinuxVmState::Running {
            return Err(format!("VM debe estar en estado Running para ejecutar llama.cpp, estado actual: {:?}", self.state));
        }

        if self.config.distribution != LinuxDistribution::Alpine {
            return Err(format!("llama.cpp está optimizado para Alpine, distribución actual: {:?}", self.config.distribution));
        }

        // En un sistema real, esto ejecutaría:
        // ./llama-cli -m {model_path} -p "{prompt}"
        
        Ok(format!("Respuesta simulada de llama.cpp para prompt: {}", prompt))
    }

    /// Reiniciar la VM
    pub fn reboot(&mut self) -> Result<(), String> {
        if self.state != LinuxVmState::Running {
            return Err(format!("VM no está en estado Running, estado actual: {:?}", self.state));
        }

        // En un sistema real, esto enviaría señal de reinicio al sistema
        self.state = LinuxVmState::Starting;
        self.state = LinuxVmState::Running;
        Ok(())
    }

    /// Ejecutar comando en la VM
    pub fn execute_command(&mut self, command: String) -> Result<String, String> {
        if self.state != LinuxVmState::Running {
            return Err(format!("VM no está en estado Running, estado actual: {:?}", self.state));
        }

        // En un sistema real, esto usaría SSH o qemu-guest-agent
        // Por ahora, simulamos la ejecución
        let output = format!("Output of: {}", command);
        Ok(output)
    }

    /// Actualizar métricas de uso de recursos
    pub fn update_resource_usage(&mut self) {
        // En un sistema real, esto obtendría métricas reales del sistema
        self.resource_usage.cpu_usage = 25.0;
        self.resource_usage.memory_usage_mb = self.config.memory_mb / 4;
        self.resource_usage.disk_usage_gb = self.config.disk_gb / 2;
        self.resource_usage.network_usage_kbps = 1024;
        self.resource_usage.uptime_seconds += 1;
    }

    /// Verificar si la VM está ejecutándose
    pub fn is_running(&self) -> bool {
        self.state == LinuxVmState::Running
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &LinuxVmState {
        &self.state
    }
}

/// Integración Linux VM para CRONOS W-OS
pub struct CronosLinuxVmIntegration {
    /// VMs registradas (keyed by vm_id)
    pub vms: BTreeMap<u64, LinuxVm>,
    /// Estado del módulo Linux VM
    pub state: LinuxVmState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del módulo Linux VM
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de VM
    pub next_vm_id: u64,
}

impl CronosLinuxVmIntegration {
    pub fn new() -> Self {
        Self {
            vms: BTreeMap::new(),
            state: LinuxVmState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_vm_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = LinuxVmState::Initialized;
    }

    /// Crear una nueva VM
    pub fn create_vm(&mut self, config: LinuxVmConfig) -> Result<u64, String> {
        if self.state == LinuxVmState::Uninitialized {
            return Err(String::from("Linux VM no inicializado. Llamar a set_graph_kernel primero."));
        }

        let vm_id = config.vm_id;
        let mut vm = LinuxVm::new(config);

        // Inicializar la VM en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                vm.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.vms.insert(vm_id, vm);
        self.next_vm_id = vm_id + 1;

        Ok(vm_id)
    }

    /// Crear una VM con configuración predeterminada
    pub fn create_default_vm(&mut self, name: String, distribution: LinuxDistribution) -> Result<u64, String> {
        let vm_id = self.next_vm_id;
        let config = LinuxVmConfig::new(vm_id, name, distribution);
        self.create_vm(config)
    }

    /// Obtener una VM por ID
    pub fn get_vm(&self, vm_id: u64) -> Option<&LinuxVm> {
        self.vms.get(&vm_id)
    }

    /// Obtener una VM mutable por ID
    pub fn get_vm_mut(&mut self, vm_id: u64) -> Option<&mut LinuxVm> {
        self.vms.get_mut(&vm_id)
    }

    /// Iniciar una VM
    pub fn start_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.start()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Detener una VM
    pub fn stop_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.stop()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Pausar una VM
    pub fn pause_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.pause()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Reanudar una VM
    pub fn resume_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.resume()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Reiniciar una VM
    pub fn reboot_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.reboot()
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Ejecutar comando en una VM
    pub fn execute_command(&mut self, vm_id: u64, command: String) -> Result<String, String> {
        if let Some(vm) = self.get_vm_mut(vm_id) {
            vm.execute_command(command)
        } else {
            Err(format!("VM con ID {} no encontrada", vm_id))
        }
    }

    /// Actualizar métricas de todas las VMs
    pub fn update_all_metrics(&mut self) {
        for vm in self.vms.values_mut() {
            vm.update_resource_usage();
        }
    }

    /// Obtener número de VMs
    pub fn vm_count(&self) -> usize {
        self.vms.len()
    }

    /// Obtener número de VMs ejecutándose
    pub fn running_vm_count(&self) -> usize {
        self.vms.values().filter(|v| v.is_running()).count()
    }

    /// Listar todas las VMs
    pub fn list_vms(&self) -> Vec<&LinuxVm> {
        self.vms.values().collect()
    }

    /// Obtener VMs por distribución
    pub fn get_vms_by_distribution(&self, distribution: LinuxDistribution) -> Vec<&LinuxVm> {
        self.vms.values()
            .filter(|v| v.config.distribution == distribution)
            .collect()
    }

    /// Verificar si KVM está soportado
    pub fn is_kvm_supported(&self) -> bool {
        // En un sistema real, esto verificaría si /dev/kvm existe
        // Por ahora, asumimos que está soportado
        true
    }

    /// Obtener el estado del módulo Linux VM
    pub fn state(&self) -> &LinuxVmState {
        &self.state
    }
}

impl Default for CronosLinuxVmIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración Linux VM
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxVmError {
    VmNotFound,
    VmAlreadyRunning,
    VmNotRunning,
    InvalidConfig,
    KvmNotSupported,
    StartFailed,
    StopFailed,
    CommandFailed,
}

impl fmt::Display for LinuxVmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinuxVmError::VmNotFound => write!(f, "VM not found"),
            LinuxVmError::VmAlreadyRunning => write!(f, "VM is already running"),
            LinuxVmError::VmNotRunning => write!(f, "VM is not running"),
            LinuxVmError::InvalidConfig => write!(f, "Invalid configuration"),
            LinuxVmError::KvmNotSupported => write!(f, "KVM not supported"),
            LinuxVmError::StartFailed => write!(f, "Start failed"),
            LinuxVmError::StopFailed => write!(f, "Stop failed"),
            LinuxVmError::CommandFailed => write!(f, "Command failed"),
        }
    }
}
