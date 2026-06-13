//! VirtIO Drivers Integration para CRONOS W-OS
//!
//! Este módulo adapta los drivers VirtIO de Redox OS a CRONOS W-OS,
//! integrando con el graph kernel y capabilities para virtualización

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del módulo VirtIO
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtIOState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Activo
    Active,
    /// Error
    Error(String),
}

/// Tipo de dispositivo VirtIO
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIODeviceType {
    /// Dispositivo de red
    Network,
    /// Dispositivo de bloque (disco)
    Block,
    /// Dispositivo de consola
    Console,
    /// Dispositivo de entrada (teclado/mouse)
    Input,
    /// Dispositivo GPU
    GPU,
    /// Dispositivo de memoria baloon
    Balloon,
}

/// Configuración de dispositivo VirtIO
#[derive(Debug, Clone)]
pub struct VirtIODeviceConfig {
    /// ID único del dispositivo
    pub device_id: u64,
    /// Tipo de dispositivo
    pub device_type: VirtIODeviceType,
    /// Nombre del dispositivo
    pub name: String,
    /// IRQ del dispositivo
    pub irq: u32,
    /// Número de colas virtqueues
    pub num_queues: u32,
    /// Habilitar MSI-X
    pub enable_msix: bool,
}

impl VirtIODeviceConfig {
    pub fn new(device_id: u64, device_type: VirtIODeviceType, name: String) -> Self {
        Self {
            device_id,
            device_type,
            name,
            irq: 0,
            num_queues: 2,
            enable_msix: true,
        }
    }

    pub fn with_irq(mut self, irq: u32) -> Self {
        self.irq = irq;
        self
    }

    pub fn with_queues(mut self, num_queues: u32) -> Self {
        self.num_queues = num_queues;
        self
    }
}

/// Dispositivo VirtIO
pub struct VirtIODevice {
    /// Configuración del dispositivo
    pub config: VirtIODeviceConfig,
    /// Estado actual
    pub state: VirtIOState,
    /// Capability de este dispositivo
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Dirección MMIO del dispositivo
    pub mmio_address: Option<u64>,
    /// Tamaño del MMIO
    pub mmio_size: u32,
    /// Métricas del dispositivo
    pub metrics: VirtIODeviceMetrics,
}

/// Métricas del dispositivo VirtIO
#[derive(Debug, Clone)]
pub struct VirtIODeviceMetrics {
    /// Bytes transferidos
    pub bytes_transferred: u64,
    /// Operaciones completadas
    pub operations_completed: u64,
    /// Interrupciones procesadas
    pub interrupts_processed: u64,
    /// Errores
    pub errors: u64,
}

impl Default for VirtIODeviceMetrics {
    fn default() -> Self {
        Self {
            bytes_transferred: 0,
            operations_completed: 0,
            interrupts_processed: 0,
            errors: 0,
        }
    }
}

impl VirtIODevice {
    pub fn new(config: VirtIODeviceConfig) -> Self {
        Self {
            config,
            state: VirtIOState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            mmio_address: None,
            mmio_size: 0,
            metrics: VirtIODeviceMetrics::default(),
        }
    }

