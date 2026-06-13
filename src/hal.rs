//! CRONOS-HAL - Hardware Abstraction Layer Traits
//! 
//! This module defines the hardware abstraction layer traits that will be used
//! to wrap Redox drivers in capabilities. These traits provide a common interface
//! for all hardware drivers in the system.
//!
//! FASE 13: HAL trait para virtio-drivers-and-devices

use crate::capability::{Capability, Cell, CapabilityRights};
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::ptr::NonNull;

/// FASE 13: HAL trait para virtio-drivers-and-devices
pub trait Hal {
    /// Allocate DMA memory region
    fn dma_alloc(&mut self, size: usize) -> Option<NonNull<u8>>;
    
    /// Free DMA memory region
    fn dma_free(&mut self, ptr: NonNull<u8>, size: usize);
    
    /// Translate physical address to virtual address
    fn phys_to_virt(&self, phys: usize) -> usize;
    
    /// Translate virtual address to physical address
    fn virt_to_phys(&self, virt: usize) -> usize;
}

/// FASE 13: Implementación HAL básica para CRONOS
pub struct CronosHal;

impl Hal for CronosHal {
    fn dma_alloc(&mut self, size: usize) -> Option<NonNull<u8>> {
        // FASE 13: Implementación básica usando el allocator del kernel
        let layout = alloc::alloc::Layout::from_size_align(size, 4096).ok()?;
        unsafe {
            let ptr = alloc::alloc::alloc(layout);
            if ptr.is_null() {
                None
            } else {
                NonNull::new(ptr)
            }
        }
    }
    
    fn dma_free(&mut self, ptr: NonNull<u8>, size: usize) {
        let layout = alloc::alloc::Layout::from_size_align(size, 4096).unwrap();
        unsafe {
            alloc::alloc::dealloc(ptr.as_ptr(), layout);
        }
    }
    
    fn phys_to_virt(&self, phys: usize) -> usize {
        // FASE 13: En CRONOS, mapeo 1:1 por ahora
        phys
    }
    
    fn virt_to_phys(&self, virt: usize) -> usize {
        // FASE 13: En CRONOS, mapeo 1:1 por ahora
        virt
    }
}

/// Base trait for all hardware devices
pub trait Device {
    /// Get the device name
    fn name(&self) -> &str;
    
    /// Get the device vendor ID
    fn vendor_id(&self) -> u16;
    
    /// Get the device ID
    fn device_id(&self) -> u16;
    
    /// Initialize the device
    fn init(&mut self) -> Result<(), DeviceError>;
    
    /// Reset the device
    fn reset(&mut self) -> Result<(), DeviceError>;
    
    /// Check if the device is ready
    fn is_ready(&self) -> bool;
    
    /// Get device capabilities
    fn capabilities(&self) -> DeviceCapabilities;
}

/// Device capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub dma: bool,
    pub interrupt: bool,
    pub mmio: bool,
    pub pio: bool,
}

impl DeviceCapabilities {
    pub const NONE: Self = Self {
        dma: false,
        interrupt: false,
        mmio: false,
        pio: false,
    };

    pub const STANDARD: Self = Self {
        dma: true,
        interrupt: true,
        mmio: true,
        pio: false,
    };
}

/// Device errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceError {
    InitializationFailed,
    ResetFailed,
    NotReady,
    Timeout,
    InvalidParameter,
    HardwareError,
    CapabilityDenied,
}

/// Trait for GPU devices
pub trait GpuDevice: Device {
    /// Set display resolution
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), DeviceError>;
    
    /// Get current resolution
    fn get_resolution(&self) -> (u32, u32);
    
    /// Get framebuffer pointer
    fn framebuffer(&self) -> *mut u8;
    
    /// Get framebuffer size
    fn framebuffer_size(&self) -> usize;
    
    /// Swap buffers
    fn swap_buffers(&mut self) -> Result<(), DeviceError>;
    
    /// Create a GPU context
    fn create_context(&mut self) -> Result<GpuContext, DeviceError>;
    
    /// Destroy a GPU context
    fn destroy_context(&mut self, context: GpuContext) -> Result<(), DeviceError>;
    
    /// Execute GPU command
    fn execute_command(&mut self, context: &GpuContext, command: GpuCommand) -> Result<(), DeviceError>;
}

/// GPU context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuContext(pub u64);

