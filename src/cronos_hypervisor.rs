//! Hypervisor de CRONOS original adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el hypervisor de CRONOS original,
//! adaptado al sistema de capabilities y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::BTreeMap;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Estado de una máquina virtual
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmState {
    Stopped,
    Running,
    Paused,
    Crashed,
}

/// Tipo de virtualización
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualizationType {
    FullVirtualization,
    Paravirtualization,
    HardwareAssisted,
}

/// Configuración de CPU para VM
#[derive(Debug, Clone)]
pub struct VmCpuConfig {
    pub vcpus: u32,
    pub cpu_quota: u32,
    pub cpu_weight: u32,
}

impl VmCpuConfig {
    pub fn new(vcpus: u32) -> Self {
        Self {
            vcpus,
            cpu_quota: 100,
            cpu_weight: 1024,
        }
    }
}

impl Default for VmCpuConfig {
    fn default() -> Self {
        Self::new(1)
    }
}

/// Configuración de memoria para VM
#[derive(Debug, Clone)]
pub struct VmMemoryConfig {
    pub memory_mb: u64,
    pub max_memory_mb: u64,
    pub hugepages_enabled: bool,
}

impl VmMemoryConfig {
    pub fn new(memory_mb: u64) -> Self {
        Self {
            memory_mb,
            max_memory_mb: memory_mb,
            hugepages_enabled: false,
        }
    }
}

impl Default for VmMemoryConfig {
    fn default() -> Self {
        Self::new(512)
    }
}

/// Configuración de red para VM
#[derive(Debug, Clone)]
pub struct VmNetworkConfig {
    pub bridge: String,
    pub mac_address: [u8; 6],
    pub ip_address: Option<String>,
    pub bandwidth_limit: Option<u64>,
}

impl VmNetworkConfig {
    pub fn new(bridge: &str) -> Self {
        Self {
            bridge: String::from(bridge),
            mac_address: [0; 6],
            ip_address: None,
            bandwidth_limit: None,
        }
    }
}

impl Default for VmNetworkConfig {
    fn default() -> Self {
        Self::new("br0")
    }
}

/// Configuración de almacenamiento para VM
#[derive(Debug, Clone)]
pub struct VmStorageConfig {
    pub disk_image: String,
    pub disk_size_gb: u64,
    pub disk_type: String,
    pub readonly: bool,
}

impl VmStorageConfig {
    pub fn new(disk_image: &str, disk_size_gb: u64) -> Self {
        Self {
            disk_image: String::from(disk_image),
            disk_size_gb,
            disk_type: String::from("qcow2"),
            readonly: false,
        }
    }
}

