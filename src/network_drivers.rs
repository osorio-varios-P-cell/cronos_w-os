//! Drivers de Red Reales (Ethernet) con Stack TCP/IP para CRONOS W-OS
//!
//! Este módulo implementa drivers de red para tarjetas Ethernet reales
//! y un stack TCP/IP básico, adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Dirección MAC (48 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress {
    pub bytes: [u8; 6],
}

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self { bytes }
    }

    pub fn broadcast() -> Self {
        Self { bytes: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }
    }

    pub fn is_broadcast(&self) -> bool {
        self.bytes == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    pub fn is_multicast(&self) -> bool {
        self.bytes[0] & 0x01 == 0x01
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bytes[0], self.bytes[1], self.bytes[2],
            self.bytes[3], self.bytes[4], self.bytes[5])
    }
}

/// Dirección IPv4
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4Address {
    pub bytes: [u8; 4],
}

impl Ipv4Address {
    pub fn new(bytes: [u8; 4]) -> Self {
        Self { bytes }
    }

    pub fn from_u32(addr: u32) -> Self {
        Self {
            bytes: [
                (addr >> 24) as u8,
                (addr >> 16) as u8,
                (addr >> 8) as u8,
                addr as u8,
            ]
        }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.bytes[0] as u32) << 24) |
        ((self.bytes[1] as u32) << 16) |
        ((self.bytes[2] as u32) << 8) |
        (self.bytes[3] as u32)
    }

    pub fn localhost() -> Self {
        Self { bytes: [127, 0, 0, 1] }
    }

    pub fn any() -> Self {
        Self { bytes: [0, 0, 0, 0] }
    }

    pub fn broadcast() -> Self {
        Self { bytes: [255, 255, 255, 255] }
    }
}

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}",
            self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3])
    }
}

/// Dirección IPv6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv6Address {
    pub bytes: [u8; 16],
}

impl Ipv6Address {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    pub fn localhost() -> Self {
        Self { bytes: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1] }
    }

    pub fn any() -> Self {
        Self { bytes: [0; 16] }
    }
}

/// Protocolo de red
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkProtocol {
    Ipv4,
    Ipv6,
    Arp,
    Icmp,
    Tcp,
    Udp,
}

/// Paquete de red
#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub data: Vec<u8>,
    pub protocol: NetworkProtocol,
    pub source_mac: MacAddress,
    pub dest_mac: MacAddress,
    pub source_ip: Option<Ipv4Address>,
    pub dest_ip: Option<Ipv4Address>,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
}

impl NetworkPacket {
    pub fn new(data: Vec<u8>, protocol: NetworkProtocol) -> Self {
        Self {
            data,
            protocol,
            source_mac: MacAddress::new([0, 0, 0, 0, 0, 0]),
            dest_mac: MacAddress::new([0, 0, 0, 0, 0, 0]),
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: None,
        }
    }
}

/// Estado del driver de red
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDriverState {
    /// No inicializado
    Uninitialized,
    /// Inicializando
    Initializing,
    /// Listo
    Ready,
    /// Transmitiendo
    Transmitting,
    /// Recibiendo
    Receiving,
    /// Error
    Error(String),
}

/// Configuración de interfaz de red
#[derive(Debug, Clone)]
pub struct NetworkInterfaceConfig {
    /// Dirección MAC
    pub mac_address: MacAddress,
    /// Dirección IPv4
    pub ipv4_address: Ipv4Address,
    /// Máscara de subred
    pub subnet_mask: Ipv4Address,
    /// Puerta de enlace
    pub gateway: Ipv4Address,
    /// MTU (Maximum Transmission Unit)
    pub mtu: u16,
    /// Habilitar DHCP
    pub enable_dhcp: bool,
}

impl Default for NetworkInterfaceConfig {
    fn default() -> Self {
        Self {
            mac_address: MacAddress::new([0, 0, 0, 0, 0, 0]),
            ipv4_address: Ipv4Address::new([192, 168, 1, 100]),
            subnet_mask: Ipv4Address::new([255, 255, 255, 0]),
            gateway: Ipv4Address::new([192, 168, 1, 1]),
            mtu: 1500,
            enable_dhcp: true,
        }
    }
}