/// GPU commands
#[derive(Debug, Clone)]
pub enum GpuCommand {
    Clear { r: u8, g: u8, b: u8, a: u8 },
    DrawRect { x: u32, y: u32, width: u32, height: u32, color: u32 },
    DrawText { x: u32, y: u32, text: alloc::string::String },
    Blit { src_x: u32, src_y: u32, dst_x: u32, dst_y: u32, width: u32, height: u32 },
}

/// Trait for NVMe storage devices
pub trait NvmeDevice: Device {
    /// Get number of namespaces
    fn namespace_count(&self) -> u32;
    
    /// Get namespace size
    fn namespace_size(&self, namespace_id: u32) -> Result<u64, DeviceError>;
    
    /// Read from namespace
    fn read(&mut self, namespace_id: u32, lba: u64, buffer: &mut [u8]) -> Result<(), DeviceError>;
    
    /// Write to namespace
    fn write(&mut self, namespace_id: u32, lba: u64, buffer: &[u8]) -> Result<(), DeviceError>;
    
    /// Flush namespace
    fn flush(&mut self, namespace_id: u32) -> Result<(), DeviceError>;
}

/// Trait for xHCI (USB) host controllers
pub trait XhciDevice: Device {
    /// Get number of ports
    fn port_count(&self) -> u32;
    
    /// Reset port
    fn reset_port(&mut self, port: u32) -> Result<(), DeviceError>;
    
    /// Enable port
    fn enable_port(&mut self, port: u32) -> Result<(), DeviceError>;
    
    /// Disable port
    fn disable_port(&mut self, port: u32) -> Result<(), DeviceError>;
    
    /// Get port status
    fn port_status(&self, port: u32) -> Result<PortStatus, DeviceError>;
    
    /// Submit USB transfer
    fn submit_transfer(&mut self, endpoint: u8, buffer: &[u8]) -> Result<(), DeviceError>;
    
    /// Receive USB transfer
    fn receive_transfer(&mut self, endpoint: u8, buffer: &mut [u8]) -> Result<usize, DeviceError>;
}

/// Port status for xHCI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortStatus {
    pub connected: bool,
    pub enabled: bool,
    pub speed: UsbSpeed,
}

/// USB speed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
}

/// Trait for WiFi devices
pub trait WifiDevice: Device {
    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Scan for networks
    fn scan_networks(&mut self) -> Result<Vec<WifiNetwork>, DeviceError>;
    
    /// Connect to network
    fn connect(&mut self, ssid: &str, password: &str) -> Result<(), DeviceError>;
    
    /// Disconnect from network
    fn disconnect(&mut self) -> Result<(), DeviceError>;
    
    /// Get connection status
    fn connection_status(&self) -> ConnectionStatus;
    
    /// Send packet
    fn send_packet(&mut self, packet: &[u8]) -> Result<(), DeviceError>;
    
    /// Receive packet
    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, DeviceError>;
}

/// WiFi network information
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: alloc::string::String,
    pub signal_strength: i8,
    pub encryption: WifiEncryption,
}

/// WiFi encryption type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiEncryption {
    Open,
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
}

/// Connection status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Trait for audio devices
pub trait AudioDevice: Device {
    /// Get number of channels
    fn channel_count(&self) -> u32;
    
    /// Get sample rate
    fn sample_rate(&self) -> u32;
    
    /// Set sample rate
    fn set_sample_rate(&mut self, rate: u32) -> Result<(), DeviceError>;
    
    /// Get buffer size
    fn buffer_size(&self) -> usize;
    
    /// Play audio buffer
    fn play(&mut self, buffer: &[u8]) -> Result<(), DeviceError>;
    
    /// Record audio buffer
    fn record(&mut self, buffer: &mut [u8]) -> Result<usize, DeviceError>;
    
    /// Stop playback
    fn stop(&mut self) -> Result<(), DeviceError>;
    
    /// Set volume
    fn set_volume(&mut self, volume: f32) -> Result<(), DeviceError>;
    
    /// Get volume
    fn volume(&self) -> f32;
}

/// Trait for network devices
pub trait NetworkDevice: Device {
    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Get MTU
    fn mtu(&self) -> u32;
    
    /// Set MTU
    fn set_mtu(&mut self, mtu: u32) -> Result<(), DeviceError>;
    
    /// Get link status
    fn link_status(&self) -> LinkStatus;
    
    /// Send packet
    fn send_packet(&mut self, packet: &[u8]) -> Result<(), DeviceError>;
    
