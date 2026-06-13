//! Contenedor de Programas en Grafos para CRONOS W-OS
//!
//! Este módulo implementa el sistema de contenedor de programas usando
//! el sistema de grafos para gestión de procesos y recursos, adaptado
//! a la arquitectura de exokernel con capabilities

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, GraphStats, NodeId};

/// Estado del proceso
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    /// No iniciado
    NotStarted,
    /// Ejecutándose
    Running,
    /// Pausado
    Paused,
    /// Detenido
    Stopped,
    /// Terminado
    Terminated,
    /// Error
    Error(String),
}

/// Tipo de contenedor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerType {
    /// Contenedor estándar
    Standard,
    /// Contenedor con aislamiento completo
    Sandbox,
    /// Contenedor con recursos dedicados
    Dedicated,
    /// Contenedor con recursos compartidos
    Shared,
}

/// Configuración de recursos del contenedor
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// Memoria máxima en MB
    pub max_memory_mb: u32,
    /// Número máximo de CPUs
    pub max_cpus: u32,
    /// Espacio de disco máximo en GB
    pub max_disk_gb: u32,
    /// Ancho de banda de red máximo en Mbps
    pub max_network_mbps: u32,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpus: 2,
            max_disk_gb: 4,
            max_network_mbps: 100,
        }
    }
}

/// Configuración del contenedor
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Nombre del contenedor
    pub name: String,
    /// Tipo de contenedor
    pub container_type: ContainerType,
    /// Comando a ejecutar
    pub command: String,
    /// Argumentos del comando
    pub args: Vec<String>,
    /// Directorio de trabajo
    pub working_directory: String,
    /// Variables de entorno
    pub environment: BTreeMap<String, String>,
    /// Configuración de recursos
    pub resource_config: ResourceConfig,
    /// Habilitar acceso a red
    pub enable_network: bool,
    /// Habilitar acceso a GPU
    pub enable_gpu: bool,
}

impl ContainerConfig {
    pub fn new(name: String, command: String) -> Self {
        Self {
            name,
            container_type: ContainerType::Standard,
            command,
            args: Vec::new(),
            working_directory: String::from("/"),
            environment: BTreeMap::new(),
            resource_config: ResourceConfig::default(),
            enable_network: true,
            enable_gpu: false,
        }
    }

    pub fn with_type(mut self, container_type: ContainerType) -> Self {
        self.container_type = container_type;
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    pub fn with_resources(mut self, resource_config: ResourceConfig) -> Self {
        self.resource_config = resource_config;
        self
    }

    pub fn with_network(mut self, enable: bool) -> Self {
        self.enable_network = enable;
        self
    }

    pub fn with_gpu(mut self, enable: bool) -> Self {
        self.enable_gpu = enable;
        self
    }
}

/// Proceso dentro del contenedor
pub struct Process {
    /// ID del proceso
    pub id: u32,
    /// ID del contenedor padre
    pub container_id: u32,
    /// Estado del proceso
    pub state: ProcessState,
    /// Código de salida (si terminó)
    pub exit_code: Option<i32>,
    /// Capacidad que representa este proceso en el grafo
    pub capability: Option<CapabilityId>,
    /// Recursos utilizados
    pub resource_usage: ProcessResourceUsage,
}

/// Uso de recursos del proceso
#[derive(Debug, Clone)]
pub struct ProcessResourceUsage {
    /// Memoria usada en MB
    pub memory_mb: u32,
    /// CPU usada (%)
    pub cpu_percent: f32,
    /// Disco usado en GB
    pub disk_gb: u32,
    /// Red usada en Mbps
    pub network_mbps: u32,
}

impl Default for ProcessResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0,
            cpu_percent: 0.0,
            disk_gb: 0,
            network_mbps: 0,
        }
    }
}

/// Contenedor de programas
pub struct Container {
    /// ID del contenedor
    pub id: u32,
    /// Configuración del contenedor
    pub config: ContainerConfig,
    /// Estado del contenedor
    pub state: ProcessState,
    /// Procesos dentro del contenedor
    pub processes: Vec<Process>,
    /// Capacidad que representa este contenedor en el grafo
    pub capability: Option<CapabilityId>,
    /// Recursos utilizados por el contenedor
    pub resource_usage: ProcessResourceUsage,
}

