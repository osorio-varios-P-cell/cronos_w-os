//! KVM/QEMU Integration para CRONOS W-OS
//!
//! Este módulo adapta KVM (Kernel-based Virtual Machine) y QEMU
//! al exokernel CRONOS W-OS, integrando con el graph kernel y capabilities

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del módulo KVM
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvmState {
    /// No inicializado
    Uninitialized,
    /// Inicializado pero sin VMs
    Idle,
    /// Ejecutando VMs
    Active,
    /// Error
    Error(String),
}

/// Configuración de VM KVM
#[derive(Debug, Clone)]
pub struct KvmVmConfig {
    /// ID único de la VM
    pub vm_id: u64,
    /// Nombre de la VM
    pub name: String,
    /// Memoria en MB
    pub memory_mb: u32,
    /// Número de vCPUs
    pub vcpu_count: u32,
    /// Tamaño del disco en GB
    pub disk_size_gb: u32,
    /// Ruta al disco virtual
    pub disk_path: String,
    /// Habilitar virtualización anidada
    pub enable_nested_virt: bool,
    /// Habilitar aceleración de hardware
    pub enable_hardware_accel: bool,
}

impl KvmVmConfig {
    pub fn new(vm_id: u64, name: String, disk_path: String) -> Self {
        Self {
            vm_id,
            name,
            memory_mb: 2048,
            vcpu_count: 2,
            disk_size_gb: 16,
            disk_path,
            enable_nested_virt: false,
            enable_hardware_accel: true,
        }
    }

    pub fn with_memory(mut self, memory_mb: u32) -> Self {
        self.memory_mb = memory_mb;
        self
    }

    pub fn with_vcpus(mut self, vcpu_count: u32) -> Self {
        self.vcpu_count = vcpu_count;
        self
    }

    pub fn with_nested_virt(mut self, enable: bool) -> Self {
        self.enable_nested_virt = enable;
        self
    }
}

/// Máquina Virtual KVM
pub struct KvmVirtualMachine {
    /// Configuración de la VM
    pub config: KvmVmConfig,
    /// Estado actual
    pub state: KvmState,
    /// Capability de esta VM
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// PID del proceso QEMU
    pub qemu_pid: Option<u32>,
    /// Uso de recursos
    pub resource_usage: KvmResourceUsage,
}

/// Uso de recursos de la VM
#[derive(Debug, Clone)]
pub struct KvmResourceUsage {
    /// Uso de CPU (%)
    pub cpu_usage: f32,
    /// Uso de memoria (MB)
    pub memory_usage_mb: u32,
    /// Uso de disco (GB)
    pub disk_usage_gb: u32,
    /// Uso de red (KB/s)
    pub network_usage_kbps: u32,
}

impl Default for KvmResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage_mb: 0,
            disk_usage_gb: 0,
            network_usage_kbps: 0,
        }
    }
}

impl KvmVirtualMachine {
    pub fn new(config: KvmVmConfig) -> Self {
        Self {
            config,
            state: KvmState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            qemu_pid: None,
            resource_usage: KvmResourceUsage::default(),
        }
    }

    /// Inicializar la VM en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != KvmState::Uninitialized {
            return Err(format!("VM ya inicializada, estado actual: {:?}", self.state));
        }

        // Crear nodo en el graph kernel para esta VM
        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("kvm_vm_{}", self.config.vm_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = KvmState::Idle;
        Ok(())
    }

    /// Iniciar la VM
    pub fn start(&mut self) -> Result<(), String> {
        if self.state != KvmState::Idle {
            return Err(format!("VM no está en estado Idle, estado actual: {:?}", self.state));
        }

        // Generar comando QEMU adaptado a CRONOS
        let qemu_command = self.generate_qemu_command();

        // En un sistema real, aquí se ejecutaría el comando QEMU
        // Por ahora, simulamos el inicio
        self.state = KvmState::Active;
        self.qemu_pid = Some(12345); // PID simulado

        Ok(())
    }

    /// Detener la VM
    pub fn stop(&mut self) -> Result<(), String> {
        if self.state != KvmState::Active {
            return Err(format!("VM no está en estado Active, estado actual: {:?}", self.state));
        }

        // En un sistema real, aquí se enviaría señal SIGTERM al proceso QEMU
        self.state = KvmState::Idle;
        self.qemu_pid = None;

        Ok(())
    }

    /// Generar comando QEMU adaptado
    fn generate_qemu_command(&self) -> String {
        let mut command = String::from("qemu-system-x86_64");

        // Memoria
        command.push_str(&format!(" -m {}", self.config.memory_mb));

        // vCPUs
        command.push_str(&format!(" -smp {}", self.config.vcpu_count));

        // KVM
        command.push_str(" -enable-kvm");

        // Disco
        command.push_str(&format!(" -drive file={},format=qcow2", self.config.disk_path));

        // Virtualización anidada
        if self.config.enable_nested_virt {
            command.push_str(" -cpu host,+vmx");
        }

        // Aceleración de hardware
        if self.config.enable_hardware_accel {
            command.push_str(" -machine type=q35,accel=kvm");
        } else {
            command.push_str(" -machine type=q35");
        }

        // Nombre de la VM
        command.push_str(&format!(" -name {}", self.config.name));

        command
    }

    /// Actualizar métricas de uso de recursos
    pub fn update_resource_usage(&mut self) {
        // En un sistema real, aquí se leerían las métricas del proceso QEMU
        if self.state == KvmState::Active {
            self.resource_usage.cpu_usage = 35.0;
            self.resource_usage.memory_usage_mb = self.config.memory_mb / 2;
            self.resource_usage.disk_usage_gb = self.config.disk_size_gb / 4;
            self.resource_usage.network_usage_kbps = 50;
        } else {
            self.resource_usage = KvmResourceUsage::default();
        }
    }

    /// Verificar si la VM está activa
    pub fn is_active(&self) -> bool {
        self.state == KvmState::Active
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &KvmState {
        &self.state
    }
}

