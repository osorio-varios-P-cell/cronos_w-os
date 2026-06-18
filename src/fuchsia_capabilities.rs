//! Fuchsia Zircon Capabilities Adaptation para CRONOS W-OS (AEGIS Layer)
//!
//! Este módulo adapta las capabilities de Fuchsia Zircon (zircon handles) a la capa AEGIS de CRONOS W-OS,
//! manteniendo la esencia del exokernel basado en grafos y las 4 capas

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Tipo de handle Zircon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZirconHandleType {
    /// Invalid handle
    Invalid,
    /// Process handle
    Process,
    /// Thread handle
    Thread,
    /// Virtual Memory Object (VMO)
    Vmo,
    /// Channel handle
    Channel,
    /// Socket handle
    Socket,
    /// Event handle
    Event,
    /// Event pair handle
    EventPair,
    /// FIFO handle
    Fifo,
    /// Timer handle
    Timer,
    /// Port handle
    Port,
    /// Interrupt handle
    Interrupt,
    /// IOMMU handle
    Iommu,
    /// Bus Transaction Initiator (BTI)
    Bti,
    /// Resource handle
    Resource,
    /// Job handle
    Job,
    /// Debuglog handle
    Debuglog,
    /// Clock handle
    Clock,
    /// Stream handle
    Stream,
    /// Pager handle
    Pager,
    /// Custom handle
    Custom,
}

/// Derecho de handle Zircon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZirconHandleRight {
    /// Read permission
    Read,
    /// Write permission
    Write,
    /// Execute permission
    Execute,
    /// Map permission
    Map,
    /// Transfer permission
    Transfer,
    /// Duplicate permission
    Duplicate,
    /// Wait permission
    Wait,
    /// Signal permission
    Signal,
    /// Property permission
    Property,
    /// Enumerate permission
    Enumerate,
    /// Destroy permission
    Destroy,
    /// Set policy permission
    SetPolicy,
    /// Get policy permission
    GetPolicy,
    /// Adjust policy permission
    AdjustPolicy,
}

/// Handle Zircon
#[derive(Debug, Clone)]
pub struct ZirconHandle {
    /// ID único del handle
    pub handle_id: u32,
    /// Tipo de handle
    pub handle_type: ZirconHandleType,
    /// Derechos del handle
    pub rights: Vec<ZirconHandleRight>,
    /// Estado del handle
    pub state: HandleState,
    /// Capability asociada
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Datos del handle
    pub data: Vec<u8>,
}

/// Estado del handle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandleState {
    /// No inicializado
    Uninitialized,
    /// Activo
    Active,
    /// Cerrado
    Closed,
    /// Error
    Error(String),
}

impl ZirconHandle {
    pub fn new(handle_id: u32, handle_type: ZirconHandleType) -> Self {
        Self {
            handle_id,
            handle_type,
            rights: Vec::new(),
            state: HandleState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            data: Vec::new(),
        }
    }

    /// Agregar derecho al handle
    pub fn add_right(&mut self, right: ZirconHandleRight) {
        if !self.rights.contains(&right) {
            self.rights.push(right);
        }
    }

    /// Verificar si tiene un derecho específico
    pub fn has_right(&self, right: ZirconHandleRight) -> bool {
        self.rights.contains(&right)
    }

    /// Inicializar el handle en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != HandleState::Uninitialized {
            return Err(format!("Handle ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("zircon_handle_{}", self.handle_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = HandleState::Active;
        Ok(())
    }

    /// Cerrar el handle
    pub fn close(&mut self) -> Result<(), String> {
        self.state = HandleState::Closed;
        Ok(())
    }

    /// Obtener estado del handle
    pub fn state(&self) -> &HandleState {
        &self.state
    }
}

/// Mensaje FIDL (Fuchsia Interface Definition Language)
#[derive(Debug, Clone)]
pub struct FidlMessage {
    /// ID único del mensaje
    pub message_id: u64,
    /// ID del servicio
    pub service_id: u64,
    /// Método del servicio
    pub method: String,
    /// Datos del mensaje
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

impl FidlMessage {
    pub fn new(message_id: u64, service_id: u64, method: String, data: Vec<u8>) -> Self {
        Self {
            message_id,
            service_id,
            method,
            data,
            timestamp: 0,
        }
    }
}

/// Servicio FIDL
#[derive(Debug, Clone)]
pub struct FidlService {
    /// ID único del servicio
    pub service_id: u64,
    /// Nombre del servicio
    pub name: String,
    /// Protocolo del servicio
    pub protocol: String,
    /// Handles asociados
    pub handles: Vec<u32>,
    /// Estado del servicio
    pub state: HandleState,
    /// Capability del servicio
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
}

impl FidlService {
    pub fn new(service_id: u64, name: String, protocol: String) -> Self {
        Self {
            service_id,
            name,
            protocol,
            handles: Vec::new(),
            state: HandleState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
        }
    }

    /// Agregar handle al servicio
    pub fn add_handle(&mut self, handle_id: u32) {
        self.handles.push(handle_id);
    }