    /// Receive packet
    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, DeviceError>;
    
    /// Get statistics
    fn statistics(&self) -> NetworkStatistics;
}

/// Link status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkStatus {
    Down,
    Up,
    Unknown,
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStatistics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u64,
}

/// Trait for input devices
pub trait InputDevice: Device {
    /// Get input type
    fn input_type(&self) -> InputType;
    
    /// Read input event
    fn read_event(&mut self) -> Result<InputEvent, DeviceError>;
    
    /// Check if event available
    fn has_event(&self) -> bool;
}

/// Input type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    Keyboard,
    Mouse,
    Touchscreen,
    Gamepad,
}

/// Input event
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyEvent { key_code: u32, pressed: bool },
    MouseEvent { x: i32, y: i32, buttons: u32 },
    TouchEvent { x: i32, y: i32, pressed: bool },
}

/// Capability wrapper for devices
pub struct DeviceCapability<T: Device> {
    device: Cell<T>,
    rights: CapabilityRights,
}

impl<T: Device> DeviceCapability<T> {
    pub fn new(device: T, rights: CapabilityRights) -> Self {
        Self {
            device: Cell::new(device),
            rights,
        }
    }

    pub fn capability(&self) -> Capability<T> {
        self.device.capability_with_rights(self.rights)
    }

    pub fn rights(&self) -> CapabilityRights {
        self.rights
    }
}

/// Generic driver wrapper that can hold any device type
pub enum Driver {
    Gpu(Box<dyn GpuDevice>),
    Nvme(Box<dyn NvmeDevice>),
    Xhci(Box<dyn XhciDevice>),
    Wifi(Box<dyn WifiDevice>),
    Audio(Box<dyn AudioDevice>),
    Network(Box<dyn NetworkDevice>),
    Input(Box<dyn InputDevice>),
}

impl Driver {
    pub fn name(&self) -> &str {
        match self {
            Driver::Gpu(d) => d.name(),
            Driver::Nvme(d) => d.name(),
            Driver::Xhci(d) => d.name(),
            Driver::Wifi(d) => d.name(),
            Driver::Audio(d) => d.name(),
            Driver::Network(d) => d.name(),
            Driver::Input(d) => d.name(),
        }
    }

    pub fn init(&mut self) -> Result<(), DeviceError> {
        match self {
            Driver::Gpu(d) => d.init(),
            Driver::Nvme(d) => d.init(),
            Driver::Xhci(d) => d.init(),
            Driver::Wifi(d) => d.init(),
            Driver::Audio(d) => d.init(),
            Driver::Network(d) => d.init(),
            Driver::Input(d) => d.init(),
        }
    }

    pub fn reset(&mut self) -> Result<(), DeviceError> {
        match self {
            Driver::Gpu(d) => d.reset(),
            Driver::Nvme(d) => d.reset(),
            Driver::Xhci(d) => d.reset(),
            Driver::Wifi(d) => d.reset(),
            Driver::Audio(d) => d.reset(),
            Driver::Network(d) => d.reset(),
            Driver::Input(d) => d.reset(),
        }
    }
}

/// Driver manager for managing all drivers in the system
pub struct DriverManager {
    drivers: alloc::collections::BTreeMap<alloc::string::String, Driver>,
}

impl DriverManager {
    pub fn new() -> Self {
        Self {
            drivers: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn register(&mut self, name: alloc::string::String, driver: Driver) {
        self.drivers.insert(name, driver);
    }

    pub fn get(&self, name: &str) -> Option<&Driver> {
        self.drivers.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Driver> {
        self.drivers.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Driver> {
        self.drivers.remove(name)
    }

    pub fn list(&self) -> Vec<alloc::string::String> {
        self.drivers.keys().cloned().collect()
    }

    pub fn init_all(&mut self) -> Result<(), Vec<alloc::string::String>> {
        let mut errors = Vec::new();
        for (name, driver) in self.drivers.iter_mut() {
            if let Err(_) = driver.init() {
                errors.push(name.clone());
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_capabilities() {
        let caps = DeviceCapabilities::STANDARD;
        assert!(caps.dma);
        assert!(caps.interrupt);
        assert!(caps.mmio);
    }

    #[test]
    fn test_driver_manager() {
        let mut manager = DriverManager::new();
        assert_eq!(manager.list().len(), 0);
    }
}
