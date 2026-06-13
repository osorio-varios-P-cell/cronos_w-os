//! Drivers de Disco para CRONOS W-OS
//!
//! Este módulo implementa drivers de disco para AHCI (SATA) y NVMe,
//! adaptados a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId};

/// Tipo de dispositivo de almacenamiento
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageDeviceType {
    /// Disco SATA (AHCI)
    AhciSata,
    /// SSD NVMe
    Nvme,
    /// Disco IDE (Legacy)
    Ide,
    /// Disco desconocido
    Unknown,
}

/// Estado del dispositivo de almacenamiento
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageDeviceState {
    /// No inicializado
    Uninitialized,
    /// Inicializando
    Initializing,
    /// Listo
    Ready,
    /// Ocupado
    Busy,
    /// Error
    Error(String),
}

/// Configuración de AHCI
#[derive(Debug, Clone)]
pub struct AhciConfig {
    /// Dirección base del registro ABAR
    pub abar_address: u64,
    /// Número de puertos
    pub port_count: u32,
    /// Habilitar NCQ
    pub enable_ncq: bool,
}

/// Puerto AHCI
#[derive(Debug, Clone)]
pub struct AhciPort {
    /// Número de puerto
    pub port_number: u32,
    /// Estado del puerto
    pub state: StorageDeviceState,
    /// Tipo de dispositivo conectado
    pub device_type: StorageDeviceType,
    /// Capacidad en sectores
    pub capacity_sectors: u64,
    /// Tamaño de sector en bytes
    pub sector_size: u32,
}

impl AhciPort {
    pub fn new(port_number: u32) -> Self {
        Self {
            port_number,
            state: StorageDeviceState::Uninitialized,
            device_type: StorageDeviceType::Unknown,
            capacity_sectors: 0,
            sector_size: 512,
        }
    }
}

/// Controlador AHCI
pub struct AhciController {
    /// Configuración AHCI
    pub config: AhciConfig,
    /// Puertos AHCI
    pub ports: Vec<AhciPort>,
    /// Habilitado
    pub enabled: bool,
}

impl AhciController {
    pub fn new(config: AhciConfig) -> Self {
        let port_count = config.port_count as usize;
        let mut ports = Vec::with_capacity(port_count);
        
        for i in 0..port_count {
            ports.push(AhciPort::new(i as u32));
        }

        Self {
            config,
            ports,
            enabled: false,
        }
    }

    /// Inicializar el controlador AHCI
    pub fn initialize(&mut self) -> Result<(), String> {
        self.enabled = true;
        
        // En un sistema real, aquí se:
        // 1. Habilitaría el controlador AHCI en el PCI
        // 2. Esperaría a que el BIOS termine
        // 3. Habilitaría las interrupciones
        // 4. Escanearía los puertos para detectar dispositivos
        
        for port in &mut self.ports {
            port.state = StorageDeviceState::Ready;
            port.device_type = StorageDeviceType::AhciSata;
            port.capacity_sectors = 1024 * 1024 * 1024; // 512GB simulado
        }

        Ok(())
    }

    /// Leer sectores de un puerto
    pub fn read_sectors(&mut self, port: u32, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), String> {
        if !self.enabled {
            return Err(String::from("AHCI controller not enabled"));
        }

        let port_index = port as usize;
        if port_index >= self.ports.len() {
            return Err(format!("Invalid port number: {}", port));
        }

        let ahci_port = &mut self.ports[port_index];
        if ahci_port.state != StorageDeviceState::Ready {
            return Err(format!("Port {} not ready", port));
        }

        // En un sistema real, aquí se:
        // 1. Prepararía el comando FIS
        // 2. Configuraría el PRDT (Physical Region Descriptor Table)
        // 3. Iniciaría el comando
        // 4. Esperaría a que termine
        // 5. Copiaría los datos al buffer

        // Simulación: llenar buffer con ceros
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        Ok(())
    }

