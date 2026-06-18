//! Plan 9 9p Protocol Adaptation para CRONOS W-OS (Networking)
//!
//! Este módulo adapta el protocolo 9p de Plan 9 al networking de CRONOS W-OS,
//! manteniendo la esencia del exokernel basado en grafos y las 4 capas

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado de la conexión 9p
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NinePConnectionState {
    /// No conectado
    Disconnected,
    /// Conectando
    Connecting,
    /// Conectado
    Connected,
    /// Autenticando
    Authenticating,
    /// Autenticado
    Authenticated,
    /// Error
    Error(String),
}

/// Tipo de mensaje 9p
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NinePMessageType {
    /// Tversion - versión del protocolo
    Tversion,
    /// Rversion - respuesta de versión
    Rversion,
    /// Tauth - autenticación
    Tauth,
    /// Rauth - respuesta de autenticación
    Rauth,
    /// Tattach - adjuntar a un árbol de archivos
    Tattach,
    /// Rattach - respuesta de attach
    Rattach,
    /// Twalk - caminar en el árbol de archivos
    Twalk,
    /// Rwalk - respuesta de walk
    Rwalk,
    /// Topen - abrir archivo
    Topen,
    /// Ropen - respuesta de open
    Ropen,
    /// Tcreate - crear archivo
    Tcreate,
    /// Rcreate - respuesta de create
    Rcreate,
    /// Tread - leer archivo
    Tread,
    /// Rread - respuesta de read
    Rread,
    /// Twrite - escribir archivo
    Twrite,
    /// Rwrite - respuesta de write
    Rwrite,
    /// Tclunk - liberar fid
    Tclunk,
    /// Rclunk - respuesta de clunk
    Rclunk,
    /// Tremove - eliminar archivo
    Tremove,
    /// Rremove - respuesta de remove
    Rremove,
    /// Tstat - obtener estadísticas
    Tstat,
    /// Rstat - respuesta de stat
    Rstat,
    /// Twstat - escribir estadísticas
    Twstat,
    /// Rwstat - respuesta de wstat
    Rwstat,
}

/// Mensaje 9p
#[derive(Debug, Clone)]
pub struct NinePMessage {
    /// ID único del mensaje
    pub message_id: u64,
    /// Tipo de mensaje
    pub message_type: NinePMessageType,
    /// Tag del mensaje
    pub tag: u16,
    /// Datos del mensaje
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

impl NinePMessage {
    pub fn new(message_id: u64, message_type: NinePMessageType, tag: u16, data: Vec<u8>) -> Self {
        Self {
            message_id,
            message_type,
            tag,
            data,
            timestamp: 0,
        }
    }
}

/// Configuración de conexión 9p
#[derive(Debug, Clone)]
pub struct NinePConnectionConfig {
    /// ID único de la conexión
    pub connection_id: u64,
    /// Dirección del servidor
    pub server_address: String,
    /// Puerto
    pub port: u16,
    /// Versión del protocolo
    pub protocol_version: String,
    /// Tamaño máximo de mensaje
    pub max_message_size: u32,
    /// Habilitado
    pub enabled: bool,
}

impl NinePConnectionConfig {
    pub fn new(connection_id: u64, server_address: String, port: u16) -> Self {
        Self {
            connection_id,
            server_address,
            port,
            protocol_version: String::from("9P2000"),
            max_message_size: 65536,
            enabled: true,
        }
    }

    pub fn with_protocol_version(mut self, protocol_version: String) -> Self {
        self.protocol_version = protocol_version;
        self
    }

    pub fn with_max_message_size(mut self, max_message_size: u32) -> Self {
        self.max_message_size = max_message_size;
        self
    }
}

/// Conexión 9p
#[derive(Debug, Clone)]
pub struct NinePConnection {
    /// Configuración de la conexión
    pub config: NinePConnectionConfig,
    /// Estado actual
    pub state: NinePConnectionState,
    /// Capability de la conexión
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// FID actual
    pub current_fid: u32,
    /// Siguiente FID
    pub next_fid: u32,
    /// Siguiente tag
    pub next_tag: u16,
    /// Mensajes pendientes
    pub pending_messages: Vec<NinePMessage>,
}

impl NinePConnection {
    pub fn new(config: NinePConnectionConfig) -> Self {
        Self {
            config,
            state: NinePConnectionState::Disconnected,
            capability_id: None,
            graph_node_id: None,
            current_fid: 0,
            next_fid: 1,
            next_tag: 1,
            pending_messages: Vec::new(),
        }
    }

    /// Inicializar la conexión en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != NinePConnectionState::Disconnected {
            return Err(format!("Conexión ya inicializada, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("9p_connection_{}", self.config.connection_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = NinePConnectionState::Connecting;
        Ok(())
    }

    /// Conectar al servidor
    pub fn connect(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Err(String::from("Conexión no está habilitada"));
        }
        self.state = NinePConnectionState::Connected;
        Ok(())
    }

    /// Autenticar
    pub fn authenticate(&mut self) -> Result<(), String> {
        self.state = NinePConnectionState::Authenticating;
        // En un sistema real, aquí se enviaría un mensaje Tauth
        self.state = NinePConnectionState::Authenticated;
        Ok(())
    }

    /// Desconectar
    pub fn disconnect(&mut self) -> Result<(), String> {
        self.state = NinePConnectionState::Disconnected;
        Ok(())
    }

    /// Crear un nuevo FID
    pub fn allocate_fid(&mut self) -> u32 {
        let fid = self.next_fid;
        self.next_fid += 1;
        fid
    }

    /// Crear un nuevo tag
    pub fn allocate_tag(&mut self) -> u16 {
        let tag = self.next_tag;
        self.next_tag = if tag == u16::MAX { 1 } else { tag + 1 };
        tag
    }

