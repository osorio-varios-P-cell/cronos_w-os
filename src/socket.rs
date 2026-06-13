//! Socket API Module
//! 
//! This module implements the Socket API for network communication.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Dominio de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketDomain {
    /// IPv4
    AfInet = 2,
    /// IPv6
    AfInet6 = 10,
}

/// Tipo de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// Stream (TCP)
    Stream = 1,
    /// Datagram (UDP)
    Datagram = 2,
    /// Raw
    Raw = 3,
}

/// Protocolo de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketProtocol {
    /// IP
    Ip = 0,
    /// ICMP
    Icmp = 1,
    /// TCP
    Tcp = 6,
    /// UDP
    Udp = 17,
}

/// Dirección de socket
#[derive(Debug, Clone)]
pub struct SocketAddress {
    /// Familia de direcciones
    pub family: SocketDomain,
    /// Puerto
    pub port: u16,
    /// Dirección IP
    pub address: [u8; 4],
}

impl SocketAddress {
    /// Crear nueva dirección
    pub fn new(family: SocketDomain, port: u16, address: [u8; 4]) -> Self {
        Self {
            family,
            port,
            address,
        }
    }

    /// Dirección de loopback
    pub fn loopback(port: u16) -> Self {
        Self::new(SocketDomain::AfInet, port, [127, 0, 0, 1])
    }

    /// Dirección any
    pub fn any(port: u16) -> Self {
        Self::new(SocketDomain::AfInet, port, [0, 0, 0, 0])
    }

    /// Convertir a string
    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}:{}",
            self.address[0], self.address[1], self.address[2], self.address[3],
            self.port
        )
    }
}

/// Estado de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    /// Cerrado
    Closed,
    /// Escuchando
    Listening,
    /// Conectando
    Connecting,
    /// Conectado
    Connected,
    /// Cerrando
    Closing,
}

/// Socket
pub struct Socket {
    /// ID del socket
    pub socket_id: u32,
    /// Dominio
    pub domain: SocketDomain,
    /// Tipo
    pub socket_type: SocketType,
    /// Protocolo
    pub protocol: SocketProtocol,
    /// Estado
    pub state: SocketState,
    /// Dirección local
    pub local_address: Option<SocketAddress>,
    /// Dirección remota
    pub remote_address: Option<SocketAddress>,
    /// Buffer de recepción
    pub receive_buffer: Vec<u8>,
    /// Buffer de envío
    pub send_buffer: Vec<u8>,
}

impl Socket {
    /// Crear nuevo socket
    pub fn new(socket_id: u32, domain: SocketDomain, socket_type: SocketType, protocol: SocketProtocol) -> Self {
        Self {
            socket_id,
            domain,
            socket_type,
            protocol,
            state: SocketState::Closed,
            local_address: None,
            remote_address: None,
            receive_buffer: Vec::new(),
            send_buffer: Vec::new(),
        }
    }

    /// Conectar a dirección
    pub fn connect(&mut self, address: SocketAddress) -> Result<(), String> {
        if self.state != SocketState::Closed {
            return Err(String::from("Socket not in closed state"));
        }

        self.remote_address = Some(address);
        self.state = SocketState::Connecting;
        
        // En un sistema real, esto iniciaría la conexión TCP
        self.state = SocketState::Connected;
        
        Ok(())
    }

    /// Enlazar a dirección
    pub fn bind(&mut self, address: SocketAddress) -> Result<(), String> {
        if self.state != SocketState::Closed {
            return Err(String::from("Socket not in closed state"));
        }

        self.local_address = Some(address);
        Ok(())
    }

    /// Escuchar conexiones
    pub fn listen(&mut self, backlog: u32) -> Result<(), String> {
        if self.state != SocketState::Closed {
            return Err(String::from("Socket not in closed state"));
        }

        if self.local_address.is_none() {
            return Err(String::from("Socket not bound"));
        }

        // En un sistema real, esto iniciaría el listen
        self.state = SocketState::Listening;
        
        let _ = backlog;
        Ok(())
    }