    /// Escribir sectores a un puerto
    pub fn write_sectors(&mut self, port: u32, lba: u64, count: u16, buffer: &[u8]) -> Result<(), String> {
        if !self.enabled {
            return Err(String::from("AHCI controller not enabled"));
        }

        let port_index = port as usize;
        if port_index >= self.ports.len() {
            return Err(format!("Invalid port number: {}", port));
        }

        let ahci_port = &mut self.ports[port_index];
        if ahci_port.state != StorageDeviceState::Ready {
            return Err(format!("Port {} not ready", port));
        }

        // En un sistema real, aquí se escribirían los datos al disco

        Ok(())
    }

    /// Obtener un puerto por número
    pub fn get_port(&self, port_number: u32) -> Option<&AhciPort> {
        self.ports.get(port_number as usize)
    }

    /// Obtener número de puertos
    pub fn port_count(&self) -> usize {
        self.ports.len()
    }
}

/// Configuración de NVMe
#[derive(Debug, Clone)]
pub struct NvmeConfig {
    /// Dirección base de los registros MMIO
    pub mmio_address: u64,
    /// Tamaño de los registros MMIO
    pub mmio_size: u32,
    /// Número de colas
    pub queue_count: u32,
}

/// Cola NVMe
#[derive(Debug, Clone)]
pub struct NvmeQueue {
    /// ID de la cola
    pub queue_id: u16,
    /// Tamaño de la cola
    pub queue_size: u16,
    /// Habilitada
    pub enabled: bool,
}

impl NvmeQueue {
    pub fn new(queue_id: u16, queue_size: u16) -> Self {
        Self {
            queue_id,
            queue_size,
            enabled: false,
        }
    }
}

/// Controlador NVMe
pub struct NvmeController {
    /// Configuración NVMe
    pub config: NvmeConfig,
    /// Colas de I/O
    pub queues: Vec<NvmeQueue>,
    /// Estado del controlador
    pub state: StorageDeviceState,
    /// Capacidad en sectores
    pub capacity_sectors: u64,
    /// Tamaño de sector en bytes
    pub sector_size: u32,
}

impl NvmeController {
    pub fn new(config: NvmeConfig) -> Self {
        let queue_count = config.queue_count as usize;
        let mut queues = Vec::with_capacity(queue_count);
        
        for i in 0..queue_count {
            queues.push(NvmeQueue::new(i as u16, 64));
        }

        Self {
            config,
            queues,
            state: StorageDeviceState::Uninitialized,
            capacity_sectors: 0,
            sector_size: 512,
        }
    }

    /// Inicializar el controlador NVMe
    pub fn initialize(&mut self) -> Result<(), String> {
        self.state = StorageDeviceState::Initializing;

        // En un sistema real, aquí se:
        // 1. Habilitaría el controlador NVMe en el PCI
        // 2. Leería los registros del controlador
        // 3. Configuraría las colas de administración e I/O
        // 4. Identificaría el namespace
        // 5. Obtendría la capacidad del dispositivo

        self.state = StorageDeviceState::Ready;
        self.capacity_sectors = 1024 * 1024 * 1024; // 512GB simulado

        for queue in &mut self.queues {
            queue.enabled = true;
        }

        Ok(())
    }

    /// Leer sectores del dispositivo NVMe
    pub fn read_sectors(&mut self, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), String> {
        if self.state != StorageDeviceState::Ready {
            return Err(format!("NVMe controller not ready, state: {:?}", self.state));
        }

        // En un sistema real, aquí se:
        // 1. Prepararía el comando NVMe
        // 2. Lo enviaría a la cola de I/O
        // 3. Esperaría a que termine
        // 4. Copiaría los datos al buffer

        // Simulación: llenar buffer con ceros
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        Ok(())
    }

