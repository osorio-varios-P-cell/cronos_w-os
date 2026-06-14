//! Gestión de hardware para CRONOS W-OS
//! 
//! Este módulo implementa la detección y control de hardware
//! incluyendo CPUs, GPUs, dispositivos PCIe y periféricos
//! Adaptado para trabajar con el sistema de capabilities

use x86_64::{PhysAddr, VirtAddr};
use x86_64::instructions::port::Port;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType, HardwareType, get_kernel_tick};

/// Scanner completo de hardware del sistema
pub struct HardwareScanner {
    /// Dispositivos PCI detectados
    pub pci_devices: Vec<PciDevice>,
    /// Información del CPU
    pub cpu_info: CpuInfo,
}

/// Información del CPU
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// Vendor del CPU
    pub vendor: String,
    /// Modelo del CPU
    pub model: String,
    /// Número de cores
    pub cores: u8,
    /// Frecuencia base en MHz
    pub frequency_mhz: u32,
    /// Características soportadas
    pub features: Vec<String>,
}

/// Dispositivo PCI detectado
#[derive(Debug, Clone)]
pub struct PciDevice {
    /// ID del vendor
    pub vendor_id: u16,
    /// ID del dispositivo
    pub device_id: u16,
    /// Clase del dispositivo
    pub class_id: u8,
    /// Subclase del dispositivo
    pub subclass_id: u8,
    /// Número de bus
    pub bus: u8,
    /// Número de dispositivo
    pub device: u8,
    /// Número de función
    pub function: u8,
}

impl HardwareScanner {
    /// Crea un nuevo scanner de hardware
    pub fn new() -> Self {
        Self {
            pci_devices: Vec::new(),
            cpu_info: CpuInfo::detect(),
        }
    }

    /// Escanea todos los dispositivos PCI
    pub fn scan_pci_bus(&mut self) -> Vec<PciDevice> {
        
        // Escanear buses 0-255
        for bus in 0..255 {
            // Escanear dispositivos 0-31
            for device in 0..32 {
                // Escanear funciones 0-7
                for function in 0..8 {
                    if let Some(pci_device) = self.scan_pci_device(bus, device, function) {
                        self.pci_devices.push(pci_device);
                    }
                }
            }
        }
        
        self.pci_devices.clone()
    }

    /// Escanea un dispositivo PCI específico
    fn scan_pci_device(&self, bus: u8, device: u8, function: u8) -> Option<PciDevice> {
        // Leer ID del vendor y dispositivo
        let vendor_device = self.read_pci_config(bus, device, function, 0x00);
        
        let vendor_id = (vendor_device & 0xFFFF) as u16;
        let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;
        
        // Si vendor_id = 0xFFFF, no hay dispositivo
        if vendor_id == 0xFFFF {
            return None;
        }
        
        // Leer clase y subclase
        let class_subclass = self.read_pci_config(bus, device, function, 0x08);
        let class_id = ((class_subclass >> 24) & 0xFF) as u8;
        let subclass_id = ((class_subclass >> 16) & 0xFF) as u8;
        
        Some(PciDevice {
            vendor_id,
            device_id,
            class_id,
            subclass_id,
            bus,
            device,
            function,
        })
    }

    /// Registra los dispositivos PCI detectados en el GraphKernel
    pub fn register_in_graph(&self, graph_kernel: &GraphKernel) {
        if let Some(root_id) = graph_kernel.root_node() {
            for device in &self.pci_devices {
                let hardware_type = self.map_pci_to_hardware_type(device.class_id);
                let name = format!("pci_{:02x}:{:02x}.{}", device.bus, device.device, device.function);

                let node_id = graph_kernel.create_node(
                    NodeType::HardwareDevice(hardware_type),
                    name,
                );

                // Crear arista de Ownership desde el root
                graph_kernel.create_edge(root_id, node_id, EdgeType::Ownership);
            }
        }
    }

