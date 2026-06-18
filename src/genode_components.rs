//! Genode Component Framework Adaptation para CRONOS W-OS (GENESIS Layer)
//!
//! Este módulo adapta el framework de componentes de Genode a la capa GENESIS de CRONOS W-OS,
//! manteniendo la esencia del exokernel basado en grafos y las 4 capas

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del componente
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Ejecutando
    Running,
    /// Pausado
    Paused,
    /// Detenido
    Stopped,
    /// Error
    Error(String),
}

/// Tipo de componente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    /// Servicio
    Service,
    /// Cliente
    Client,
    /// Driver
    Driver,
    /// Sistema de archivos
    Filesystem,
    /// Red
    Network,
    /// UI
    Ui,
    /// Custom
    Custom,
}

/// Configuración de componente
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    /// ID único del componente
    pub component_id: u64,
    /// Nombre del componente
    pub name: String,
    /// Tipo de componente
    pub component_type: ComponentType,
    /// Binario o código del componente
    pub binary: String,
    /// Recursos requeridos (CPU, memoria)
    pub resources: String,
    /// Dependencias de otros componentes
    pub dependencies: Vec<u64>,
    /// Habilitado
    pub enabled: bool,
}

impl ComponentConfig {
    pub fn new(component_id: u64, name: String, component_type: ComponentType, binary: String) -> Self {
        Self {
            component_id,
            name,
            component_type,
            binary,
            resources: String::new(),
            dependencies: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_resources(mut self, resources: String) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<u64>) -> Self {
        self.dependencies = dependencies;
        self
    }
}

/// Sesión de componente
#[derive(Debug, Clone)]
pub struct ComponentSession {
    /// ID único de la sesión
    pub session_id: u64,
    /// ID del componente
    pub component_id: u64,
    /// Estado de la sesión
    pub state: ComponentState,
    /// Capability de la sesión
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Timestamp de creación
    pub created_at: u64,
}

impl ComponentSession {
    pub fn new(session_id: u64, component_id: u64) -> Self {
        Self {
            session_id,
            component_id,
            state: ComponentState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            created_at: 0,
        }
    }

    /// Inicializar la sesión en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != ComponentState::Uninitialized {
            return Err(format!("Sesión ya inicializada, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("component_session_{}", self.session_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = ComponentState::Initialized;
        Ok(())
    }

    /// Iniciar la sesión
    pub fn start(&mut self) -> Result<(), String> {
        self.state = ComponentState::Running;
        Ok(())
    }

    /// Pausar la sesión
    pub fn pause(&mut self) -> Result<(), String> {
        self.state = ComponentState::Paused;
        Ok(())
    }

    /// Detener la sesión
    pub fn stop(&mut self) -> Result<(), String> {
        self.state = ComponentState::Stopped;
        Ok(())
    }

    /// Obtener estado de la sesión
    pub fn state(&self) -> &ComponentState {
        &self.state
    }
}

/// Componente Genode
pub struct GenodeComponent {
    /// Configuración del componente
    pub config: ComponentConfig,
    /// Estado actual
    pub state: ComponentState,
    /// Capability del componente
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Sesiones activas
    pub sessions: Vec<ComponentSession>,
    /// Siguiente ID de sesión
    pub next_session_id: u64,
}

impl GenodeComponent {
    pub fn new(config: ComponentConfig) -> Self {
        Self {
            config,
            state: ComponentState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            sessions: Vec::new(),
            next_session_id: 1,
        }
    }

    /// Inicializar el componente en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != ComponentState::Uninitialized {
            return Err(format!("Componente ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("genode_component_{}", self.config.component_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = ComponentState::Initialized;
        Ok(())
    }

    /// Iniciar el componente
    pub fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Err(String::from("Componente no está habilitado"));
        }
        self.state = ComponentState::Running;
        Ok(())
    }

    /// Pausar el componente
    pub fn pause(&mut self) -> Result<(), String> {
        self.state = ComponentState::Paused;
        Ok(())
    }

    /// Detener el componente
    pub fn stop(&mut self) -> Result<(), String> {
        self.state = ComponentState::Stopped;
        Ok(())
    }

    /// Crear una nueva sesión
    pub fn create_session(&mut self, graph_kernel: &GraphKernel) -> Result<u64, String> {
        let session_id = self.next_session_id;
        let mut session = ComponentSession::new(session_id, self.config.component_id);
        session.initialize(graph_kernel)?;
        session.start()?;
        self.sessions.push(session);
        self.next_session_id += 1;
        Ok(session_id)
    }

    /// Obtener una sesión por ID
    pub fn get_session(&self, session_id: u64) -> Option<&ComponentSession> {
        self.sessions.iter().find(|s| s.session_id == session_id)
    }