/// Interfaz de red
pub struct NetworkInterface {
    pub id: u64,
    pub name: String,
    pub state: NetworkDriverState,
    pub config: NetworkInterfaceConfig,
    pub tx_queue: Vec<NetworkPacket>,
    pub rx_queue: Vec<NetworkPacket>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl NetworkInterface {
    pub fn new(id: u64, name: String, config: NetworkInterfaceConfig) -> Self {
        Self {
            id,
            name,
            state: NetworkDriverState::Uninitialized,
            config,
            tx_queue: Vec::new(),
            rx_queue: Vec::new(),
            graph_node_id: None,
        }
    }

    /// Inicializar la interfaz
    pub fn initialize(&mut self) -> Result<(), String> {
        self.state = NetworkDriverState::Initializing;

        // En un sistema real, aquí se:
        // 1. Detectaría el tipo de tarjeta de red (Intel, Realtek, etc.)
        // 2. Inicializaría el hardware
        // 3. Configuraría los registros del dispositivo
        // 4. Habilitaría las interrupciones
        // 5. Configuraría DMA si está disponible

        self.state = NetworkDriverState::Ready;
        Ok(())
    }

    /// Enviar un paquete
    pub fn send_packet(&mut self, packet: NetworkPacket) -> Result<(), String> {
        if self.state != NetworkDriverState::Ready {
            return Err(format!("Interface not ready, state: {:?}", self.state));
        }

        self.tx_queue.push(packet);
        self.state = NetworkDriverState::Transmitting;

        // En un sistema real, aquí se:
        // 1. Prepararía el descriptor de transmisión
        // 2. Copiaría los datos al buffer del dispositivo
        // 3. Iniciaría la transmisión
        // 4. Esperaría a que termine

        self.state = NetworkDriverState::Ready;
        Ok(())
    }

    /// Recibir un paquete
    pub fn receive_packet(&mut self) -> Option<NetworkPacket> {
        if self.rx_queue.is_empty() {
            return None;
        }
        Some(self.rx_queue.remove(0))
    }

    /// Verificar si está lista
    pub fn is_ready(&self) -> bool {
        self.state == NetworkDriverState::Ready
    }
}

/// Stack TCP/IP
pub struct TcpIpStack {
    pub interfaces: BTreeMap<u64, NetworkInterface>,
    pub arp_table: BTreeMap<Ipv4Address, MacAddress>,
    pub routing_table: BTreeMap<Ipv4Address, u64>, // IP -> interface_id
    pub next_interface_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl TcpIpStack {
    pub fn new() -> Self {
        Self {
            interfaces: BTreeMap::new(),
            arp_table: BTreeMap::new(),
            routing_table: BTreeMap::new(),
            next_interface_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Agregar una interfaz de red
    pub fn add_interface(&mut self, name: String, config: NetworkInterfaceConfig) -> Result<u64, String> {
        let interface_id = self.next_interface_id;
        self.next_interface_id += 1;

        let ipv4_address = config.ipv4_address;

        let mut interface = NetworkInterface::new(interface_id, name, config);

        // Registrar la interfaz como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::HardwareDevice(crate::graph_kernel::HardwareType::Network);
            let node_name = format!("network_interface_{}", interface_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            interface.graph_node_id = node_id;
        }

        interface.initialize()?;
        self.interfaces.insert(interface_id, interface);

        // Agregar a la tabla de rutas
        self.routing_table.insert(ipv4_address, interface_id);

        Ok(interface_id)
    }

    /// Obtener una interfaz por ID
    pub fn get_interface(&self, interface_id: u64) -> Option<&NetworkInterface> {
        self.interfaces.get(&interface_id)
    }

    /// Obtener una interfaz mutable por ID
    pub fn get_interface_mut(&mut self, interface_id: u64) -> Option<&mut NetworkInterface> {
        self.interfaces.get_mut(&interface_id)
    }

    /// Enviar un paquete TCP
    pub fn send_tcp(&mut self, interface_id: u64, dest_ip: Ipv4Address, dest_port: u16, data: &[u8]) -> Result<(), String> {
        if let Some(interface) = self.get_interface_mut(interface_id) {
            let mut packet = NetworkPacket::new(data.to_vec(), NetworkProtocol::Tcp);
            packet.dest_ip = Some(dest_ip);
            packet.dest_port = Some(dest_port);
            packet.source_ip = Some(interface.config.ipv4_address);
            interface.send_packet(packet)
        } else {
            Err(format!("Interface {} not found", interface_id))
        }
    }

    /// Enviar un paquete UDP
    pub fn send_udp(&mut self, interface_id: u64, dest_ip: Ipv4Address, dest_port: u16, data: &[u8]) -> Result<(), String> {
        if let Some(interface) = self.get_interface_mut(interface_id) {
            let mut packet = NetworkPacket::new(data.to_vec(), NetworkProtocol::Udp);
            packet.dest_ip = Some(dest_ip);
            packet.dest_port = Some(dest_port);
            packet.source_ip = Some(interface.config.ipv4_address);
            interface.send_packet(packet)
        } else {
            Err(format!("Interface {} not found", interface_id))
        }
    }

    /// Enviar un paquete ICMP (ping)
    pub fn send_ping(&mut self, interface_id: u64, dest_ip: Ipv4Address) -> Result<(), String> {
        if let Some(interface) = self.get_interface_mut(interface_id) {
            let mut packet = NetworkPacket::new(vec![8, 0, 0, 0, 0, 0, 0, 0], NetworkProtocol::Icmp); // Echo request
            packet.dest_ip = Some(dest_ip);
            packet.source_ip = Some(interface.config.ipv4_address);
            interface.send_packet(packet)
        } else {
            Err(format!("Interface {} not found", interface_id))
        }
    }

    /// Agregar entrada ARP
    pub fn add_arp_entry(&mut self, ip: Ipv4Address, mac: MacAddress) {
        self.arp_table.insert(ip, mac);
    }

    /// Buscar entrada ARP
    pub fn lookup_arp(&self, ip: Ipv4Address) -> Option<MacAddress> {
        self.arp_table.get(&ip).copied()
    }

    /// Procesar paquetes recibidos
    pub fn process_rx_queue(&mut self, interface_id: u64) -> Vec<NetworkPacket> {
        let mut processed = Vec::new();
        if let Some(interface) = self.get_interface_mut(interface_id) {
            while let Some(packet) = interface.receive_packet() {
                processed.push(packet);
            }
        }
        processed
    }

    /// Obtener número de interfaces
    pub fn interface_count(&self) -> usize {
        self.interfaces.len()
    }
}

impl Default for TcpIpStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver de red Intel (e1000)
pub struct IntelE1000Driver {
    pub mmio_address: u64,
    pub mac_address: MacAddress,
    pub state: NetworkDriverState,
}

impl IntelE1000Driver {
    pub fn new(mmio_address: u64) -> Self {
        Self {
            mmio_address,
            mac_address: MacAddress::new([0, 0, 0, 0, 0, 0]),
            state: NetworkDriverState::Uninitialized,
        }
    }

    /// Inicializar el driver
    pub fn initialize(&mut self) -> Result<(), String> {
        self.state = NetworkDriverState::Initializing;

        // En un sistema real, aquí se:
        // 1. Leería la dirección MAC desde el registro EEPROM
        // 2. Inicializaría los registros del dispositivo
        // 3. Configuraría los descriptores de recepción y transmisión
        // 4. Habilitaría las interrupciones

        self.state = NetworkDriverState::Ready;
        Ok(())
    }

    /// Leer dirección MAC
    pub fn read_mac_address(&mut self) -> MacAddress {
        // En un sistema real, aquí se leería desde el registro EEPROM
        MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56])
    }
}

/// Driver de red Realtek (RTL8139)
pub struct RealtekRtl8139Driver {
    pub io_base: u16,
    pub mac_address: MacAddress,
    pub state: NetworkDriverState,
}

impl RealtekRtl8139Driver {
    pub fn new(io_base: u16) -> Self {
        Self {
            io_base,
            mac_address: MacAddress::new([0, 0, 0, 0, 0, 0]),
            state: NetworkDriverState::Uninitialized,
        }
    }

