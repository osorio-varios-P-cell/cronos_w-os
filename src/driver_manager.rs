//! DriverManager Centralizado - FASE 2
//!
//! Este módulo unifica todos los drivers del sistema (Redox, GPU reales, Network reales)
//! bajo un único gestor centralizado que se integra con GraphKernel.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeType, HardwareType, NodeId, EdgeType};

// Importar drivers de Redox
use crate::drivers::{
    RedoxGpuDriver, RedoxNvmeDriver, RedoxXhciDriver, 
    RedoxWifiDriver, RedoxAudioDriver, RedoxNetworkDriver,
};

// Importar drivers de GPU reales
use crate::gpu_drivers::{
    GpuDriverManager, VesaDriver, IntelGpuDriver, AmdGpuDriver, NvidiaGpuDriver
};

// Importar drivers de red reales
use crate::network_drivers::{
    NetworkDriverManager, TcpIpStack, NetworkInterface, NetworkInterfaceConfig
};

/// Tipo de driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    /// GPU driver
    Gpu,
    /// Storage driver (NVMe, etc.)
    Storage,
    /// Network driver
    Network,
    /// USB driver
    Usb,
    /// Audio driver
    Audio,
    /// WiFi driver
    Wifi,
}

/// Estado del driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    /// No inicializado
    Uninitialized,
    /// Inicializando
    Initializing,
    /// Listo
    Ready,
    /// En error
    Error,
    /// Detenido
    Stopped,
}

/// Información de un driver registrado
#[derive(Debug, Clone)]
pub struct DriverInfo {
    /// ID único del driver
    pub id: u64,
    /// Tipo de driver
    pub driver_type: DriverType,
    /// Nombre del driver
    pub name: String,
    /// Estado actual
    pub state: DriverState,
    /// ID del nodo en el grafo
    pub graph_node_id: Option<NodeId>,
    /// Vendor ID (si aplica)
    pub vendor_id: Option<u16>,
    /// Device ID (si aplica)
    pub device_id: Option<u16>,
}

/// DriverManager Centralizado - FASE 2
pub struct DriverManager {
    /// Graph kernel reference
    graph_kernel: Option<Cell<GraphKernel>>,
    
    /// Drivers de Redox
    redox_gpu_drivers: BTreeMap<u64, RedoxGpuDriver>,
    redox_nvme_drivers: BTreeMap<u64, RedoxNvmeDriver>,
    redox_xhci_drivers: BTreeMap<u64, RedoxXhciDriver>,
    redox_wifi_drivers: BTreeMap<u64, RedoxWifiDriver>,
    redox_audio_drivers: BTreeMap<u64, RedoxAudioDriver>,
    redox_network_drivers: BTreeMap<u64, RedoxNetworkDriver>,
    
    /// Drivers de GPU reales
    gpu_driver_manager: Option<GpuDriverManager>,
    
    /// Drivers de red reales
    network_driver_manager: Option<NetworkDriverManager>,
    
    /// Información de todos los drivers
    driver_info: BTreeMap<u64, DriverInfo>,
    
    /// Next driver ID
    next_driver_id: u64,
    
    /// Estado global del manager
    global_state: DriverState,
}

impl DriverManager {
    /// Crear nuevo DriverManager centralizado
    pub fn new() -> Self {
        Self {
            graph_kernel: None,
            redox_gpu_drivers: BTreeMap::new(),
            redox_nvme_drivers: BTreeMap::new(),
            redox_xhci_drivers: BTreeMap::new(),
            redox_wifi_drivers: BTreeMap::new(),
            redox_audio_drivers: BTreeMap::new(),
            redox_network_drivers: BTreeMap::new(),
            gpu_driver_manager: None,
            network_driver_manager: None,
            driver_info: BTreeMap::new(),
            next_driver_id: 1,
            global_state: DriverState::Uninitialized,
        }
    }
    
    /// Establecer el GraphKernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        // Clonar antes de mover para poder propagar a los managers
        let graph_kernel_clone = graph_kernel.clone();
        self.graph_kernel = Some(Cell::new(graph_kernel));
        
        // Propagar a los managers existentes
        if let Some(ref mut gpu_manager) = self.gpu_driver_manager {
            gpu_manager.set_graph_kernel(graph_kernel_clone.clone());
        }
        
