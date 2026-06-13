//! DHCP Client Module
//! 
//! This module implements a DHCP client for automatic IP configuration.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de mensaje DHCP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpMessageType {
    /// Discover
    Discover = 1,
    /// Offer
    Offer = 2,
    /// Request
    Request = 3,
    /// Decline
    Decline = 4,
    /// Ack
    Ack = 5,
    /// Nak
    Nak = 6,
    /// Release
    Release = 7,
}

impl DhcpMessageType {
    /// Crear desde u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(DhcpMessageType::Discover),
            2 => Some(DhcpMessageType::Offer),
            3 => Some(DhcpMessageType::Request),
            4 => Some(DhcpMessageType::Decline),
            5 => Some(DhcpMessageType::Ack),
            6 => Some(DhcpMessageType::Nak),
            7 => Some(DhcpMessageType::Release),
            _ => None,
        }
    }

    /// Convertir a u8
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

/// Opción DHCP
#[derive(Debug, Clone)]
pub struct DhcpOption {
    /// Tipo de opción
    pub option_type: u8,
    /// Datos de la opción
    pub data: Vec<u8>,
}

impl DhcpOption {
    /// Crear nueva opción
    pub fn new(option_type: u8, data: Vec<u8>) -> Self {
        Self {
            option_type,
            data,
        }
    }

    /// Opción de subnet mask
    pub fn subnet_mask(mask: [u8; 4]) -> Self {
        Self::new(1, mask.to_vec())
    }

    /// Opción de router
    pub fn router(router: [u8; 4]) -> Self {
        Self::new(3, router.to_vec())
    }

    /// Opción de DNS
    pub fn dns_server(dns: [u8; 4]) -> Self {
        Self::new(6, dns.to_vec())
    }
}

/// Mensaje DHCP
#[derive(Debug, Clone)]
pub struct DhcpMessage {
    /// Tipo de mensaje
    pub message_type: DhcpMessageType,
    /// Dirección MAC del cliente
    pub client_mac: [u8; 6],
    /// Dirección IP ofrecida
    pub offered_ip: [u8; 4],
    /// Dirección IP del servidor
    pub server_ip: [u8; 4],
    /// Opciones
    pub options: Vec<DhcpOption>,
}

impl DhcpMessage {
    /// Crear nuevo mensaje
    pub fn new(message_type: DhcpMessageType, client_mac: [u8; 6]) -> Self {
        Self {
            message_type,
            client_mac,
            offered_ip: [0; 4],
            server_ip: [0; 4],
            options: Vec::new(),
        }
    }

    /// Agregar opción
    pub fn add_option(&mut self, option: DhcpOption) {
        self.options.push(option);
    }

    /// Obtener opción por tipo
    pub fn get_option(&self, option_type: u8) -> Option<&DhcpOption> {
        self.options.iter().find(|o| o.option_type == option_type)
    }
}

/// Configuración DHCP
#[derive(Debug, Clone)]
pub struct DhcpConfig {
    /// Dirección IP
    pub ip_address: [u8; 4],
    /// Máscara de subred
    pub subnet_mask: [u8; 4],
    /// Router (gateway)
    pub router: [u8; 4],
    /// Servidores DNS
    pub dns_servers: Vec<[u8; 4]>,
    /// Tiempo de lease
    pub lease_time: u32,
}

impl DhcpConfig {
    /// Crear nueva configuración
    pub fn new(ip_address: [u8; 4], subnet_mask: [u8; 4]) -> Self {
        Self {
            ip_address,
            subnet_mask,
            router: [0; 4],
            dns_servers: Vec::new(),
            lease_time: 86400, // 24 horas
        }
    }

    /// Convertir IP a string
    pub fn ip_to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.ip_address[0], self.ip_address[1], self.ip_address[2], self.ip_address[3]
        )
    }
}

/// Estado del cliente DHCP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpState {
    /// Inicializando
    Init,
    /// Enviando discover
    Selecting,
    /// Esperando offer
    Requesting,
    /// Configurado
    Bound,
    /// Renovando
    Renewing,
    /// Error
    Failed,
}

/// Cliente DHCP
pub struct DhcpClient {
    /// Estado
    pub state: DhcpState,
    /// Dirección MAC
    pub mac_address: [u8; 6],
    /// Configuración actual
    pub config: Option<DhcpConfig>,
    /// ID de transacción
    pub transaction_id: u32,
}

impl DhcpClient {
    /// Crear nuevo cliente
    pub fn new(mac_address: [u8; 6]) -> Self {
        Self {
            state: DhcpState::Init,
            mac_address,
            config: None,
            transaction_id: 0,
        }
    }

    /// Iniciar proceso DHCP
    pub fn start(&mut self) -> Result<(), String> {
        self.state = DhcpState::Selecting;
        self.transaction_id += 1;
        
        // En un sistema real, esto enviaría un DHCP DISCOVER
        Ok(())
    }

    /// Procesar mensaje DHCP
    pub fn process_message(&mut self, message: DhcpMessage) -> Result<(), String> {
        match message.message_type {
            DhcpMessageType::Offer => {
                if self.state == DhcpState::Selecting {
                    self.state = DhcpState::Requesting;
                    // En un sistema real, esto enviaría un DHCP REQUEST
                }
            }
            DhcpMessageType::Ack => {
                if self.state == DhcpState::Requesting {
                    let config = DhcpConfig::new(message.offered_ip, [255, 255, 255, 0]);
                    self.config = Some(config);
                    self.state = DhcpState::Bound;
                }
            }
            DhcpMessageType::Nak => {
                self.state = DhcpState::Init;
                self.config = None;
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Renovar lease
    pub fn renew(&mut self) -> Result<(), String> {
        if self.state != DhcpState::Bound {
            return Err(String::from("Not in bound state"));
        }

        self.state = DhcpState::Renewing;
        self.transaction_id += 1;
        
        // En un sistema real, esto enviaría un DHCP REQUEST para renovar
        self.state = DhcpState::Bound;
        
        Ok(())
    }

    /// Liberar configuración
    pub fn release(&mut self) -> Result<(), String> {
        if self.state != DhcpState::Bound {
            return Err(String::from("Not in bound state"));
        }

        // En un sistema real, esto enviaría un DHCP RELEASE
        self.config = None;
        self.state = DhcpState::Init;
        
        Ok(())
    }

    /// Obtener configuración
    pub fn get_config(&self) -> Option<&DhcpConfig> {
        self.config.as_ref()
    }

    /// Obtener estado
    pub fn get_state(&self) -> DhcpState {
        self.state
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("DHCP Client Status\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("State: {:?}\n", self.state));
        report.push_str(&format!("MAC Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
            self.mac_address[0], self.mac_address[1], self.mac_address[2],
            self.mac_address[3], self.mac_address[4], self.mac_address[5]
        ));
        
        if let Some(ref config) = self.config {
            report.push_str("\nConfiguration:\n");
            report.push_str(&format!("  IP Address: {}\n", config.ip_to_string()));
            report.push_str(&format!("  Subnet Mask: {}.{}.{}.{}\n",
                config.subnet_mask[0], config.subnet_mask[1], config.subnet_mask[2], config.subnet_mask[3]
            ));
            report.push_str(&format!("  Lease Time: {} seconds\n", config.lease_time));
        }
        
        report
    }
}

impl Default for DhcpClient {
    fn default() -> Self {
        Self::new([0; 6])
    }
}