/// Máquina virtual
#[derive(Debug, Clone)]
pub struct VirtualMachine {
    pub vm_id: u64,
    pub name: String,
    pub state: VmState,
    pub virtualization_type: VirtualizationType,
    pub cpu_config: VmCpuConfig,
    pub memory_config: VmMemoryConfig,
    pub network_config: Option<VmNetworkConfig>,
    pub storage_config: Option<VmStorageConfig>,
    pub creation_time: u64,
    pub uptime: u64,
    pub capability_id: Option<CapabilityId>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl VirtualMachine {
    pub fn new(vm_id: u64, name: &str, virtualization_type: VirtualizationType) -> Self {
        Self {
            vm_id,
            name: String::from(name),
            state: VmState::Stopped,
            virtualization_type,
            cpu_config: VmCpuConfig::default(),
            memory_config: VmMemoryConfig::default(),
            network_config: None,
            storage_config: None,
            creation_time: 0,
            uptime: 0,
            capability_id: None,
            graph_node_id: None,
        }
    }

    pub fn set_cpu_config(&mut self, config: VmCpuConfig) {
        self.cpu_config = config;
    }

    pub fn set_memory_config(&mut self, config: VmMemoryConfig) {
        self.memory_config = config;
    }

    pub fn set_network_config(&mut self, config: VmNetworkConfig) {
        self.network_config = Some(config);
    }

    pub fn set_storage_config(&mut self, config: VmStorageConfig) {
        self.storage_config = Some(config);
    }

    pub fn is_running(&self) -> bool {
        self.state == VmState::Running
    }

    pub fn is_stopped(&self) -> bool {
        self.state == VmState::Stopped
    }

    pub fn is_paused(&self) -> bool {
        self.state == VmState::Paused
    }

    pub fn is_crashed(&self) -> bool {
        self.state == VmState::Crashed
    }
}

/// Hypervisor
pub struct CronosHypervisor {
    pub virtual_machines: BTreeMap<u64, VirtualMachine>,
    pub next_vm_id: u64,
    pub enabled: bool,
    pub hardware_virtualization_supported: bool,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosHypervisor {
    pub fn new() -> Self {
        Self {
            virtual_machines: BTreeMap::new(),
            next_vm_id: 1,
            enabled: true,
            hardware_virtualization_supported: false,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_hardware_virtualization_supported(&self) -> bool {
        self.hardware_virtualization_supported
    }

    pub fn set_hardware_virtualization_supported(&mut self, supported: bool) {
        self.hardware_virtualization_supported = supported;
    }

    pub fn create_vm(&mut self, name: &str, virtualization_type: VirtualizationType) -> Result<u64, String> {
        if !self.enabled {
            return Err(String::from("Hypervisor is disabled"));
        }

        if virtualization_type == VirtualizationType::HardwareAssisted && !self.hardware_virtualization_supported {
            return Err(String::from("Hardware virtualization not supported"));
        }

        let vm_id = self.next_vm_id;
        self.next_vm_id += 1;

        let mut vm = VirtualMachine::new(vm_id, name, virtualization_type);

        // Crear capability para la VM
        let capability_id = CapabilityId::new();
        vm.capability_id = Some(capability_id);

        // Registrar la VM como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("vm_{}", vm_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            vm.graph_node_id = node_id;
        }

        self.virtual_machines.insert(vm_id, vm);
        Ok(vm_id)
    }

    pub fn destroy_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if let Some(vm) = self.get_vm(vm_id) {
            if vm.is_running() {
                return Err(String::from("Virtual machine is running"));
            }
        }

        if self.virtual_machines.remove(&vm_id).is_some() {
            Ok(())
        } else {
            Err(format!("Virtual machine {} not found", vm_id))
        }
    }

    pub fn get_vm(&self, vm_id: u64) -> Option<&VirtualMachine> {
        self.virtual_machines.get(&vm_id)
    }

    pub fn get_vm_mut(&mut self, vm_id: u64) -> Option<&mut VirtualMachine> {
        self.virtual_machines.get_mut(&vm_id)
    }

    pub fn list_vms(&self) -> Vec<&VirtualMachine> {
        self.virtual_machines.values().collect()
    }

    pub fn list_running_vms(&self) -> Vec<&VirtualMachine> {
        self.virtual_machines.values().filter(|vm| vm.is_running()).collect()
    }

    pub fn list_stopped_vms(&self) -> Vec<&VirtualMachine> {
        self.virtual_machines.values().filter(|vm| vm.is_stopped()).collect()
    }

    pub fn vm_count(&self) -> usize {
        self.virtual_machines.len()
    }

    pub fn running_vm_count(&self) -> usize {
        self.virtual_machines.values().filter(|vm| vm.is_running()).count()
    }

    pub fn start_vm(&mut self, vm_id: u64) -> Result<(), String> {
        if !self.enabled {
            return Err(String::from("Hypervisor is disabled"));
        }

        let vm = self.get_vm_mut(vm_id).ok_or(format!("Virtual machine {} not found", vm_id))?;
        
        if vm.is_running() {
            return Err(String::from("Virtual machine is already running"));
        }

        if vm.is_crashed() {
            return Err(String::from("Virtual machine has crashed"));
        }

        // Verificar que la VM tenga configuración mínima
        if vm.storage_config.is_none() {
            return Err(String::from("Virtual machine is not configured"));
        }

        vm.state = VmState::Running;
        vm.creation_time = 0; // En un sistema real, esto sería el timestamp actual
        
        Ok(())
    }

    pub fn stop_vm(&mut self, vm_id: u64) -> Result<(), String> {
        let vm = self.get_vm_mut(vm_id).ok_or(format!("Virtual machine {} not found", vm_id))?;
        
        if !vm.is_running() {
            return Err(String::from("Virtual machine is not running"));
        }

        vm.state = VmState::Stopped;
        
        Ok(())
    }

    pub fn pause_vm(&mut self, vm_id: u64) -> Result<(), String> {
        let vm = self.get_vm_mut(vm_id).ok_or(format!("Virtual machine {} not found", vm_id))?;
        
        if !vm.is_running() {
            return Err(String::from("Virtual machine is not running"));
        }

        vm.state = VmState::Paused;
        
        Ok(())
    }

    pub fn resume_vm(&mut self, vm_id: u64) -> Result<(), String> {
        let vm = self.get_vm_mut(vm_id).ok_or(format!("Virtual machine {} not found", vm_id))?;
        
        if !vm.is_paused() {
            return Err(String::from("Virtual machine is not paused"));
        }

        vm.state = VmState::Running;
        
        Ok(())
    }

    pub fn reboot_vm(&mut self, vm_id: u64) -> Result<(), String> {
        let vm = self.get_vm_mut(vm_id).ok_or(format!("Virtual machine {} not found", vm_id))?;
        
        if !vm.is_running() && !vm.is_paused() {
            return Err(String::from("Virtual machine is not running"));
        }

        vm.state = VmState::Stopped;
        vm.state = VmState::Running;
        
        Ok(())
    }

    pub fn get_vm_stats(&self, vm_id: u64) -> Result<VmStats, String> {
        let vm = self.get_vm(vm_id).ok_or(format!("Virtual machine {} not found", vm_id))?;
        
        Ok(VmStats {
            vm_id: vm.vm_id,
            state: vm.state.clone(),
            cpu_usage: 0,
            memory_usage: 0,
            disk_usage: 0,
            network_io: 0,
            uptime: vm.uptime,
        })
    }
}

impl Default for CronosHypervisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de VM
#[derive(Debug, Clone)]
pub struct VmStats {
    pub vm_id: u64,
    pub state: VmState,
    pub cpu_usage: u32,
    pub memory_usage: u64,
    pub disk_usage: u64,
    pub network_io: u64,
    pub uptime: u64,
}

/// Errores del hypervisor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypervisorError {
    HypervisorDisabled,
    HardwareVirtualizationNotSupported,
    VmNotFound,
    VmAlreadyRunning,
    VmNotRunning,
    VmNotPaused,
    VmRunning,
    VmCrashed,
    VmNotConfigured,
    InsufficientResources,
    InvalidConfiguration,
}

impl fmt::Display for HypervisorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HypervisorError::HypervisorDisabled => write!(f, "Hypervisor is disabled"),
            HypervisorError::HardwareVirtualizationNotSupported => write!(f, "Hardware virtualization not supported"),
            HypervisorError::VmNotFound => write!(f, "Virtual machine not found"),
            HypervisorError::VmAlreadyRunning => write!(f, "Virtual machine is already running"),
            HypervisorError::VmNotRunning => write!(f, "Virtual machine is not running"),
            HypervisorError::VmNotPaused => write!(f, "Virtual machine is not paused"),
            HypervisorError::VmRunning => write!(f, "Virtual machine is running"),
            HypervisorError::VmCrashed => write!(f, "Virtual machine has crashed"),
            HypervisorError::VmNotConfigured => write!(f, "Virtual machine is not configured"),
            HypervisorError::InsufficientResources => write!(f, "Insufficient resources"),
            HypervisorError::InvalidConfiguration => write!(f, "Invalid configuration"),
        }
    }
}