    /// Inicializar el servicio en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != HandleState::Uninitialized {
            return Err(format!("Servicio ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("fidl_service_{}", self.service_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = HandleState::Active;
        Ok(())
    }

    /// Obtener estado del servicio
    pub fn state(&self) -> &HandleState {
        &self.state
    }
}

/// Gestor de Capabilities de Fuchsia
#[derive(Debug, Clone)]
pub struct FuchsiaCapabilityManager {
    /// Handles Zircon registrados (keyed by handle_id)
    pub handles: BTreeMap<u32, ZirconHandle>,
    /// Servicios FIDL registrados (keyed by service_id)
    pub services: BTreeMap<u64, FidlService>,
    /// Estado del gestor
    pub state: HandleState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del gestor
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de handle
    pub next_handle_id: u32,
    /// Siguiente ID de servicio
    pub next_service_id: u64,
}

impl FuchsiaCapabilityManager {
    pub fn new() -> Self {
        Self {
            handles: BTreeMap::new(),
            services: BTreeMap::new(),
            state: HandleState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_handle_id: 1,
            next_service_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = HandleState::Active;
    }

    /// Crear un nuevo handle Zircon
    pub fn create_handle(&mut self, handle_type: ZirconHandleType, rights: Vec<ZirconHandleRight>) -> Result<u32, String> {
        if self.state == HandleState::Uninitialized {
            return Err(String::from("FuchsiaCapabilityManager no inicializado. Llamar a set_graph_kernel primero."));
        }

        let handle_id = self.next_handle_id;
        let mut handle = ZirconHandle::new(handle_id, handle_type);

        // Agregar derechos al handle
        for right in rights {
            handle.add_right(right);
        }

        // Inicializar el handle en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                handle.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.handles.insert(handle_id, handle);
        self.next_handle_id += 1;

        Ok(handle_id)
    }

    /// Obtener un handle por ID
    pub fn get_handle(&self, handle_id: u32) -> Option<&ZirconHandle> {
        self.handles.get(&handle_id)
    }

    /// Obtener un handle mutable por ID
    pub fn get_handle_mut(&mut self, handle_id: u32) -> Option<&mut ZirconHandle> {
        self.handles.get_mut(&handle_id)
    }

    /// Cerrar un handle
    pub fn close_handle(&mut self, handle_id: u32) -> Result<(), String> {
        if let Some(handle) = self.get_handle_mut(handle_id) {
            handle.close()
        } else {
            Err(format!("Handle con ID {} no encontrado", handle_id))
        }
    }

    /// Duplicar un handle
    pub fn duplicate_handle(&mut self, handle_id: u32) -> Result<u32, String> {
        let original = self.get_handle(handle_id)
            .ok_or_else(|| format!("Handle con ID {} no encontrado", handle_id))?;

        let new_handle_id = self.next_handle_id;
        let mut new_handle = ZirconHandle::new(new_handle_id, original.handle_type);
        new_handle.rights = original.rights.clone();

        // Inicializar el nuevo handle
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                new_handle.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.handles.insert(new_handle_id, new_handle);
        self.next_handle_id += 1;

        Ok(new_handle_id)
    }

    /// Crear un nuevo servicio FIDL
    pub fn create_service(&mut self, name: String, protocol: String) -> Result<u64, String> {
        if self.state == HandleState::Uninitialized {
            return Err(String::from("FuchsiaCapabilityManager no inicializado. Llamar a set_graph_kernel primero."));
        }

        let service_id = self.next_service_id;
        let mut service = FidlService::new(service_id, name, protocol);

        // Inicializar el servicio en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                service.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.services.insert(service_id, service);
        self.next_service_id += 1;

        Ok(service_id)
    }

    /// Obtener un servicio por ID
    pub fn get_service(&self, service_id: u64) -> Option<&FidlService> {
        self.services.get(&service_id)
    }

    /// Obtener un servicio mutable por ID
    pub fn get_service_mut(&mut self, service_id: u64) -> Option<&mut FidlService> {
        self.services.get_mut(&service_id)
    }

    /// Agregar handle a un servicio
    pub fn add_handle_to_service(&mut self, service_id: u64, handle_id: u32) -> Result<(), String> {
        if let Some(service) = self.get_service_mut(service_id) {
            service.add_handle(handle_id);
            Ok(())
        } else {
            Err(format!("Servicio con ID {} no encontrado", service_id))
        }
    }

    /// Obtener número de handles
    pub fn handle_count(&self) -> usize {
        self.handles.len()
    }

    /// Obtener número de servicios
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Listar todos los handles
    pub fn list_handles(&self) -> Vec<&ZirconHandle> {
        self.handles.values().collect()
    }

    /// Listar todos los servicios
    pub fn list_services(&self) -> Vec<&FidlService> {
        self.services.values().collect()
    }

    /// Obtener handles por tipo
    pub fn get_handles_by_type(&self, handle_type: ZirconHandleType) -> Vec<&ZirconHandle> {
        self.handles.values()
            .filter(|h| h.handle_type == handle_type)
            .collect()
    }

    /// Obtener el estado del gestor
    pub fn state(&self) -> &HandleState {
        &self.state
    }
}

impl Default for FuchsiaCapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración Fuchsia Capabilities
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuchsiaCapabilityError {
    HandleNotFound,
    HandleAlreadyExists,
    HandleNotInitialized,
    ServiceNotFound,
    ServiceAlreadyExists,
    ServiceNotInitialized,
    InvalidHandleRight,
    InvalidHandleType,
}

impl fmt::Display for FuchsiaCapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuchsiaCapabilityError::HandleNotFound => write!(f, "Handle not found"),
            FuchsiaCapabilityError::HandleAlreadyExists => write!(f, "Handle already exists"),
            FuchsiaCapabilityError::HandleNotInitialized => write!(f, "Handle not initialized"),
            FuchsiaCapabilityError::ServiceNotFound => write!(f, "Service not found"),
            FuchsiaCapabilityError::ServiceAlreadyExists => write!(f, "Service already exists"),
            FuchsiaCapabilityError::ServiceNotInitialized => write!(f, "Service not initialized"),
            FuchsiaCapabilityError::InvalidHandleRight => write!(f, "Invalid handle right"),
            FuchsiaCapabilityError::InvalidHandleType => write!(f, "Invalid handle type"),
        }
    }
}