    /// Escribir sectores al dispositivo NVMe
    pub fn write_sectors(&mut self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), String> {
        if self.state != StorageDeviceState::Ready {
            return Err(format!("NVMe controller not ready, state: {:?}", self.state));
        }

        // En un sistema real, aquí se escribirían los datos al dispositivo

        Ok(())
    }

    /// Obtener una cola por ID
    pub fn get_queue(&self, queue_id: u16) -> Option<&NvmeQueue> {
        self.queues.get(queue_id as usize)
    }

    /// Obtener número de colas
    pub fn queue_count(&self) -> usize {
        self.queues.len()
    }
}

/// Dispositivo de almacenamiento
pub struct StorageDevice {
    /// ID del dispositivo
    pub id: u64,
    /// Tipo de dispositivo
    pub device_type: StorageDeviceType,
    /// Estado del dispositivo
    pub state: StorageDeviceState,
    /// Controlador AHCI (si aplica)
    pub ahci_controller: Option<AhciController>,
    /// Controlador NVMe (si aplica)
    pub nvme_controller: Option<NvmeController>,
    /// Capacidad en bytes
    pub capacity_bytes: u64,
    /// Tamaño de sector en bytes
    pub sector_size: u32,
    /// ID del nodo en el grafo que representa este dispositivo
    pub graph_node_id: Option<NodeId>,
}

impl StorageDevice {
    pub fn new(id: u64, device_type: StorageDeviceType) -> Self {
        Self {
            id,
            device_type,
            state: StorageDeviceState::Uninitialized,
            ahci_controller: None,
            nvme_controller: None,
            capacity_bytes: 0,
            sector_size: 512,
            graph_node_id: None,
        }
    }

    /// Inicializar el dispositivo
    pub fn initialize(&mut self) -> Result<(), String> {
        self.state = StorageDeviceState::Initializing;

        match self.device_type {
            StorageDeviceType::AhciSata => {
                if let Some(ref mut controller) = self.ahci_controller {
                    controller.initialize()?;
                    self.capacity_bytes = controller.ports.iter()
                        .filter_map(|p| if p.device_type == StorageDeviceType::AhciSata {
                            Some(p.capacity_sectors * p.sector_size as u64)
                        } else {
                            None
                        })
                        .sum();
                }
            }
            StorageDeviceType::Nvme => {
                if let Some(ref mut controller) = self.nvme_controller {
                    controller.initialize()?;
                    self.capacity_bytes = controller.capacity_sectors * controller.sector_size as u64;
                }
            }
            _ => {
                return Err(format!("Unsupported device type: {:?}", self.device_type));
            }
        }

        self.state = StorageDeviceState::Ready;
        Ok(())
    }

    /// Leer sectores
    pub fn read_sectors(&mut self, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), String> {
        match self.device_type {
            StorageDeviceType::AhciSata => {
                if let Some(ref mut controller) = self.ahci_controller {
                    // Usar el primer puerto disponible
                    controller.read_sectors(0, lba, count, buffer)
                } else {
                    Err(String::from("No AHCI controller available"))
                }
            }
            StorageDeviceType::Nvme => {
                if let Some(ref mut controller) = self.nvme_controller {
                    controller.read_sectors(lba, count, buffer)
                } else {
                    Err(String::from("No NVMe controller available"))
                }
            }
            _ => Err(format!("Unsupported device type: {:?}", self.device_type)),
        }
    }

    /// Escribir sectores
    pub fn write_sectors(&mut self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), String> {
        match self.device_type {
            StorageDeviceType::AhciSata => {
                if let Some(ref mut controller) = self.ahci_controller {
                    controller.write_sectors(0, lba, count, buffer)
                } else {
                    Err(String::from("No AHCI controller available"))
                }
            }
            StorageDeviceType::Nvme => {
                if let Some(ref mut controller) = self.nvme_controller {
                    controller.write_sectors(lba, count, buffer)
                } else {
                    Err(String::from("No NVMe controller available"))
                }
            }
            _ => Err(format!("Unsupported device type: {:?}", self.device_type)),
        }
    }