    /// Inicializar el dispositivo en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != VirtIOState::Uninitialized {
            return Err(format!("Dispositivo ya inicializado, estado actual: {:?}", self.state));
        }

        // Crear nodo en el graph kernel para este dispositivo
        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("virtio_{}_{}", self.config.device_type as u8, self.config.device_id),
        );
        self.graph_node_id = Some(node_id);

        // Asignar dirección MMIO simulada
        self.mmio_address = Some(0xFE000000 + (self.config.device_id * 0x1000));
        self.mmio_size = 0x1000;

        self.state = VirtIOState::Initialized;
        Ok(())
    }

    /// Activar el dispositivo
    pub fn activate(&mut self) -> Result<(), String> {
        if self.state != VirtIOState::Initialized {
            return Err(format!("Dispositivo no está en estado Initialized, estado actual: {:?}", self.state));
        }

        // En un sistema real, aquí se inicializarían los virtqueues
        // y se configuraría el dispositivo VirtIO
        self.state = VirtIOState::Active;
        Ok(())
    }

    /// Desactivar el dispositivo
    pub fn deactivate(&mut self) -> Result<(), String> {
        if self.state != VirtIOState::Active {
            return Err(format!("Dispositivo no está en estado Active, estado actual: {:?}", self.state));
        }

        self.state = VirtIOState::Initialized;
        Ok(())
    }

    /// Leer del dispositivo (para block/console)
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, String> {
        if self.state != VirtIOState::Active {
            return Err(format!("Dispositivo no está activo, estado actual: {:?}", self.state));
        }

        // En un sistema real, esto usaría el virtqueue para leer
        let bytes_read = buffer.len().min(512);
        self.metrics.bytes_transferred += bytes_read as u64;
        self.metrics.operations_completed += 1;

        Ok(bytes_read)
    }

    /// Escribir al dispositivo (para block/console)
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, String> {
        if self.state != VirtIOState::Active {
            return Err(format!("Dispositivo no está activo, estado actual: {:?}", self.state));
        }

        // En un sistema real, esto usaría el virtqueue para escribir
        let bytes_written = buffer.len();
        self.metrics.bytes_transferred += bytes_written as u64;
        self.metrics.operations_completed += 1;

        Ok(bytes_written)
    }

    /// Procesar interrupción del dispositivo
    pub fn handle_interrupt(&mut self) -> Result<(), String> {
        if self.state != VirtIOState::Active {
            return Err(format!("Dispositivo no está activo, estado actual: {:?}", self.state));
        }

        // En un sistema real, esto procesaría la interrupción VirtIO
        self.metrics.interrupts_processed += 1;

        Ok(())
    }

    /// Verificar si el dispositivo está activo
    pub fn is_active(&self) -> bool {
        self.state == VirtIOState::Active
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &VirtIOState {
        &self.state
    }

    /// Actualizar métricas
    pub fn update_metrics(&mut self) {
        if self.state == VirtIOState::Active {
            // Simular actividad del dispositivo
            self.metrics.bytes_transferred += 1024;
            self.metrics.operations_completed += 1;
        }
    }
}

/// Integración VirtIO Drivers para CRONOS W-OS
pub struct CronosVirtIOIntegration {
    /// Dispositivos registrados (keyed by device_id)
    pub devices: BTreeMap<u64, VirtIODevice>,
    /// Estado del módulo VirtIO
    pub state: VirtIOState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del módulo VirtIO
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de dispositivo
    pub next_device_id: u64,
}

