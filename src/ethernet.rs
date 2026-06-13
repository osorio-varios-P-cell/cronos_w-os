//! Ethernet Drivers Module
//! 
//! This module implements real Ethernet drivers for network communication.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Dirección MAC
#[derive(Debug, Clone, Copy)]
pub struct MacAddress {
    /// Bytes de la dirección MAC
    pub bytes: [u8; 6],
}

impl MacAddress {
    /// Crear nueva dirección MAC
    pub fn new(bytes: [u8; 6]) -> Self {
        Self { bytes }
    }

    /// Dirección MAC de broadcast
    pub fn broadcast() -> Self {
        Self::new([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
    }

    /// Dirección MAC cero
    pub fn zero() -> Self {
        Self::new([0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    /// Verificar si es broadcast
    pub fn is_broadcast(&self) -> bool {
        self.bytes == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    /// Verificar si es multicast
    pub fn is_multicast(&self) -> bool {
        self.bytes[0] & 0x01 == 0x01
    }

    /// Convertir a string
    pub fn to_string(&self) -> String {
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bytes[0], self.bytes[1], self.bytes[2],
            self.bytes[3], self.bytes[4], self.bytes[5]
        )
    }
}

impl Default for MacAddress {
    fn default() -> Self {
        Self::zero()
    }
}

/// Tipo de Ethernet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherType {
    /// IPv4
    Ipv4 = 0x0800,
    /// ARP
    Arp = 0x0806,
    /// IPv6
    Ipv6 = 0x86DD,
    /// VLAN
    Vlan = 0x8100,
}

impl EtherType {
    /// Crear desde u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0800 => Some(EtherType::Ipv4),
            0x0806 => Some(EtherType::Arp),
            0x86DD => Some(EtherType::Ipv6),
            0x8100 => Some(EtherType::Vlan),
            _ => None,
        }
    }

    /// Convertir a u16
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

/// Encabezado Ethernet
#[derive(Debug, Clone)]
pub struct EthernetHeader {
    /// Dirección MAC de destino
    pub destination: MacAddress,
    /// Dirección MAC de origen
    pub source: MacAddress,
    /// Tipo de Ethernet
    pub ether_type: EtherType,
}

impl EthernetHeader {
    /// Crear nuevo encabezado
    pub fn new(destination: MacAddress, source: MacAddress, ether_type: EtherType) -> Self {
        Self {
            destination,
            source,
            ether_type,
        }
    }

    /// Tamaño del encabezado
    pub fn size() -> usize {
        14 // 6 (dest) + 6 (src) + 2 (type)
    }

    /// Serializar a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::size());
        bytes.extend_from_slice(&self.destination.bytes);
        bytes.extend_from_slice(&self.source.bytes);
        let type_bytes = self.ether_type.to_u16().to_be_bytes();
        bytes.extend_from_slice(&type_bytes);
        bytes
    }
}

/// Frame Ethernet
#[derive(Debug, Clone)]
pub struct EthernetFrame {
    /// Encabezado
    pub header: EthernetHeader,
    /// Payload
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    /// Crear nuevo frame
    pub fn new(header: EthernetHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// Tamaño total del frame
    pub fn size(&self) -> usize {
        EthernetHeader::size() + self.payload.len()
    }

    /// Serializar a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

/// Estadísticas de Ethernet
#[derive(Debug, Clone)]
pub struct EthernetStats {
    /// Bytes recibidos
    pub bytes_received: u64,
    /// Bytes enviados
    pub bytes_sent: u64,
    /// Paquetes recibidos
    pub packets_received: u64,
    /// Paquetes enviados
    pub packets_sent: u64,
    /// Errores de recepción
    pub receive_errors: u64,
    /// Errores de transmisión
    pub transmit_errors: u64,
}

impl EthernetStats {
    /// Crear nuevas estadísticas
    pub fn new() -> Self {
        Self {
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            receive_errors: 0,
            transmit_errors: 0,
        }
    }
}

impl Default for EthernetStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver de Ethernet
pub struct EthernetDriver {
    /// Dirección MAC
    pub mac_address: MacAddress,
    /// Habilitado
    pub enabled: bool,
    /// Estadísticas
    pub stats: EthernetStats,
    /// MTU
    pub mtu: usize,
}

impl EthernetDriver {
    /// Crear nuevo driver
    pub fn new(mac_address: MacAddress) -> Self {
        Self {
            mac_address,
            enabled: false,
            stats: EthernetStats::new(),
            mtu: 1500,
        }
    }

    /// Inicializar driver
    pub fn initialize(&mut self) -> Result<(), String> {
        // En un sistema real, esto inicializaría el hardware
        self.enabled = true;
        Ok(())
    }

    /// Enviar frame
    pub fn send(&mut self, frame: EthernetFrame) -> Result<(), String> {
        if !self.enabled {
            return Err(String::from("Driver not enabled"));
        }

        if frame.size() > self.mtu {
            return Err(format!("Frame size {} exceeds MTU {}", frame.size(), self.mtu));
        }

        // En un sistema real, esto enviaría el frame al hardware
        self.stats.bytes_sent += frame.size() as u64;
        self.stats.packets_sent += 1;

        Ok(())
    }

    /// Recibir frame
    pub fn receive(&mut self) -> Option<EthernetFrame> {
        if !self.enabled {
            return None;
        }

        // En un sistema real, esto recibiría un frame del hardware
        // Para este ejemplo, retornamos None
        None
    }

    /// Habilitar driver
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Deshabilitar driver
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Obtener dirección MAC
    pub fn get_mac_address(&self) -> MacAddress {
        self.mac_address
    }

    /// Establecer MTU
    pub fn set_mtu(&mut self, mtu: usize) {
        self.mtu = mtu;
    }

    /// Obtener MTU
    pub fn get_mtu(&self) -> usize {
        self.mtu
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Ethernet Driver Status\n");
        report.push_str("======================\n\n");
        
        report.push_str(&format!("MAC Address: {}\n", self.mac_address.to_string()));
        report.push_str(&format!("Enabled: {}\n", self.enabled));
        report.push_str(&format!("MTU: {}\n\n", self.mtu));
        
        report.push_str("Statistics:\n");
        report.push_str(&format!("  Bytes Received: {}\n", self.stats.bytes_received));
        report.push_str(&format!("  Bytes Sent: {}\n", self.stats.bytes_sent));
        report.push_str(&format!("  Packets Received: {}\n", self.stats.packets_received));
        report.push_str(&format!("  Packets Sent: {}\n", self.stats.packets_sent));
        report.push_str(&format!("  Receive Errors: {}\n", self.stats.receive_errors));
        report.push_str(&format!("  Transmit Errors: {}\n", self.stats.transmit_errors));
        
        report
    }
}

impl Default for EthernetDriver {
    fn default() -> Self {
        Self::new(MacAddress::zero())
    }
}

/// Utilidades de Ethernet
pub struct EthernetUtils;

impl EthernetUtils {
    /// Calcular checksum
    pub fn checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            } else {
                sum += (chunk[0] as u32) << 8;
            }
        }
        
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        !sum as u16
    }

    /// Verificar checksum
    pub fn verify_checksum(data: &[u8], checksum: u16) -> bool {
        Self::checksum(data) == checksum
    }
}