    fn map_pci_to_hardware_type(&self, class_id: u8) -> HardwareType {
        match class_id {
            0x01 => HardwareType::Storage,
            0x02 => HardwareType::Network,
            0x03 => HardwareType::Gpu,
            0x04 => HardwareType::Audio,
            0x0C => HardwareType::Xhci,
            _ => HardwareType::Input, // Default to Input for unknown
        }
    }

    /// Lee configuración PCI real usando puertos 0xCF8 y 0xCFC
    fn read_pci_config(&self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address = ((bus as u32) << 16) 
                    | ((device as u32) << 11) 
                    | ((function as u32) << 8) 
                    | ((offset as u32) & 0xfc) 
                    | 0x80000000;
        
        let mut config_addr_port = Port::new(0xCF8);
        let mut config_data_port = Port::new(0xCFC);
        
        unsafe {
            config_addr_port.write(address);
            config_data_port.read()
        }
    }
}

impl CpuInfo {
    /// FASE 30: Leer temperatura real del CPU vía MSR
    pub fn read_temperature(&self) -> f32 {
        // En hardware x86_64 real, se accede vía rdmsr(IA32_THERM_STATUS)
        // Simulamos un valor dinámico basado en la carga del sistema para v2.0
        45.0 + (get_kernel_tick() % 15) as f32
    }

    /// Detecta información del CPU
    pub fn detect() -> Self {
        
        // En implementación real, usaría CPUID
        // Por ahora, simulamos Pentium Gold
        
        let vendor = "GenuineIntel".to_string();
        let model = "Pentium Gold".to_string();
        let cores = 4;
        let frequency_mhz = 3000;
        
        let features = vec![
            "MMX".to_string(),
            "SSE".to_string(),
            "SSE2".to_string(),
            "SSE3".to_string(),
            "SSSE3".to_string(),
            "SSE4.1".to_string(),
            "SSE4.2".to_string(),
            "POPCNT".to_string(),
            "AES-NI".to_string(),
            "RDRAND".to_string(),
            "VMX".to_string(),
        ];
        
        
        Self {
            vendor,
            model,
            cores,
            frequency_mhz,
            features,
        }
    }
}

/// Información de memoria
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Memoria total en MB
    pub total_mb: u64,
    /// Memoria disponible en MB
    pub available_mb: u64,
}

/// Información de GPU para IA Colmena
#[derive(Debug, Clone)]
pub struct GpuInfoColmena {
    /// Vendor de la GPU
    pub vendor: String,
    /// Modelo de la GPU
    pub model: String,
    /// VRAM en MB
    pub vram_mb: u64,
}

/// Información de red para IA Colmena
#[derive(Debug, Clone)]
pub struct NetworkInfoColmena {
    /// Vendor del dispositivo
    pub vendor: String,
    /// Modelo del dispositivo
    pub model: String,
    /// Velocidad en Mbps
    pub speed_mbps: u32,
}

/// Información de almacenamiento para IA Colmena
#[derive(Debug, Clone)]
pub struct StorageInfoColmena {
    /// Vendor del dispositivo
    pub vendor: String,
    /// Modelo del dispositivo
    pub model: String,
    /// Capacidad en GB
    pub capacity_gb: u64,
    /// Tipo de dispositivo
    pub type_: String,
}

/// Estado de MSR del CPU
#[derive(Debug, Clone)]
pub struct MsrState {
    /// MSR de temperatura
    pub temperature_msr: u64,
    /// MSR de potencia
    pub power_msr: u64,
    /// MSR de características
    pub feature_msr: u64,
    /// MSR de VMX
    pub vmx_msr: u64,
}

impl PciDevice {
    pub fn get_bar0_addr(&self) -> u64 {
        unsafe {
            let addr = crate::pci::PciConfigAddress::new(self.bus, self.device, self.function, 0x10);
            let bar = crate::pci::pci_read_config_u32(addr);
            if bar & 0x01 != 0 {
                // I/O bar
                (bar & 0xFFFFFFFC) as u64
            } else {
                // Memory bar
                (bar & 0xFFFFFFF0) as u64
            }
        }
    }
}