impl CronosVirtIOIntegration {
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
            state: VirtIOState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_device_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = VirtIOState::Initialized;
    }

    /// Crear un nuevo dispositivo VirtIO
    pub fn create_device(&mut self, config: VirtIODeviceConfig) -> Result<u64, String> {
        if self.state == VirtIOState::Uninitialized {
            return Err(String::from("VirtIO no inicializado. Llamar a set_graph_kernel primero."));
        }

        let device_id = config.device_id;
        let mut device = VirtIODevice::new(config);

        // Inicializar el dispositivo en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                device.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.devices.insert(device_id, device);
        self.next_device_id = device_id + 1;

        Ok(device_id)
    }

    /// Crear un dispositivo con configuración predeterminada
    pub fn create_default_device(&mut self, device_type: VirtIODeviceType, name: String) -> Result<u64, String> {
        let device_id = self.next_device_id;
        let config = VirtIODeviceConfig::new(device_id, device_type, name);
        self.create_device(config)
    }

    /// Obtener un dispositivo por ID
    pub fn get_device(&self, device_id: u64) -> Option<&VirtIODevice> {
        self.devices.get(&device_id)
    }

    /// Obtener un dispositivo mutable por ID
    pub fn get_device_mut(&mut self, device_id: u64) -> Option<&mut VirtIODevice> {
        self.devices.get_mut(&device_id)
    }

    /// Activar un dispositivo
    pub fn activate_device(&mut self, device_id: u64) -> Result<(), String> {
        if let Some(device) = self.get_device_mut(device_id) {
            device.activate()
        } else {
            Err(format!("Dispositivo con ID {} no encontrado", device_id))
        }
    }

    /// Desactivar un dispositivo
    pub fn deactivate_device(&mut self, device_id: u64) -> Result<(), String> {
        if let Some(device) = self.get_device_mut(device_id) {
            device.deactivate()
        } else {
            Err(format!("Dispositivo con ID {} no encontrado", device_id))
        }
    }

    /// Leer de un dispositivo
    pub fn read_device(&mut self, device_id: u64, buffer: &mut [u8]) -> Result<usize, String> {
        if let Some(device) = self.get_device_mut(device_id) {
            device.read(buffer)
        } else {
            Err(format!("Dispositivo con ID {} no encontrado", device_id))
        }
    }

    /// Escribir a un dispositivo
    pub fn write_device(&mut self, device_id: u64, buffer: &[u8]) -> Result<usize, String> {
        if let Some(device) = self.get_device_mut(device_id) {
            device.write(buffer)
        } else {
            Err(format!("Dispositivo con ID {} no encontrado", device_id))
        }
    }

    /// Procesar interrupción de un dispositivo
    pub fn handle_interrupt(&mut self, device_id: u64) -> Result<(), String> {
        if let Some(device) = self.get_device_mut(device_id) {
            device.handle_interrupt()
        } else {
            Err(format!("Dispositivo con ID {} no encontrado", device_id))
        }
    }

    /// Actualizar métricas de todos los dispositivos
    pub fn update_all_metrics(&mut self) {
        for device in self.devices.values_mut() {
            device.update_metrics();
        }
    }

    /// Obtener número de dispositivos
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Obtener número de dispositivos activos
    pub fn active_device_count(&self) -> usize {
        self.devices.values().filter(|d| d.is_active()).count()
    }

    /// Listar todos los dispositivos
    pub fn list_devices(&self) -> Vec<&VirtIODevice> {
        self.devices.values().collect()
    }

    /// Obtener dispositivos por tipo
    pub fn get_devices_by_type(&self, device_type: VirtIODeviceType) -> Vec<&VirtIODevice> {
        self.devices.values()
            .filter(|d| d.config.device_type == device_type)
            .collect()
    }

    /// Verificar si VirtIO está soportado
    pub fn is_virtio_supported(&self) -> bool {
        // En un sistema real, esto verificaría si el hardware soporta VirtIO
        // Por ahora, asumimos que está soportado
        true
    }

    /// Obtener el estado del módulo VirtIO
    pub fn state(&self) -> &VirtIOState {
        &self.state
    }

    /// Obtener métricas agregadas
    pub fn get_aggregated_metrics(&self) -> VirtIODeviceMetrics {
        let mut total = VirtIODeviceMetrics::default();
        for device in self.devices.values() {
            total.bytes_transferred += device.metrics.bytes_transferred;
            total.operations_completed += device.metrics.operations_completed;
            total.interrupts_processed += device.metrics.interrupts_processed;
            total.errors += device.metrics.errors;
        }
        total
    }
}

impl Default for CronosVirtIOIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración VirtIO
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtIOIntegrationError {
    DeviceNotFound,
    DeviceAlreadyActive,
    DeviceNotActive,
    InvalidConfig,
    VirtIONotSupported,
    MMIOAllocationFailed,
    QueueInitializationFailed,
}

impl fmt::Display for VirtIOIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtIOIntegrationError::DeviceNotFound => write!(f, "Device not found"),
            VirtIOIntegrationError::DeviceAlreadyActive => write!(f, "Device is already active"),
            VirtIOIntegrationError::DeviceNotActive => write!(f, "Device is not active"),
            VirtIOIntegrationError::InvalidConfig => write!(f, "Invalid configuration"),
            VirtIOIntegrationError::VirtIONotSupported => write!(f, "VirtIO not supported on this hardware"),
            VirtIOIntegrationError::MMIOAllocationFailed => write!(f, "MMIO allocation failed"),
            VirtIOIntegrationError::QueueInitializationFailed => write!(f, "Queue initialization failed"),
        }
    }
}
