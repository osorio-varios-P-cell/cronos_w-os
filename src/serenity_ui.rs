//! SerenityOS UI Adaptation para CRONOS W-OS (Lumen Layer)
//!
//! Este módulo adapta la UI moderna de SerenityOS a la capa Lumen de CRONOS W-OS,
//! manteniendo la esencia del exokernel basado en grafos y las 4 capas

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del componente UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiComponentState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Visible
    Visible,
    /// Oculto
    Hidden,
    /// Minimizado
    Minimized,
    /// Maximizado
    Maximized,
    /// Cerrado
    Closed,
    /// Error
    Error(String),
}

/// Tipo de componente UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiComponentType {
    /// Ventana
    Window,
    /// Botón
    Button,
    /// Etiqueta
    Label,
    /// Campo de texto
    TextField,
    /// Lista
    List,
    /// Menú
    Menu,
    /// Barra de herramientas
    Toolbar,
    /// Barra de estado
    StatusBar,
    /// Panel
    Panel,
    /// Custom
    Custom,
}

/// Evento UI
#[derive(Debug, Clone)]
pub struct UiEvent {
    /// ID único del evento
    pub event_id: u64,
    /// Tipo de evento
    pub event_type: String,
    /// ID del componente origen
    pub source_component_id: u64,
    /// Datos del evento
    pub event_data: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Configuración de componente UI
#[derive(Debug, Clone)]
pub struct UiComponentConfig {
    /// ID único del componente
    pub component_id: u64,
    /// Nombre del componente
    pub name: String,
    /// Tipo de componente
    pub component_type: UiComponentType,
    /// Título
    pub title: String,
    /// Posición X
    pub x: i32,
    /// Posición Y
    pub y: i32,
    /// Ancho
    pub width: u32,
    /// Alto
    pub height: u32,
    /// Visible
    pub visible: bool,
    /// Habilitado
    pub enabled: bool,
}

impl UiComponentConfig {
    pub fn new(component_id: u64, name: String, component_type: UiComponentType, title: String) -> Self {
        Self {
            component_id,
            name,
            component_type,
            title,
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            visible: true,
            enabled: true,
        }
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

/// Componente UI de SerenityOS
pub struct SerenityUiComponent {
    /// Configuración del componente
    pub config: UiComponentConfig,
    /// Estado actual
    pub state: UiComponentState,
    /// Capability del componente
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Eventos pendientes
    pub pending_events: Vec<UiEvent>,
    /// Componentes hijos
    pub children: Vec<u64>,
}

impl SerenityUiComponent {
    pub fn new(config: UiComponentConfig) -> Self {
        Self {
            config,
            state: UiComponentState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            pending_events: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Inicializar el componente en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != UiComponentState::Uninitialized {
            return Err(format!("Componente ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("ui_component_{}", self.config.component_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = UiComponentState::Initialized;
        Ok(())
    }

    /// Mostrar el componente
    pub fn show(&mut self) -> Result<(), String> {
        if !self.config.visible {
            self.config.visible = true;
        }
        self.state = UiComponentState::Visible;
        Ok(())
    }

    /// Ocultar el componente
    pub fn hide(&mut self) -> Result<(), String> {
        if self.config.visible {
            self.config.visible = false;
        }
        self.state = UiComponentState::Hidden;
        Ok(())
    }

    /// Minimizar el componente
    pub fn minimize(&mut self) -> Result<(), String> {
        self.state = UiComponentState::Minimized;
        Ok(())
    }

    /// Maximizar el componente
    pub fn maximize(&mut self) -> Result<(), String> {
        self.state = UiComponentState::Maximized;
        Ok(())
    }

    /// Cerrar el componente
    pub fn close(&mut self) -> Result<(), String> {
        self.state = UiComponentState::Closed;
        Ok(())
    }

    /// Agregar evento pendiente
    pub fn add_event(&mut self, event: UiEvent) {
        self.pending_events.push(event);
    }

    /// Obtener eventos pendientes
    pub fn get_events(&mut self) -> Vec<UiEvent> {
        let events = self.pending_events.clone();
        self.pending_events.clear();
        events
    }

    /// Agregar componente hijo
    pub fn add_child(&mut self, child_id: u64) {
        self.children.push(child_id);
    }

    /// Obtener estado del componente
    pub fn state(&self) -> &UiComponentState {
        &self.state
    }
}

/// Gestor de UI de SerenityOS
pub struct SerenityUiManager {
    /// Componentes UI registrados (keyed by component_id)
    pub components: BTreeMap<u64, SerenityUiComponent>,
    /// Componentes raíz (ventanas)
    pub root_components: Vec<u64>,
    /// Estado del gestor
    pub state: UiComponentState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del gestor
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de componente
    pub next_component_id: u64,
    /// Siguiente ID de evento
    pub next_event_id: u64,
}

impl SerenityUiManager {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            root_components: Vec::new(),
            state: UiComponentState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_component_id: 1,
            next_event_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = UiComponentState::Initialized;
    }

    /// Crear un nuevo componente UI
    pub fn create_component(&mut self, config: UiComponentConfig) -> Result<u64, String> {
        if self.state == UiComponentState::Uninitialized {
            return Err(String::from("SerenityUiManager no inicializado. Llamar a set_graph_kernel primero."));
        }

        let component_id = config.component_id;
        let mut component = SerenityUiComponent::new(config);

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

        // Si es una ventana, agregar a componentes raíz
        if component.config.component_type == UiComponentType::Window {
            self.root_components.push(component_id);
        }

        self.components.insert(component_id, component);
        self.next_component_id = component_id + 1;

        Ok(component_id)
    }

    /// Crear un componente con configuración predeterminada
    pub fn create_default_component(&mut self, name: String, component_type: UiComponentType, title: String) -> Result<u64, String> {
        let component_id = self.next_component_id;
        let config = UiComponentConfig::new(component_id, name, component_type, title);
        self.create_component(config)
    }

    /// Obtener un componente por ID
    pub fn get_component(&self, component_id: u64) -> Option<&SerenityUiComponent> {
        self.components.get(&component_id)
    }

    /// Obtener un componente mutable por ID
    pub fn get_component_mut(&mut self, component_id: u64) -> Option<&mut SerenityUiComponent> {
        self.components.get_mut(&component_id)
    }

    /// Mostrar componente
    pub fn show_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.show()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Ocultar componente
    pub fn hide_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.hide()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Minimizar componente
    pub fn minimize_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.minimize()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Maximizar componente
    pub fn maximize_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.maximize()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Cerrar componente
    pub fn close_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.close()
        } else {
            Err(format!("Componente con ID {} no encontrado", component_id))
        }
    }

    /// Crear evento UI
    pub fn create_event(&mut self, source_component_id: u64, event_type: String, event_data: String) -> Result<u64, String> {
        let event_id = self.next_event_id;
        let event = UiEvent {
            event_id,
            event_type,
            source_component_id,
            event_data,
            timestamp: 0,
        };

        if let Some(component) = self.get_component_mut(source_component_id) {
            component.add_event(event);
        }

        self.next_event_id += 1;
        Ok(event_id)
    }

    /// Procesar eventos de un componente
    pub fn process_events(&mut self, component_id: u64) -> Vec<UiEvent> {
        if let Some(component) = self.get_component_mut(component_id) {
            component.get_events()
        } else {
            Vec::new()
        }
    }

    /// Obtener número de componentes
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Listar todos los componentes
    pub fn list_components(&self) -> Vec<&SerenityUiComponent> {
        self.components.values().collect()
    }

    /// Obtener componentes por tipo
    pub fn get_components_by_type(&self, component_type: UiComponentType) -> Vec<&SerenityUiComponent> {
        self.components.values()
            .filter(|c| c.config.component_type == component_type)
            .collect()
    }

    /// Obtener componentes raíz (ventanas)
    pub fn get_root_components(&self) -> Vec<&SerenityUiComponent> {
        self.root_components.iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    /// Obtener el estado del gestor
    pub fn state(&self) -> &UiComponentState {
        &self.state
    }
}

impl Default for SerenityUiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración Serenity UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerenityUiError {
    ComponentNotFound,
    ComponentAlreadyExists,
    ComponentNotInitialized,
    EventCreationFailed,
    EventProcessingFailed,
}

impl fmt::Display for SerenityUiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerenityUiError::ComponentNotFound => write!(f, "Component not found"),
            SerenityUiError::ComponentAlreadyExists => write!(f, "Component already exists"),
            SerenityUiError::ComponentNotInitialized => write!(f, "Component not initialized"),
            SerenityUiError::EventCreationFailed => write!(f, "Event creation failed"),
            SerenityUiError::EventProcessingFailed => write!(f, "Event processing failed"),
        }
    }
}
