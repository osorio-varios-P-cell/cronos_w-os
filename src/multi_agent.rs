//! Multi-Agent System para CRONOS W-OS (Hive AI)
//!
//! Este módulo implementa un sistema multi-agente en Hive AI,
//! permitiendo que múltiples agentes de IA trabajen en conjunto

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del agente
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Inactivo
    Idle,
    /// Trabajando
    Working,
    /// Esperando
    Waiting,
    /// Completado
    Completed,
    /// Error
    Error(String),
}

/// Tipo de agente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Agente de análisis
    Analyst,
    /// Agente de planificación
    Planner,
    /// Agente de ejecución
    Executor,
    /// Agente de monitoreo
    Monitor,
    /// Agente de optimización
    Optimizer,
    /// Agente de seguridad
    Security,
    /// Custom
    Custom,
}

/// Configuración de agente
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// ID único del agente
    pub agent_id: u64,
    /// Nombre del agente
    pub name: String,
    /// Tipo de agente
    pub agent_type: AgentType,
    /// Capacidad del agente
    pub capability: String,
    /// Prioridad
    pub priority: u32,
    /// Habilitado
    pub enabled: bool,
}

impl AgentConfig {
    pub fn new(agent_id: u64, name: String, agent_type: AgentType, capability: String) -> Self {
        Self {
            agent_id,
            name,
            agent_type,
            capability,
            priority: 50,
            enabled: true,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Tarea del agente
#[derive(Debug, Clone)]
pub struct AgentTask {
    /// ID único de la tarea
    pub task_id: u64,
    /// ID del agente asignado
    pub agent_id: u64,
    /// Descripción de la tarea
    pub description: String,
    /// Datos de entrada
    pub input_data: String,
    /// Datos de salida
    pub output_data: Option<String>,
    /// Estado de la tarea
    pub state: AgentState,
    /// Timestamp de creación
    pub created_at: u64,
    /// Timestamp de completado
    pub completed_at: Option<u64>,
}

/// Agente de IA
pub struct Agent {
    /// Configuración del agente
    pub config: AgentConfig,
    /// Estado actual
    pub state: AgentState,
    /// Capability del agente
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Tareas asignadas
    pub tasks: Vec<AgentTask>,
    /// Siguiente ID de tarea
    pub next_task_id: u64,
}

impl Agent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            state: AgentState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            tasks: Vec::new(),
            next_task_id: 1,
        }
    }

    /// Inicializar el agente en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != AgentState::Uninitialized {
            return Err(format!("Agente ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("agent_{}", self.config.agent_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = AgentState::Initialized;
        Ok(())
    }

    /// Asignar tarea al agente
    pub fn assign_task(&mut self, description: String, input_data: String) -> Result<u64, String> {
        if !self.config.enabled {
            return Err(String::from("Agente no está habilitado"));
        }

        let task_id = self.next_task_id;
        let task = AgentTask {
            task_id,
            agent_id: self.config.agent_id,
            description,
            input_data,
            output_data: None,
            state: AgentState::Idle,
            created_at: 0,
            completed_at: None,
        };

        self.tasks.push(task);
        self.next_task_id += 1;

        Ok(task_id)
    }

    /// Ejecutar tarea
    pub fn execute_task(&mut self, task_id: u64) -> Result<String, String> {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            task.state = AgentState::Working;

            // En un sistema real, esto ejecutaría la tarea usando la IA
            // Por ahora, simulamos la ejecución
            let output = format!("Output of task {} by agent {}", task_id, self.config.name);

            task.output_data = Some(output.clone());
            task.state = AgentState::Completed;
            task.completed_at = Some(0);

            Ok(output)
        } else {
            Err(format!("Tarea con ID {} no encontrada", task_id))
        }
    }

    /// Obtener estado del agente
    pub fn state(&self) -> &AgentState {
        &self.state
    }
}

/// Mensaje entre agentes
#[derive(Debug, Clone)]
pub struct AgentMessage {
    /// ID único del mensaje
    pub message_id: u64,
    /// ID del agente emisor
    pub from_agent_id: u64,
    /// ID del agente receptor
    pub to_agent_id: u64,
    /// Contenido del mensaje
    pub content: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Sistema Multi-Agente
pub struct MultiAgentSystem {
    /// Agentes registrados (keyed by agent_id)
    pub agents: BTreeMap<u64, Agent>,
    /// Mensajes entre agentes
    pub messages: Vec<AgentMessage>,
    /// Estado del sistema
    pub state: AgentState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del sistema
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de agente
    pub next_agent_id: u64,
    /// Siguiente ID de mensaje
    pub next_message_id: u64,
}

impl MultiAgentSystem {
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
            messages: Vec::new(),
            state: AgentState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_agent_id: 1,
            next_message_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = AgentState::Initialized;
    }