    /// Obtener una sesión mutable por ID
    pub fn get_session_mut(&mut self, session_id: u64) -> Option<&mut ComponentSession> {
        self.sessions.iter_mut().find(|s| s.session_id == session_id)
    }

    /// Eliminar una sesión
    pub fn remove_session(&mut self, session_id: u64) -> Result<(), String> {
        let pos = self.sessions.iter().position(|s| s.session_id == session_id)
            .ok_or_else(|| format!("Sesión con ID {} no encontrada", session_id))?;
        self.sessions.remove(pos).stop()?;
        Ok(())
    }

    /// Obtener estado del componente
    pub fn state(&self) -> &ComponentState {
        &self.state
    }
}

/// Gestor de componentes Genode
#[derive(Debug, Clone)]
pub struct GenodeComponentManager {
    /// Componentes registrados (keyed by component_id)
    pub components: BTreeMap<u64, GenodeComponent>,
    /// Estado del gestor
    pub state: ComponentState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del gestor
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de componente
    pub next_component_id: u64,
}

impl GenodeComponentManager {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            state: ComponentState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_component_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = ComponentState::Initialized;
    }

    /// Crear un nuevo componente
    pub fn create_component(&mut self, config: ComponentConfig) -> Result<u64, String> {
        if self.state == ComponentState::Uninitialized {
            return Err(String::from("GenodeComponentManager no inicializado. Llamar a set_graph_kernel primero."));
        }

        let component_id = config.component_id;
        let mut component = GenodeComponent::new(config);

        // Inicializar el componente en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                component.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.components.insert(component_id, component);
        self.next_component_id = component_id + 1;

        Ok(component_id)
    }

    /// Crear un componente con configuración predeterminada
    pub fn create_default_component(&mut self, name: String, component_type: ComponentType, binary: String) -> Result<u64, String> {
        let component_id = self.next_component_id;
        let config = ComponentConfig::new(component_id, name, component_type, binary);
        self.create_component(config)
    }

    /// Obtener un componente por ID
    pub fn get_component(&self, component_id: u64) -> Option<&GenodeComponent> {
        self.components.get(&component_id)
    }

    /// Obtener un componente mutable por ID
    pub fn get_component_mut(&mut self, component_id: u64) -> Option<&mut GenodeComponent> {
        self.components.get_mut(&component_id)
    }

    /// Iniciar un componente
    pub fn start_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.start()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Pausar un componente
    pub fn pause_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.pause()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Detener un componente
    pub fn stop_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.stop()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Crear sesión en un componente
    pub fn create_session(&mut self, component_id: u64) -> Result<u64, String> {
        // Primero obtener el graph_kernel capability
        let graph_kernel_cap = self.graph_kernel.as_ref()
            .map(|gk| gk.capability())
            .ok_or_else(|| String::from("Graph kernel no disponible"))?;

        // Luego obtener el componente mutable
        let component = self.get_component_mut(component_id)
            .ok_or_else(|| format!("Componente con ID {} no encontrado", component_id))?;

        // Finalmente crear la sesión con el graph_kernel
        let result = invoke_capability(&graph_kernel_cap, |gk| {
            component.create_session(gk)
        });
        result.unwrap_or(Err(String::from("Error al crear sesión")))
    }

    /// Obtener número de componentes
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Listar todos los componentes
    pub fn list_components(&self) -> Vec<&GenodeComponent> {
        self.components.values().collect()
    }

    /// Obtener componentes por tipo
    pub fn get_components_by_type(&self, component_type: ComponentType) -> Vec<&GenodeComponent> {
        self.components.values()
            .filter(|c| c.config.component_type == component_type)
            .collect()
    }

    /// Obtener componentes activos
    pub fn get_active_components(&self) -> Vec<&GenodeComponent> {
        self.components.values()
            .filter(|c| c.state == ComponentState::Running)
            .collect()
    }

    /// Obtener el estado del gestor
    pub fn state(&self) -> &ComponentState {
        &self.state
    }
}

impl Default for GenodeComponentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración Genode Components
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenodeComponentError {
    ComponentNotFound,
    ComponentAlreadyExists,
    ComponentNotInitialized,
    SessionNotFound,
    SessionCreationFailed,
    DependencyNotFound,
}

impl fmt::Display for GenodeComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenodeComponentError::ComponentNotFound => write!(f, "Component not found"),
            GenodeComponentError::ComponentAlreadyExists => write!(f, "Component already exists"),
            GenodeComponentError::ComponentNotInitialized => write!(f, "Component not initialized"),
            GenodeComponentError::SessionNotFound => write!(f, "Session not found"),
            GenodeComponentError::SessionCreationFailed => write!(f, "Session creation failed"),
            GenodeComponentError::DependencyNotFound => write!(f, "Dependency not found"),
        }
    }
}
