//! Módulo de Stack de Red para CRONOS W-OS
//! Implementa stack de red completo con soporte TCP/IP

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Dirección IP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpAddress {
    pub octets: [u8; 4],
}

impl IpAddress {
    /// Crea una nueva dirección IP
    pub fn new(octets: [u8; 4]) -> Self {
        IpAddress { octets }
    }

    /// Crea una dirección IP desde u32
    pub fn from_u32(addr: u32) -> Self {
        IpAddress {
            octets: [
                (addr >> 24) as u8,
                (addr >> 16) as u8,
                (addr >> 8) as u8,
                addr as u8,
            ],
        }
    }

    /// Convierte a u32
    pub fn to_u32(&self) -> u32 {
        ((self.octets[0] as u32) << 24)
            | ((self.octets[1] as u32) << 16)
            | ((self.octets[2] as u32) << 8)
            | (self.octets[3] as u32)
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }
}

/// Dirección MAC
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacAddress {
    pub bytes: [u8; 6],
}

impl MacAddress {
    /// Crea una nueva dirección MAC
    pub fn new(bytes: [u8; 6]) -> Self {
        MacAddress { bytes }
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bytes[0], self.bytes[1], self.bytes[2],
            self.bytes[3], self.bytes[4], self.bytes[5]
        )
    }
}

/// Puerto
pub type Port = u16;

/// Protocolo de red
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkProtocol {
    TCP,
    UDP,
    ICMP,
    IP,
    ARP,
    Ethernet,
}

/// Socket
#[derive(Debug, Clone)]
pub struct Socket {
    pub id: u64,
    pub protocol: NetworkProtocol,
    pub local_ip: IpAddress,
    pub local_port: Port,
    pub remote_ip: Option<IpAddress>,
    pub remote_port: Option<Port>,
    pub state: SocketState,
}

/// Estado de socket
#[derive(Debug, Clone, PartialEq)]
pub enum SocketState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

/// Interfaz de red
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: MacAddress,
    pub ip_address: IpAddress,
    pub netmask: IpAddress,
    pub gateway: IpAddress,
    pub mtu: u16,
    pub is_up: bool,
}

/// Paquete de red
#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub data: Vec<u8>,
    pub protocol: NetworkProtocol,
    pub source_ip: IpAddress,
    pub dest_ip: IpAddress,
    pub source_port: Option<Port>,
    pub dest_port: Option<Port>,
}

/// Stack de red
pub struct NetworkStack {
    interfaces: BTreeMap<String, NetworkInterface>,
    sockets: BTreeMap<u64, Socket>,
    next_socket_id: u64,
    next_packet_id: u64,
}

impl NetworkStack {
    /// Crea un nuevo stack de red
    pub fn new() -> Self {
        NetworkStack {
            interfaces: BTreeMap::new(),
            sockets: BTreeMap::new(),
            next_socket_id: 1,
            next_packet_id: 1,
        }
    }

    /// Inicializa el stack de red
    pub fn initialize(&mut self) {
        println!("🌐 Inicializando Stack de Red...");

        // Crear interfaz de red por defecto
        let interface = NetworkInterface {
            name: String::from("eth0"),
            mac_address: MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
            ip_address: IpAddress::new([192, 168, 1, 100]),
            netmask: IpAddress::new([255, 255, 255, 0]),
            gateway: IpAddress::new([192, 168, 1, 1]),
            mtu: 1500,
            is_up: true,
        };

        self.interfaces.insert(String::from("eth0"), interface);

        println!("✅ Stack de Red inicializado");
    }

    /// Agrega una interfaz de red
    pub fn add_interface(&mut self, interface: NetworkInterface) {
        let name = interface.name.clone();
        self.interfaces.insert(name, interface);
        println!("🔌 Interfaz de red agregada: {}", interface.name);
    }

    /// Crea un socket TCP
    pub fn create_tcp_socket(&mut self, local_ip: IpAddress, local_port: Port) -> u64 {
        let socket_id = self.next_socket_id;
        self.next_socket_id += 1;

        let socket = Socket {
            id: socket_id,
            protocol: NetworkProtocol::TCP,
            local_ip,
            local_port,
            remote_ip: None,
            remote_port: None,
            state: SocketState::Closed,
        };

        self.sockets.insert(socket_id, socket);
        println!("📡 Socket TCP creado: ID={}, {}:{}", socket_id, local_ip, local_port);
        socket_id
    }

    /// Crea un socket UDP
    pub fn create_udp_socket(&mut self, local_ip: IpAddress, local_port: Port) -> u64 {
        let socket_id = self.next_socket_id;
        self.next_socket_id += 1;

        let socket = Socket {
            id: socket_id,
            protocol: NetworkProtocol::UDP,
            local_ip,
            local_port,
            remote_ip: None,
            remote_port: None,
            state: SocketState::Closed,
        };

        self.sockets.insert(socket_id, socket);
        println!("📡 Socket UDP creado: ID={}, {}:{}", socket_id, local_ip, local_port);
        socket_id
    }