impl Container {
    pub fn new(id: u32, config: ContainerConfig) -> Self {
        Self {
            id,
            config,
            state: ProcessState::NotStarted,
            processes: Vec::new(),
            capability: None,
            resource_usage: ProcessResourceUsage::default(),
        }
    }

    /// Iniciar el contenedor
    pub fn start(&mut self) -> Result<(), String> {
        if self.state != ProcessState::NotStarted && self.state != ProcessState::Stopped {
            return Err(format!("Contenedor no está en estado NotStarted o Stopped, estado actual: {:?}", self.state));
        }

        self.state = ProcessState::Running;

        // En un sistema real, aquí se crearía el proceso principal
        // Por ahora, simulamos la creación del proceso
        let main_process = Process {
            id: 1,
            container_id: self.id,
            state: ProcessState::Running,
            exit_code: None,
            capability: None,
            resource_usage: ProcessResourceUsage::default(),
        };

        self.processes.push(main_process);

        Ok(())
    }

    /// Detener el contenedor
    pub fn stop(&mut self) -> Result<(), String> {
        if self.state != ProcessState::Running && self.state != ProcessState::Paused {
            return Err(format!("Contenedor no está en estado Running o Paused, estado actual: {:?}", self.state));
        }

        self.state = ProcessState::Stopped;

        // Detener todos los procesos
        for process in &mut self.processes {
            process.state = ProcessState::Stopped;
            process.exit_code = Some(0);
        }

        Ok(())
    }

    /// Pausar el contenedor
    pub fn pause(&mut self) -> Result<(), String> {
        if self.state != ProcessState::Running {
            return Err(format!("Contenedor no está en estado Running, estado actual: {:?}", self.state));
        }

        self.state = ProcessState::Paused;

        // Pausar todos los procesos
        for process in &mut self.processes {
            process.state = ProcessState::Paused;
        }

        Ok(())
    }

    /// Reanudar el contenedor
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != ProcessState::Paused {
            return Err(format!("Contenedor no está en estado Paused, estado actual: {:?}", self.state));
        }

        self.state = ProcessState::Running;

        // Reanudar todos los procesos
        for process in &mut self.processes {
            process.state = ProcessState::Running;
        }

        Ok(())
    }

    /// Actualizar métricas de uso de recursos
    pub fn update_resource_usage(&mut self) {
        // En un sistema real, aquí se leerían las métricas de los procesos
        // Por ahora, usamos valores simulados
        if self.state == ProcessState::Running {
            self.resource_usage.memory_mb = self.config.resource_config.max_memory_mb / 2;
            self.resource_usage.cpu_percent = 25.0;
            self.resource_usage.disk_gb = self.config.resource_config.max_disk_gb / 4;
            self.resource_usage.network_mbps = self.config.resource_config.max_network_mbps / 10;
        } else {
            self.resource_usage = ProcessResourceUsage::default();
        }
    }

    /// FASE 29: Aplicar vista restringida (Graph Namespace)
    pub fn apply_graph_namespace(&self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if let Some(container_node) = self.capability.map(|id| NodeId(id.0)) {
            // En un modelo de grafos, el namespace se define por los nodos alcanzables.
            // Registramos el aislamiento en el nodo del contenedor.
            graph_kernel.invoke_node_operation_mut::<(), _, _>(container_node, |node| {
                node.set_metadata(String::from("isolation"), String::from("strict-graph-namespace"));
                node.set_metadata(String::from("visible_subgraph_root"), format!("{}", container_node.0));
            });
            Ok(())
        } else {
            Err(String::from("Container node not found in graph"))
        }
    }

    /// Verificar si está ejecutándose
    pub fn is_running(&self) -> bool {
        self.state == ProcessState::Running
    }

    /// Obtener el número de procesos
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
}

/// Gestor de contenedores
pub struct ContainerManager {
    /// Contenedores registrados
    pub containers: Vec<Container>,
    /// Referencia al graph kernel para registrar contenedores como nodos
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Próximo ID de contenedor
    pub next_container_id: u32,
}