        if let Some(ref mut network_manager) = self.network_driver_manager {
            network_manager.set_graph_kernel(graph_kernel_clone);
        }
    }
    
    /// Inicializar todos los managers de drivers
    pub fn initialize_managers(&mut self) -> Result<(), String> {
        self.global_state = DriverState::Initializing;
        
        // Inicializar GPU driver manager
        let mut gpu_manager = GpuDriverManager::new();
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone());
            if let Some(clone) = gk_clone {
                gpu_manager.set_graph_kernel(clone);
            }
        }
        self.gpu_driver_manager = Some(gpu_manager);
        
        // Inicializar Network driver manager
        let mut network_manager = NetworkDriverManager::new();
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone());
            if let Some(clone) = gk_clone {
                network_manager.set_graph_kernel(clone);
            }
        }
        self.network_driver_manager = Some(network_manager);
        
        self.global_state = DriverState::Ready;
        Ok(())
    }
    
    /// Agregar un driver de GPU de Redox
    pub fn add_redox_gpu_driver(&mut self, driver: RedoxGpuDriver) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;
        
        // Establecer graph kernel si está disponible
        let mut driver = driver;
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone()).expect("Failed to clone graph kernel");
            driver.set_graph_kernel(gk_clone);
        }
        
        // Registrar en el grafo
        let node_id = driver.register_in_graph()?;
        
        // Guardar driver
        self.redox_gpu_drivers.insert(driver_id, driver);
        
        // Guardar información
        self.driver_info.insert(driver_id, DriverInfo {
            id: driver_id,
            driver_type: DriverType::Gpu,
            name: String::from("redox_gpu"),
            state: DriverState::Ready,
            graph_node_id: Some(node_id),
            vendor_id: None,
            device_id: None,
        });
        
        Ok(driver_id)
    }
    
    /// Agregar un driver de NVMe de Redox
    pub fn add_redox_nvme_driver(&mut self, driver: RedoxNvmeDriver) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;
        
        let mut driver = driver;
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone()).expect("Failed to clone graph kernel");
            driver.set_graph_kernel(gk_clone);
        }
        
        let node_id = driver.register_in_graph()?;
        self.redox_nvme_drivers.insert(driver_id, driver);
        
        self.driver_info.insert(driver_id, DriverInfo {
            id: driver_id,
            driver_type: DriverType::Storage,
            name: String::from("redox_nvme"),
            state: DriverState::Ready,
            graph_node_id: Some(node_id),
            vendor_id: None,
            device_id: None,
        });
        
        Ok(driver_id)
    }
    
    /// Agregar un driver de xHCI de Redox
    pub fn add_redox_xhci_driver(&mut self, driver: RedoxXhciDriver) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;
        
        let mut driver = driver;
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone()).expect("Failed to clone graph kernel");
            driver.set_graph_kernel(gk_clone);
        }
        
        let node_id = driver.register_in_graph()?;
        self.redox_xhci_drivers.insert(driver_id, driver);
        
        self.driver_info.insert(driver_id, DriverInfo {
            id: driver_id,
            driver_type: DriverType::Usb,
            name: String::from("redox_xhci"),
            state: DriverState::Ready,
            graph_node_id: Some(node_id),
            vendor_id: None,
            device_id: None,
        });
        
        Ok(driver_id)
    }
    
    /// Agregar un driver de WiFi de Redox
    pub fn add_redox_wifi_driver(&mut self, driver: RedoxWifiDriver) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;
        
        let mut driver = driver;
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone()).expect("Failed to clone graph kernel");
            driver.set_graph_kernel(gk_clone);
        }
        
        let node_id = driver.register_in_graph()?;
        self.redox_wifi_drivers.insert(driver_id, driver);
        
        self.driver_info.insert(driver_id, DriverInfo {
            id: driver_id,
            driver_type: DriverType::Wifi,
            name: String::from("redox_wifi"),
            state: DriverState::Ready,
            graph_node_id: Some(node_id),
            vendor_id: None,
            device_id: None,
        });
        
        Ok(driver_id)
    }
    
    /// Agregar un driver de Audio de Redox
    pub fn add_redox_audio_driver(&mut self, driver: RedoxAudioDriver) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;
        
        let mut driver = driver;
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone()).expect("Failed to clone graph kernel");
            driver.set_graph_kernel(gk_clone);
        }
        
        let node_id = driver.register_in_graph()?;
        self.redox_audio_drivers.insert(driver_id, driver);
        
        self.driver_info.insert(driver_id, DriverInfo {
            id: driver_id,
            driver_type: DriverType::Audio,
            name: String::from("redox_audio"),
            state: DriverState::Ready,
            graph_node_id: Some(node_id),
            vendor_id: None,
            device_id: None,
        });
        
        Ok(driver_id)
    }
    
    /// Agregar un driver de Network de Redox
    pub fn add_redox_network_driver(&mut self, driver: RedoxNetworkDriver) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;
        
        let mut driver = driver;
        if let Some(ref graph_kernel) = self.graph_kernel {
            let gk_clone = invoke_capability(&graph_kernel.capability(), |gk| gk.clone()).expect("Failed to clone graph kernel");
            driver.set_graph_kernel(gk_clone);
        }
        
        let node_id = driver.register_in_graph()?;
        self.redox_network_drivers.insert(driver_id, driver);
        
        self.driver_info.insert(driver_id, DriverInfo {
            id: driver_id,
            driver_type: DriverType::Network,
            name: String::from("redox_network"),
            state: DriverState::Ready,
            graph_node_id: Some(node_id),
            vendor_id: None,
            device_id: None,
        });
        
        Ok(driver_id)
    }
    
    /// Obtener información de un driver
    pub fn get_driver_info(&self, driver_id: u64) -> Option<&DriverInfo> {
        self.driver_info.get(&driver_id)
    }
    
    /// Obtener todos los drivers de un tipo específico
    pub fn get_drivers_by_type(&self, driver_type: DriverType) -> Vec<&DriverInfo> {
        self.driver_info.values()
            .filter(|info| info.driver_type == driver_type)
            .collect()
    }
    
    /// Obtener número total de drivers
    pub fn total_driver_count(&self) -> usize {
        self.driver_info.len()
    }
    
    /// Obtener número de drivers por tipo
    pub fn driver_count_by_type(&self, driver_type: DriverType) -> usize {
        self.get_drivers_by_type(driver_type).len()
    }
    
    /// Detener todos los drivers
    pub fn shutdown_all(&mut self) -> Result<(), String> {
        self.global_state = DriverState::Stopped;
        
        // Limpiar todos los drivers
        self.redox_gpu_drivers.clear();
        self.redox_nvme_drivers.clear();
        self.redox_xhci_drivers.clear();
        self.redox_wifi_drivers.clear();
        self.redox_audio_drivers.clear();
        self.redox_network_drivers.clear();
        self.driver_info.clear();
        
        Ok(())
    }
}

impl Default for DriverManager {
    fn default() -> Self {
        Self::new()
    }
}