    /// Aceptar conexión
    pub fn accept(&mut self) -> Result<Socket, String> {
        if self.state != SocketState::Listening {
            return Err(String::from("Socket not in listening state"));
        }

        // En un sistema real, esto aceptaría una conexión entrante
        Err(String::from("No pending connections"))
    }

    /// Enviar datos
    pub fn send(&mut self, data: &[u8]) -> Result<usize, String> {
        if self.state != SocketState::Connected {
            return Err(String::from("Socket not connected"));
        }

        self.send_buffer.extend_from_slice(data);
        Ok(data.len())
    }

    /// Recibir datos
    pub fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, String> {
        if self.state != SocketState::Connected {
            return Err(String::from("Socket not connected"));
        }

        if self.receive_buffer.is_empty() {
            return Ok(0);
        }

        let len = buffer.len().min(self.receive_buffer.len());
        buffer[..len].copy_from_slice(&self.receive_buffer[..len]);
        self.receive_buffer.drain(..len);

        Ok(len)
    }

    /// Cerrar socket
    pub fn close(&mut self) -> Result<(), String> {
        self.state = SocketState::Closing;
        
        // En un sistema real, esto cerraría la conexión
        self.state = SocketState::Closed;
        self.receive_buffer.clear();
        self.send_buffer.clear();
        
        Ok(())
    }

    /// Obtener estado
    pub fn get_state(&self) -> SocketState {
        self.state
    }
}

/// Gestor de sockets
pub struct SocketManager {
    /// Sockets activos
    pub sockets: Vec<Socket>,
    /// Siguiente ID de socket
    pub next_socket_id: u32,
}

impl SocketManager {
    /// Crear nuevo gestor
    pub fn new() -> Self {
        Self {
            sockets: Vec::new(),
            next_socket_id: 1,
        }
    }

    /// Crear socket
    pub fn socket(&mut self, domain: SocketDomain, socket_type: SocketType, protocol: SocketProtocol) -> Result<u32, String> {
        let socket_id = self.next_socket_id;
        let socket = Socket::new(socket_id, domain, socket_type, protocol);
        
        self.sockets.push(socket);
        self.next_socket_id += 1;
        
        Ok(socket_id)
    }

    /// Obtener socket por ID
    pub fn get(&self, socket_id: u32) -> Option<&Socket> {
        self.sockets.iter().find(|s| s.socket_id == socket_id)
    }

    /// Obtener socket mutable por ID
    pub fn get_mut(&mut self, socket_id: u32) -> Option<&mut Socket> {
        self.sockets.iter_mut().find(|s| s.socket_id == socket_id)
    }

    /// Cerrar socket
    pub fn close(&mut self, socket_id: u32) -> Result<(), String> {
        let index = self.sockets.iter()
            .position(|s| s.socket_id == socket_id)
            .ok_or_else(|| String::from("Socket not found"))?;
        
        self.sockets[index].close()?;
        self.sockets.remove(index);
        
        Ok(())
    }

    /// Obtener número de sockets activos
    pub fn socket_count(&self) -> usize {
        self.sockets.len()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Socket Manager Status\n");
        report.push_str("=====================\n\n");
        
        report.push_str(&format!("Active Sockets: {}\n\n", self.socket_count()));
        
        for socket in &self.sockets {
            report.push_str(&format!("Socket ID: {}\n", socket.socket_id));
            report.push_str(&format!("  State: {:?}\n", socket.state));
            if let Some(ref addr) = socket.local_address {
                report.push_str(&format!("  Local: {}\n", addr.to_string()));
            }
            if let Some(ref addr) = socket.remote_address {
                report.push_str(&format!("  Remote: {}\n", addr.to_string()));
            }
            report.push('\n');
        }
        
        report
    }
}

impl Default for SocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de socket
pub struct SocketUtils;

impl SocketUtils {
    /// Verificar si un puerto es válido
    pub fn is_valid_port(port: u16) -> bool {
        port > 0 && port <= 65535
    }

    /// Verificar si un puerto es privilegiado
    pub fn is_privileged_port(port: u16) -> bool {
        port < 1024
    }
}
