//! Live Evolution de Theseus (Hot Reloading de Componentes)
//!
//! Este módulo incorpora las técnicas de live evolution de Theseus OS
//! para permitir actualizar componentes en tiempo real sin reiniciar CRONOS W-OS

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Estado de un componente
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentState {
    /// No inicializado
    Uninitialized,
    /// Inicializando
    Initializing,
    /// Activo
    Active,
    /// Actualizando
    Updating,
    /// Pausado
    Paused,
    /// Error
    Error(String),
}

/// Versión de un componente
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ComponentVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Comparar versiones
    pub fn compare(&self, other: &ComponentVersion) -> core::cmp::Ordering {
        if self.major != other.major {
            self.major.cmp(&other.major)
        } else if self.minor != other.minor {
            self.minor.cmp(&other.minor)
        } else {
            self.patch.cmp(&other.patch)
        }
    }
}

impl fmt::Display for ComponentVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Componente del sistema
#[derive(Debug, Clone)]
pub struct SystemComponent {
    pub component_id: u64,
    pub name: String,
    pub version: ComponentVersion,
    pub state: ComponentState,
    pub dependencies: BTreeSet<u64>, // IDs de componentes dependientes
    pub dependents: BTreeSet<u64>, // IDs de componentes que dependen de este
    pub capability_id: Option<CapabilityId>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
    pub last_update: u64,
}

impl SystemComponent {
    pub fn new(component_id: u64, name: String, version: ComponentVersion) -> Self {
        Self {
            component_id,
            name,
            version,
            state: ComponentState::Uninitialized,
            dependencies: BTreeSet::new(),
            dependents: BTreeSet::new(),
            capability_id: None,
            graph_node_id: None,
            last_update: 0,
        }
    }

    /// Verificar si el componente puede ser actualizado
    pub fn can_update(&self) -> bool {
        matches!(self.state, ComponentState::Active | ComponentState::Paused)
    }

    /// Verificar si el componente tiene dependencias activas
    pub fn has_active_dependencies(&self, components: &BTreeMap<u64, SystemComponent>) -> bool {
        self.dependencies.iter().any(|dep_id| {
            components.get(dep_id)
                .map(|c| matches!(c.state, ComponentState::Active))
                .unwrap_or(false)
        })
    }
}

/// Gestor de live evolution
pub struct LiveEvolutionManager {
    pub components: BTreeMap<u64, SystemComponent>,
    pub next_component_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
    pub update_queue: Vec<u64>,
    pub rollback_stack: Vec<(u64, ComponentVersion)>,
    pub auto_rollback_enabled: bool,
}

impl LiveEvolutionManager {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            next_component_id: 1,
            graph_kernel: None,
            update_queue: Vec::new(),
            rollback_stack: Vec::new(),
            auto_rollback_enabled: true,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Registrar un componente
    pub fn register_component(&mut self, name: String, version: ComponentVersion) -> u64 {
        let component_id = self.next_component_id;
        self.next_component_id += 1;

        let mut component = SystemComponent::new(component_id, name, version);

        // Crear capability para el componente
        let capability_id = CapabilityId::new();
        component.capability_id = Some(capability_id);

        // Registrar el componente como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::Process;
            let node_name = format!("live_component_{}", component_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            component.graph_node_id = node_id;
        }

        self.components.insert(component_id, component);
        component_id
    }

    /// Agregar una dependencia entre componentes
    pub fn add_dependency(&mut self, component_id: u64, dependency_id: u64) -> Result<(), String> {
        // Verificar que ambos componentes existan
        if !self.components.contains_key(&component_id) {
            return Err(format!("Component {} not found", component_id));
        }
        if !self.components.contains_key(&dependency_id) {
            return Err(format!("Dependency {} not found", dependency_id));
        }

        // Agregar la dependencia
        self.components.get_mut(&component_id).unwrap().dependencies.insert(dependency_id);
        self.components.get_mut(&dependency_id).unwrap().dependents.insert(component_id);

        Ok(())
    }