    /// Verificar si está listo
    pub fn is_ready(&self) -> bool {
        self.state == StorageDeviceState::Ready
    }
}

/// Gestor de drivers de disco
pub struct DiskDriverManager {
    /// Dispositivos de almacenamiento
    pub devices: Vec<StorageDevice>,
    /// Referencia al graph kernel
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Próximo ID de dispositivo
    pub next_device_id: u64,
}

impl DiskDriverManager {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            graph_kernel: None,
            next_device_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Agregar un dispositivo AHCI
    pub fn add_ahci_device(&mut self, abar_address: u64, port_count: u32) -> Result<u64, String> {
        let device_id = self.next_device_id;
        self.next_device_id += 1;

        let config = AhciConfig {
            abar_address,
            port_count,
            enable_ncq: true,
        };

        let controller = AhciController::new(config);
        
        let mut device = StorageDevice::new(device_id, StorageDeviceType::AhciSata);
        device.ahci_controller = Some(controller);

        // Registrar el dispositivo como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::NodeType;
            let node_type = NodeType::HardwareDevice(crate::graph_kernel::HardwareType::Storage);
            let node_name = format!("ahci_device_{}", device_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            device.graph_node_id = node_id;
        }

        self.devices.push(device);
        Ok(device_id)
    }

    /// Agregar un dispositivo NVMe
    pub fn add_nvme_device(&mut self, mmio_address: u64, mmio_size: u32, queue_count: u32) -> Result<u64, String> {
        let device_id = self.next_device_id;
        self.next_device_id += 1;

        let config = NvmeConfig {
            mmio_address,
            mmio_size,
            queue_count,
        };

        let controller = NvmeController::new(config);
        
        let mut device = StorageDevice::new(device_id, StorageDeviceType::Nvme);
        device.nvme_controller = Some(controller);

        // Registrar el dispositivo como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::NodeType;
            let node_type = NodeType::HardwareDevice(crate::graph_kernel::HardwareType::Nvme);
            let node_name = format!("nvme_device_{}", device_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            device.graph_node_id = node_id;
        }

        self.devices.push(device);
        Ok(device_id)
    }

    /// Inicializar todos los dispositivos
    pub fn initialize_all(&mut self) -> Result<(), String> {
        for device in &mut self.devices {
            device.initialize()?;
        }
        Ok(())
    }

    /// Obtener un dispositivo por ID
    pub fn get_device(&self, device_id: u64) -> Option<&StorageDevice> {
        self.devices.iter().find(|d| d.id == device_id)
    }

    /// Obtener un dispositivo mutable por ID
    pub fn get_device_mut(&mut self, device_id: u64) -> Option<&mut StorageDevice> {
        self.devices.iter_mut().find(|d| d.id == device_id)
    }

    /// Obtener número de dispositivos
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Listar todos los dispositivos
    pub fn list_devices(&self) -> &[StorageDevice] {
        &self.devices
    }
}

impl Default for DiskDriverManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de drivers de disco
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskDriverError {
    DeviceNotFound,
    DeviceNotReady,
    InvalidDeviceType,
    InitializationFailed,
    ReadFailed,
    WriteFailed,
    InvalidLba,
    InvalidSectorCount,
}

impl fmt::Display for DiskDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiskDriverError::DeviceNotFound => write!(f, "Device not found"),
            DiskDriverError::DeviceNotReady => write!(f, "Device not ready"),
            DiskDriverError::InvalidDeviceType => write!(f, "Invalid device type"),
            DiskDriverError::InitializationFailed => write!(f, "Initialization failed"),
            DiskDriverError::ReadFailed => write!(f, "Read failed"),
            DiskDriverError::WriteFailed => write!(f, "Write failed"),
            DiskDriverError::InvalidLba => write!(f, "Invalid LBA"),
            DiskDriverError::InvalidSectorCount => write!(f, "Invalid sector count"),
        }
    }
}