/// Integración KVM para CRONOS W-OS
pub struct CronosKvmIntegration {
    /// VMs registradas (keyed by vm_id)
    pub vms: BTreeMap<u64, KvmVirtualMachine>,
    /// Estado del módulo KVM
    pub state: KvmState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del módulo KVM
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de VM
    pub next_vm_id: u64,
}

impl CronosKvmIntegration {
    pub fn new() -> Self {
        Self {
            vms: BTreeMap::new(),
            state: KvmState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_vm_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = KvmState::Idle;
    }

    /// Crear una nueva VM
    pub fn create_vm(&mut self, config: KvmVmConfig) -> Result<u64, String> {
        if self.state == KvmState::Uninitialized {
            return Err(String::from("KVM no inicializado. Llamar a set_graph_kernel primero."));
        }

        let vm_id = config.vm_id;
        let mut vm = KvmVirtualMachine::new(config);

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
    pub fn create_default_vm(&mut self, name: String, disk_path: String) -> Result<u64, String> {
        let vm_id = self.next_vm_id;
        let config = KvmVmConfig::new(vm_id, name, disk_path);
        self.create_vm(config)
    }

    /// Obtener una VM por ID
    pub fn get_vm(&self, vm_id: u64) -> Option<&KvmVirtualMachine> {
        self.vms.get(&vm_id)
    }

    /// Obtener una VM mutable por ID
    pub fn get_vm_mut(&mut self, vm_id: u64) -> Option<&mut KvmVirtualMachine> {
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

    /// Obtener número de VMs activas
    pub fn active_vm_count(&self) -> usize {
        self.vms.values().filter(|vm| vm.is_active()).count()
    }

    /// Listar todas las VMs
    pub fn list_vms(&self) -> Vec<&KvmVirtualMachine> {
        self.vms.values().collect()
    }

    /// Verificar si KVM está habilitado en el hardware
    pub fn is_kvm_supported(&self) -> bool {
        // En un sistema real, esto verificaría /dev/kvm o CPUID
        // Por ahora, asumimos que está soportado
        true
    }

    /// Obtener el estado del módulo KVM
    pub fn state(&self) -> &KvmState {
        &self.state
    }
}

impl Default for CronosKvmIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración KVM
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvmIntegrationError {
    VmNotFound,
    VmAlreadyRunning,
    VmNotRunning,
    InvalidConfig,
    KvmNotSupported,
    DiskImageNotFound,
    GraphKernelNotInitialized,
}

impl fmt::Display for KvmIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvmIntegrationError::VmNotFound => write!(f, "VM not found"),
            KvmIntegrationError::VmAlreadyRunning => write!(f, "VM is already running"),
            KvmIntegrationError::VmNotRunning => write!(f, "VM is not running"),
            KvmIntegrationError::InvalidConfig => write!(f, "Invalid configuration"),
            KvmIntegrationError::KvmNotSupported => write!(f, "KVM not supported on this hardware"),
            KvmIntegrationError::DiskImageNotFound => write!(f, "Disk image not found"),
            KvmIntegrationError::GraphKernelNotInitialized => write!(f, "Graph kernel not initialized"),
        }
    }
}