    /// Inicializar el driver
    pub fn initialize(&mut self) -> Result<(), String> {
        self.state = NetworkDriverState::Initializing;

        // En un sistema real, aquí se:
        // 1. Habilitaría el dispositivo en el bus PCI
        // 2. Leería la dirección MAC
        // 3. Configuraría los buffers de recepción y transmisión
        // 4. Habilitaría las interrupciones

        self.state = NetworkDriverState::Ready;
        Ok(())
    }

    /// Leer dirección MAC
    pub fn read_mac_address(&mut self) -> MacAddress {
        // En un sistema real, aquí se leería desde el registro MAC
        MacAddress::new([0x52, 0x54, 0x00, 0xAB, 0xCD, 0xEF])
    }
}

/// Gestor de drivers de red
pub struct NetworkDriverManager {
    pub tcp_ip_stack: TcpIpStack,
    pub intel_drivers: BTreeMap<u64, IntelE1000Driver>,
    pub realtek_drivers: BTreeMap<u64, RealtekRtl8139Driver>,
    pub next_driver_id: u64,
}

impl NetworkDriverManager {
    pub fn new() -> Self {
        Self {
            tcp_ip_stack: TcpIpStack::new(),
            intel_drivers: BTreeMap::new(),
            realtek_drivers: BTreeMap::new(),
            next_driver_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.tcp_ip_stack.set_graph_kernel(graph_kernel);
    }

    /// Agregar un driver Intel e1000
    pub fn add_intel_driver(&mut self, mmio_address: u64) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;

        let mut driver = IntelE1000Driver::new(mmio_address);
        driver.initialize()?;
        let mac = driver.read_mac_address();

        let config = NetworkInterfaceConfig {
            mac_address: mac,
            ..Default::default()
        };

        self.tcp_ip_stack.add_interface(format!("eth{}", driver_id), config)?;
        self.intel_drivers.insert(driver_id, driver);

        Ok(driver_id)
    }