    /// Crear un nuevo agente
    pub fn create_agent(&mut self, config: AgentConfig) -> Result<u64, String> {
        if self.state == AgentState::Uninitialized {
            return Err(String::from("Multi-Agent System no inicializado. Llamar a set_graph_kernel primero."));
        }

        let agent_id = config.agent_id;
        let mut agent = Agent::new(config);

        // Inicializar el agente en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                agent.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.agents.insert(agent_id, agent);
        self.next_agent_id = agent_id + 1;

        Ok(agent_id)
    }

    /// Crear un agente con configuración predeterminada
    pub fn create_default_agent(&mut self, name: String, agent_type: AgentType, capability: String) -> Result<u64, String> {
        let agent_id = self.next_agent_id;
        let config = AgentConfig::new(agent_id, name, agent_type, capability);
        self.create_agent(config)
    }

    /// Obtener un agente por ID
    pub fn get_agent(&self, agent_id: u64) -> Option<&Agent> {
        self.agents.get(&agent_id)
    }

    /// Obtener un agente mutable por ID
    pub fn get_agent_mut(&mut self, agent_id: u64) -> Option<&mut Agent> {
        self.agents.get_mut(&agent_id)
    }

    /// Asignar tarea a un agente
    pub fn assign_task_to_agent(&mut self, agent_id: u64, description: String, input_data: String) -> Result<u64, String> {
        if let Some(agent) = self.get_agent_mut(agent_id) {
            agent.assign_task(description, input_data)
        } else {
            Err(format!("Agente con ID {} no encontrado", agent_id))
        }
    }

    /// Ejecutar tarea de un agente
    pub fn execute_agent_task(&mut self, agent_id: u64, task_id: u64) -> Result<String, String> {
        if let Some(agent) = self.get_agent_mut(agent_id) {
            agent.execute_task(task_id)
        } else {
            Err(format!("Agente con ID {} no encontrado", agent_id))
        }
    }

    /// Enviar mensaje entre agentes
    pub fn send_message(&mut self, from_agent_id: u64, to_agent_id: u64, content: String) -> Result<(), String> {
        let message = AgentMessage {
            message_id: self.next_message_id,
            from_agent_id,
            to_agent_id,
            content,
            timestamp: 0,
        };

        self.messages.push(message);
        self.next_message_id += 1;

        Ok(())
    }

    /// Obtener mensajes para un agente
    pub fn get_messages_for_agent(&self, agent_id: u64) -> Vec<&AgentMessage> {
        self.messages.iter()
            .filter(|m| m.to_agent_id == agent_id)
            .collect()
    }

    /// Ejecutar tarea colaborativa entre múltiples agentes
    pub fn execute_collaborative_task(&mut self, agent_ids: Vec<u64>, description: String, input_data: String) -> Result<Vec<String>, String> {
        let mut results = Vec::new();

        for agent_id in agent_ids {
            let task_id = self.assign_task_to_agent(agent_id, description.clone(), input_data.clone())?;
            let result = self.execute_agent_task(agent_id, task_id)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Obtener número de agentes
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Obtener número de agentes activos
    pub fn active_agent_count(&self) -> usize {
        self.agents.values().filter(|a| a.config.enabled).count()
    }

    /// Listar todos los agentes
    pub fn list_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    /// Obtener agentes por tipo
    pub fn get_agents_by_type(&self, agent_type: AgentType) -> Vec<&Agent> {
        self.agents.values()
            .filter(|a| a.config.agent_type == agent_type)
            .collect()
    }

    /// Obtener el estado del sistema
    pub fn state(&self) -> &AgentState {
        &self.state
    }
}

impl Default for MultiAgentSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del sistema multi-agente
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiAgentError {
    AgentNotFound,
    AgentAlreadyExists,
    AgentNotEnabled,
    TaskNotFound,
    TaskAlreadyAssigned,
    MessageDeliveryFailed,
    CollaborationFailed,
}

impl fmt::Display for MultiAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiAgentError::AgentNotFound => write!(f, "Agent not found"),
            MultiAgentError::AgentAlreadyExists => write!(f, "Agent already exists"),
            MultiAgentError::AgentNotEnabled => write!(f, "Agent not enabled"),
            MultiAgentError::TaskNotFound => write!(f, "Task not found"),
            MultiAgentError::TaskAlreadyAssigned => write!(f, "Task already assigned"),
            MultiAgentError::MessageDeliveryFailed => write!(f, "Message delivery failed"),
            MultiAgentError::CollaborationFailed => write!(f, "Collaboration failed"),
        }
    }
}