    /// Agregar mensaje pendiente
    pub fn add_pending_message(&mut self, message: NinePMessage) {
        self.pending_messages.push(message);
    }

    /// Obtener mensajes pendientes
    pub fn get_pending_messages(&mut self) -> Vec<NinePMessage> {
        let messages = self.pending_messages.clone();
        self.pending_messages.clear();
        messages
    }

    /// Obtener estado de la conexión
    pub fn state(&self) -> &NinePConnectionState {
        &self.state
    }
}

/// Gestor de protocolo 9p
#[derive(Debug, Clone)]
pub struct NinePManager {
    /// Conexiones 9p registradas (keyed by connection_id)
    pub connections: BTreeMap<u64, NinePConnection>,
    /// Estado del gestor
    pub state: NinePConnectionState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del gestor
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de conexión
    pub next_connection_id: u64,
}

impl NinePManager {
    pub fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            state: NinePConnectionState::Disconnected,
            graph_kernel: None,
            capability_id: None,
            next_connection_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = NinePConnectionState::Connected;
    }

    /// Crear una nueva conexión 9p
    pub fn create_connection(&mut self, config: NinePConnectionConfig) -> Result<u64, String> {
        if self.state == NinePConnectionState::Disconnected {
            return Err(String::from("NinePManager no inicializado. Llamar a set_graph_kernel primero."));
        }

        let connection_id = config.connection_id;
        let mut connection = NinePConnection::new(config);

        // Inicializar la conexión en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                connection.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.connections.insert(connection_id, connection);
        self.next_connection_id = connection_id + 1;

        Ok(connection_id)
    }

    /// Crear una conexión con configuración predeterminada
    pub fn create_default_connection(&mut self, server_address: String, port: u16) -> Result<u64, String> {
        let connection_id = self.next_connection_id;
        let config = NinePConnectionConfig::new(connection_id, server_address, port);
        self.create_connection(config)
    }

    /// Obtener una conexión por ID
    pub fn get_connection(&self, connection_id: u64) -> Option<&NinePConnection> {
        self.connections.get(&connection_id)
    }

    /// Obtener una conexión mutable por ID
    pub fn get_connection_mut(&mut self, connection_id: u64) -> Option<&mut NinePConnection> {
        self.connections.get_mut(&connection_id)
    }

    /// Conectar una conexión
    pub fn connect(&mut self, connection_id: u64) -> Result<(), String> {
        if let Some(connection) = self.get_connection_mut(connection_id) {
            connection.connect()
        } else {
            Err(format!("Conexión con ID {} no encontrada", connection_id))
        }
    }

    /// Autenticar una conexión
    pub fn authenticate(&mut self, connection_id: u64) -> Result<(), String> {
        if let Some(connection) = self.get_connection_mut(connection_id) {
            connection.authenticate()
        } else {
            Err(format!("Conexión con ID {} no encontrada", connection_id))
        }
    }

    /// Desconectar una conexión
    pub fn disconnect(&mut self, connection_id: u64) -> Result<(), String> {
        if let Some(connection) = self.get_connection_mut(connection_id) {
            connection.disconnect()
        } else {
            Err(format!("Conexión con ID {} no encontrada", connection_id))
        }
    }

    /// Crear mensaje 9p
    pub fn create_message(&mut self, connection_id: u64, message_type: NinePMessageType, data: Vec<u8>) -> Result<u64, String> {
        let message_id = self.next_connection_id * 1000 + self.connections.len() as u64;
        let tag = if let Some(conn) = self.get_connection_mut(connection_id) {
            conn.allocate_tag()
        } else {
            return Err(format!("Conexión con ID {} no encontrada", connection_id));
        };

        let message = NinePMessage::new(message_id, message_type, tag, data);

        if let Some(connection) = self.get_connection_mut(connection_id) {
            connection.add_pending_message(message);
        }

        Ok(message_id)
    }

    /// Obtener mensajes pendientes de una conexión
    pub fn get_pending_messages(&mut self, connection_id: u64) -> Vec<NinePMessage> {
        if let Some(connection) = self.get_connection_mut(connection_id) {
            connection.get_pending_messages()
        } else {
            Vec::new()
        }
    }

    /// Obtener número de conexiones
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Listar todas las conexiones
    pub fn list_connections(&self) -> Vec<&NinePConnection> {
        self.connections.values().collect()
    }

    /// Obtener conexiones activas
    pub fn get_active_connections(&self) -> Vec<&NinePConnection> {
        self.connections.values()
            .filter(|c| c.state == NinePConnectionState::Authenticated)
            .collect()
    }

    /// Obtener el estado del gestor
    pub fn state(&self) -> &NinePConnectionState {
        &self.state
    }
}

impl Default for NinePManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración 9p
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NinePError {
    ConnectionNotFound,
    ConnectionAlreadyExists,
    ConnectionNotInitialized,
    AuthenticationFailed,
    MessageCreationFailed,
    InvalidMessageType,
    ProtocolVersionMismatch,
}

impl fmt::Display for NinePError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NinePError::ConnectionNotFound => write!(f, "Connection not found"),
            NinePError::ConnectionAlreadyExists => write!(f, "Connection already exists"),
            NinePError::ConnectionNotInitialized => write!(f, "Connection not initialized"),
            NinePError::AuthenticationFailed => write!(f, "Authentication failed"),
            NinePError::MessageCreationFailed => write!(f, "Message creation failed"),
            NinePError::InvalidMessageType => write!(f, "Invalid message type"),
            NinePError::ProtocolVersionMismatch => write!(f, "Protocol version mismatch"),
        }
    }
}