    /// Inicializar un componente
    pub fn initialize_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.components.get_mut(&component_id) {
            component.state = ComponentState::Initializing;

            // En un sistema real, aquí se inicializaría el componente
            // verificando que todas las dependencias estén activas

            component.state = ComponentState::Active;
            Ok(())
        } else {
            Err(format!("Component {} not found", component_id))
        }
    }

    /// Actualizar un componente (hot reload)
    pub fn update_component(&mut self, component_id: u64, new_version: ComponentVersion) -> Result<(), String> {
        if let Some(component) = self.components.get_mut(&component_id) {
            if !component.can_update() {
                return Err(format!("Component {} cannot be updated in current state", component_id));
            }

            // Guardar la versión anterior para rollback
            self.rollback_stack.push((component_id, component.version.clone()));

            component.state = ComponentState::Updating;
            component.version = new_version;
            component.last_update = 0; // En un sistema real, timestamp actual

            // En un sistema real, aquí se:
            // 1. Cargaría la nueva versión del componente
            // 2. Migraría el estado del componente
            // 3. Verificaría que las dependencias sean compatibles
            // 4. Notificaría a los componentes dependientes

            component.state = ComponentState::Active;
            Ok(())
        } else {
            Err(format!("Component {} not found", component_id))
        }
    }

    /// Pausar un componente
    pub fn pause_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.components.get_mut(&component_id) {
            if matches!(component.state, ComponentState::Active) {
                component.state = ComponentState::Paused;
                Ok(())
            } else {
                Err(format!("Component {} is not active", component_id))
            }
        } else {
            Err(format!("Component {} not found", component_id))
        }
    }

    /// Reanudar un componente
    pub fn resume_component(&mut self, component_id: u64) -> Result<(), String> {
        if let Some(component) = self.components.get_mut(&component_id) {
            if matches!(component.state, ComponentState::Paused) {
                component.state = ComponentState::Active;
                Ok(())
            } else {
                Err(format!("Component {} is not paused", component_id))
            }
        } else {
            Err(format!("Component {} not found", component_id))
        }
    }

    /// Rollback a una versión anterior
    pub fn rollback_component(&mut self, component_id: u64) -> Result<(), String> {
        // Buscar la versión anterior en el stack
        let rollback_index = self.rollback_stack.iter().position(|(id, _)| *id == component_id);

        if let Some(index) = rollback_index {
            let (_, old_version) = self.rollback_stack.remove(index);

            if let Some(component) = self.components.get_mut(&component_id) {
                component.state = ComponentState::Updating;
                component.version = old_version;
                component.state = ComponentState::Active;
                Ok(())
            } else {
                Err(format!("Component {} not found", component_id))
            }
        } else {
            Err(format!("No previous version found for component {}", component_id))
        }
    }

    /// Obtener un componente
    pub fn get_component(&self, component_id: u64) -> Option<&SystemComponent> {
        self.components.get(&component_id)
    }

    /// Obtener componentes por nombre
    pub fn get_components_by_name(&self, name: &str) -> Vec<&SystemComponent> {
        self.components.values()
            .filter(|c| c.name == name)
            .collect()
    }

    /// Obtener componentes dependientes
    pub fn get_dependents(&self, component_id: u64) -> Vec<&SystemComponent> {
        if let Some(component) = self.components.get(&component_id) {
            component.dependents.iter()
                .filter_map(|id| self.components.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Verificar si una actualización es segura
    pub fn is_update_safe(&self, component_id: u64, new_version: &ComponentVersion) -> bool {
        if let Some(component) = self.components.get(&component_id) {
            // Verificar que la nueva versión sea mayor
            if component.version.compare(new_version) != core::cmp::Ordering::Less {
                return false;
            }

            // Verificar que los componentes dependientes no se vean afectados
            for dependent_id in &component.dependents {
                if let Some(dependent) = self.components.get(dependent_id) {
                    // En un sistema real, aquí se verificaría la compatibilidad
                    // de las APIs entre versiones
                }
            }

            true
        } else {
            false
        }
    }

    /// Habilitar/deshabilitar auto-rollback
    pub fn set_auto_rollback(&mut self, enabled: bool) {
        self.auto_rollback_enabled = enabled;
    }

    /// Obtener estadísticas
    pub fn stats(&self) -> EvolutionStats {
        let total = self.components.len();
        let active = self.components.values().filter(|c| matches!(c.state, ComponentState::Active)).count();
        let updating = self.components.values().filter(|c| matches!(c.state, ComponentState::Updating)).count();
        let paused = self.components.values().filter(|c| matches!(c.state, ComponentState::Paused)).count();
        let error = self.components.values().filter(|c| matches!(c.state, ComponentState::Error(_))).count();

        EvolutionStats {
            total_components: total,
            active_components: active,
            updating_components: updating,
            paused_components: paused,
            error_components: error,
        }
    }
}

impl Default for LiveEvolutionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de live evolution
#[derive(Debug, Clone)]
pub struct EvolutionStats {
    pub total_components: usize,
    pub active_components: usize,
    pub updating_components: usize,
    pub paused_components: usize,
    pub error_components: usize,
}

/// Errores de live evolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveEvolutionError {
    ComponentNotFound,
    InvalidVersion,
    DependencyConflict,
    UpdateInProgress,
    RollbackFailed,
}

impl fmt::Display for LiveEvolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiveEvolutionError::ComponentNotFound => write!(f, "Component not found"),
            LiveEvolutionError::InvalidVersion => write!(f, "Invalid version"),
            LiveEvolutionError::DependencyConflict => write!(f, "Dependency conflict"),
            LiveEvolutionError::UpdateInProgress => write!(f, "Update in progress"),
            LiveEvolutionError::RollbackFailed => write!(f, "Rollback failed"),
        }
    }
}