impl ContainerManager {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            graph_kernel: None,
            next_container_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Crear un nuevo contenedor
    pub fn create_container(&mut self, config: ContainerConfig) -> Result<u32, String> {
        let container_id = self.next_container_id;
        self.next_container_id += 1;

        let container = Container::new(container_id, config);

        // Registrar el contenedor como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            invoke_capability_mut(&graph_kernel.capability(), |gk| {
                // En un sistema completo, aquí se crearía un nodo para el contenedor
                // con sus recursos y dependencias
            });
        }

        self.containers.push(container);
        Ok(container_id)
    }

    /// Obtener un contenedor por ID
    pub fn get_container(&self, container_id: u32) -> Option<&Container> {
        self.containers.iter().find(|c| c.id == container_id)
    }

    /// Obtener un contenedor mutable por ID
    pub fn get_container_mut(&mut self, container_id: u32) -> Option<&mut Container> {
        self.containers.iter_mut().find(|c| c.id == container_id)
    }

    /// Iniciar un contenedor
    pub fn start_container(&mut self, container_id: u32) -> Result<(), String> {
        if let Some(container) = self.get_container_mut(container_id) {
            container.start()
        } else {
            Err(format!("Contenedor con ID {} no encontrado", container_id))
        }
    }

    /// Detener un contenedor
    pub fn stop_container(&mut self, container_id: u32) -> Result<(), String> {
        if let Some(container) = self.get_container_mut(container_id) {
            container.stop()
        } else {
            Err(format!("Contenedor con ID {} no encontrado", container_id))
        }
    }

    /// Pausar un contenedor
    pub fn pause_container(&mut self, container_id: u32) -> Result<(), String> {
        if let Some(container) = self.get_container_mut(container_id) {
            container.pause()
        } else {
            Err(format!("Contenedor con ID {} no encontrado", container_id))
        }
    }

    /// Reanudar un contenedor
    pub fn resume_container(&mut self, container_id: u32) -> Result<(), String> {
        if let Some(container) = self.get_container_mut(container_id) {
            container.resume()
        } else {
            Err(format!("Contenedor con ID {} no encontrado", container_id))
        }
    }

    /// Eliminar un contenedor
    pub fn remove_container(&mut self, container_id: u32) -> Result<(), String> {
        if let Some(index) = self.containers.iter().position(|c| c.id == container_id) {
            self.containers.remove(index);
            Ok(())
        } else {
            Err(format!("Contenedor con ID {} no encontrado", container_id))
        }
    }

    /// Actualizar métricas de todos los contenedores
    pub fn update_all_metrics(&mut self) {
        for container in &mut self.containers {
            container.update_resource_usage();
        }
    }

    /// Obtener estadísticas del grafo
    pub fn get_graph_stats(&self) -> GraphStats {
        if let Some(ref graph_kernel) = self.graph_kernel {
            invoke_capability(&graph_kernel.capability(), |gk| {
                gk.get_stats()
            }).unwrap_or_default()
        } else {
            GraphStats::default()
        }
    }

    /// Obtener número de contenedores
    pub fn container_count(&self) -> usize {
        self.containers.len()
    }

    /// Obtener número de contenedores ejecutándose
    pub fn running_container_count(&self) -> usize {
        self.containers.iter().filter(|c| c.is_running()).count()
    }

    /// Listar todos los contenedores
    pub fn list_containers(&self) -> &[Container] {
        &self.containers
    }
}

impl Default for ContainerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de contenedor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerError {
    ContainerNotFound,
    ContainerAlreadyRunning,
    ContainerNotRunning,
    InvalidConfig,
    InsufficientResources,
    ProcessCreationFailed,
    ResourceLimitExceeded,
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerError::ContainerNotFound => write!(f, "Container not found"),
            ContainerError::ContainerAlreadyRunning => write!(f, "Container is already running"),
            ContainerError::ContainerNotRunning => write!(f, "Container is not running"),
            ContainerError::InvalidConfig => write!(f, "Invalid configuration"),
            ContainerError::InsufficientResources => write!(f, "Insufficient system resources"),
            ContainerError::ProcessCreationFailed => write!(f, "Process creation failed"),
            ContainerError::ResourceLimitExceeded => write!(f, "Resource limit exceeded"),
        }
    }
}