    /// Agregar un driver Realtek RTL8139
    pub fn add_realtek_driver(&mut self, io_base: u16) -> Result<u64, String> {
        let driver_id = self.next_driver_id;
        self.next_driver_id += 1;

        let mut driver = RealtekRtl8139Driver::new(io_base);
        driver.initialize()?;
        let mac = driver.read_mac_address();

        let config = NetworkInterfaceConfig {
            mac_address: mac,
            ..Default::default()
        };

        self.tcp_ip_stack.add_interface(format!("eth{}", driver_id), config)?;
        self.realtek_drivers.insert(driver_id, driver);

        Ok(driver_id)
    }

    /// Obtener el stack TCP/IP
    pub fn tcp_ip_stack(&mut self) -> &mut TcpIpStack {
        &mut self.tcp_ip_stack
    }

    /// Obtener número de drivers
    pub fn driver_count(&self) -> usize {
        self.intel_drivers.len() + self.realtek_drivers.len()
    }
}

impl Default for NetworkDriverManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de drivers de red
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDriverError {
    DriverNotFound,
    InterfaceNotFound,
    InitializationFailed,
    TransmissionFailed,
    ReceptionFailed,
    InvalidAddress,
    BufferFull,
}

impl fmt::Display for NetworkDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkDriverError::DriverNotFound => write!(f, "Driver not found"),
            NetworkDriverError::InterfaceNotFound => write!(f, "Interface not found"),
            NetworkDriverError::InitializationFailed => write!(f, "Initialization failed"),
            NetworkDriverError::TransmissionFailed => write!(f, "Transmission failed"),
            NetworkDriverError::ReceptionFailed => write!(f, "Reception failed"),
            NetworkDriverError::InvalidAddress => write!(f, "Invalid address"),
            NetworkDriverError::BufferFull => write!(f, "Buffer full"),
        }
    }
}