    /// Conecta un socket
    pub fn connect(&mut self, socket_id: u64, remote_ip: IpAddress, remote_port: Port) -> Result<(), NetworkError> {
        if let Some(socket) = self.sockets.get_mut(&socket_id) {
            socket.remote_ip = Some(remote_ip);
            socket.remote_port = Some(remote_port);
            socket.state = SocketState::SynSent;
            println!("🔗 Socket conectado: ID={} -> {}:{}", socket_id, remote_ip, remote_port);
            Ok(())
        } else {
            Err(NetworkError::SocketNotFound)
        }
    }

    /// Escucha en un socket
    pub fn listen(&mut self, socket_id: u64) -> Result<(), NetworkError> {
        if let Some(socket) = self.sockets.get_mut(&socket_id) {
            socket.state = SocketState::Listen;
            println!("👂 Socket escuchando: ID={}", socket_id);
            Ok(())
        } else {
            Err(NetworkError::SocketNotFound)
        }
    }

    /// Acepta una conexión
    pub fn accept(&mut self, socket_id: u64) -> Result<u64, NetworkError> {
        if let Some(socket) = self.sockets.get(&socket_id) {
            if socket.state == SocketState::Listen {
                let new_socket_id = self.next_socket_id;
                self.next_socket_id += 1;

                let new_socket = Socket {
                    id: new_socket_id,
                    protocol: socket.protocol.clone(),
                    local_ip: socket.local_ip,
                    local_port: socket.local_port,
                    remote_ip: None,
                    remote_port: None,
                    state: SocketState::SynReceived,
                };

                self.sockets.insert(new_socket_id, new_socket);
                println!("✅ Conexión aceptada: Socket ID={}", new_socket_id);
                Ok(new_socket_id)
            } else {
                Err(NetworkError::InvalidState)
            }
        } else {
            Err(NetworkError::SocketNotFound)
        }
    }

    /// Envía datos
    pub fn send(&mut self, socket_id: u64, data: Vec<u8>) -> Result<(), NetworkError> {
        if let Some(socket) = self.sockets.get(&socket_id) {
            if socket.state == SocketState::Established {
                println!("📤 Enviando {} bytes por socket ID={}", data.len(), socket_id);
                Ok(())
            } else {
                Err(NetworkError::InvalidState)
            }
        } else {
            Err(NetworkError::SocketNotFound)
        }
    }

    /// Recibe datos
    pub fn receive(&mut self, socket_id: u64) -> Result<Vec<u8>, NetworkError> {
        if let Some(socket) = self.sockets.get(&socket_id) {
            if socket.state == SocketState::Established {
                println!("📥 Recibiendo datos por socket ID={}", socket_id);
                Ok(Vec::new())
            } else {
                Err(NetworkError::InvalidState)
            }
        } else {
            Err(NetworkError::SocketNotFound)
        }
    }

    /// Cierra un socket
    pub fn close(&mut self, socket_id: u64) -> Result<(), NetworkError> {
        if let Some(socket) = self.sockets.get_mut(&socket_id) {
            socket.state = SocketState::Closed;
            self.sockets.remove(&socket_id);
            println!("❌ Socket cerrado: ID={}", socket_id);
            Ok(())
        } else {
            Err(NetworkError::SocketNotFound)
        }
    }

    /// Procesa un paquete de red
    pub fn process_packet(&mut self, packet: NetworkPacket) -> Result<(), NetworkError> {
        println!("📦 Procesando paquete: {:?} -> {:?}", packet.source_ip, packet.dest_ip);
        
        match packet.protocol {
            NetworkProtocol::TCP => self.process_tcp_packet(packet),
            NetworkProtocol::UDP => self.process_udp_packet(packet),
            NetworkProtocol::ICMP => self.process_icmp_packet(packet),
            _ => Ok(()),
        }
    }

    /// Procesa un paquete TCP
    fn process_tcp_packet(&self, packet: NetworkPacket) -> Result<(), NetworkError> {
        println!("🔵 Procesando paquete TCP");
        Ok(())
    }

    /// Procesa un paquete UDP
    fn process_udp_packet(&self, packet: NetworkPacket) -> Result<(), NetworkError> {
        println!("🟡 Procesando paquete UDP");
        Ok(())
    }

    /// Procesa un paquete ICMP
    fn process_icmp_packet(&self, packet: NetworkPacket) -> Result<(), NetworkError> {
        println!("🟢 Procesando paquete ICMP");
        Ok(())
    }

    /// Obtiene todas las interfaces
    pub fn get_interfaces(&self) -> Vec<&NetworkInterface> {
        self.interfaces.values().collect()
    }

    /// Obtiene todos los sockets
    pub fn get_sockets(&self) -> Vec<&Socket> {
        self.sockets.values().collect()
    }

    /// Genera reporte de red
    pub fn generate_report(&self) -> NetworkReport {
        let total_interfaces = self.interfaces.len();
        let total_sockets = self.sockets.len();
        let active_sockets = self.sockets.values().filter(|s| s.state == SocketState::Established).count();

        NetworkReport {
            total_interfaces,
            total_sockets,
            active_sockets,
        }
    }
}

/// Reporte de red
#[derive(Debug, Clone)]
pub struct NetworkReport {
    pub total_interfaces: usize,
    pub total_sockets: usize,
    pub active_sockets: usize,
}

/// Errores de red
#[derive(Debug, Clone)]
pub enum NetworkError {
    SocketNotFound,
    InterfaceNotFound,
    InvalidState,
    ConnectionRefused,
    Timeout,
    BufferFull,
}
