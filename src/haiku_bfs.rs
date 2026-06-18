//! Haiku BFS Adaptation para CRONOS W-OS (VFS + Grafos)
//!
//! Este módulo adapta el sistema de archivos BFS (Be File System) de Haiku OS
//! al VFS + grafos de CRONOS W-OS, manteniendo la esencia del exokernel basado en grafos

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del nodo BFS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BfsNodeState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Activo
    Active,
    /// Bloqueado
    Blocked,
    /// Borrado
    Deleted,
    /// Error
    Error(String),
}

/// Tipo de nodo BFS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BfsNodeType {
    /// Directorio
    Directory,
    /// Archivo
    File,
    /// Enlace simbólico
    Symlink,
    /// Dispositivo de bloque
    BlockDevice,
    /// Dispositivo de carácter
    CharDevice,
    /// FIFO
    Fifo,
    /// Socket
    Socket,
}

/// Atributo extendido BFS
#[derive(Debug, Clone)]
pub struct BfsAttribute {
    /// Nombre del atributo
    pub name: String,
    /// Tipo de datos
    pub data_type: String,
    /// Datos del atributo
    pub data: Vec<u8>,
    /// Tamaño
    pub size: u64,
}

/// Nodo BFS
#[derive(Debug, Clone)]
pub struct BfsNode {
    /// ID único del nodo
    pub node_id: u64,
    /// ID del padre
    pub parent_id: Option<u64>,
    /// Nombre del nodo
    pub name: String,
    /// Tipo de nodo
    pub node_type: BfsNodeType,
    /// Estado actual
    pub state: BfsNodeState,
    /// Tamaño en bytes
    pub size: u64,
    /// Timestamp de creación
    pub created_at: u64,
    /// Timestamp de modificación
    pub modified_at: u64,
    /// Atributos extendidos
    pub attributes: Vec<BfsAttribute>,
    /// Capability del nodo
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
}

impl BfsNode {
    pub fn new(node_id: u64, name: String, node_type: BfsNodeType) -> Self {
        Self {
            node_id,
            parent_id: None,
            name,
            node_type,
            state: BfsNodeState::Uninitialized,
            size: 0,
            created_at: 0,
            modified_at: 0,
            attributes: Vec::new(),
            capability_id: None,
            graph_node_id: None,
        }
    }

    /// Inicializar el nodo en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != BfsNodeState::Uninitialized {
            return Err(format!("Nodo ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("bfs_node_{}", self.node_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = BfsNodeState::Active;
        Ok(())
    }

    /// Agregar atributo extendido
    pub fn add_attribute(&mut self, attribute: BfsAttribute) {
        self.attributes.push(attribute);
    }

    /// Obtener atributo por nombre
    pub fn get_attribute(&self, name: &str) -> Option<&BfsAttribute> {
        self.attributes.iter().find(|a| a.name == name)
    }

    /// Eliminar atributo por nombre
    pub fn remove_attribute(&mut self, name: &str) -> Option<BfsAttribute> {
        let pos = self.attributes.iter().position(|a| a.name == name)?;
        Some(self.attributes.remove(pos))
    }

    /// Obtener estado del nodo
    pub fn state(&self) -> &BfsNodeState {
        &self.state
    }
}

/// Configuración de volumen BFS
#[derive(Debug, Clone)]
pub struct BfsVolumeConfig {
    /// ID único del volumen
    pub volume_id: u64,
    /// Nombre del volumen
    pub name: String,
    /// Tamaño del volumen en bytes
    pub size: u64,
    /// Tamaño de bloque
    pub block_size: u32,
    /// Habilitar journaling
    pub enable_journaling: bool,
    /// Habilitar compresión
    pub enable_compression: bool,
}

impl BfsVolumeConfig {
    pub fn new(volume_id: u64, name: String, size: u64) -> Self {
        Self {
            volume_id,
            name,
            size,
            block_size: 4096,
            enable_journaling: true,
            enable_compression: false,
        }
    }

    pub fn with_block_size(mut self, block_size: u32) -> Self {
        self.block_size = block_size;
        self
    }
}

/// Volumen BFS
pub struct BfsVolume {
    /// Configuración del volumen
    pub config: BfsVolumeConfig,
    /// Estado del volumen
    pub state: BfsNodeState,
    /// Nodos del volumen (keyed by node_id)
    pub nodes: BTreeMap<u64, BfsNode>,
    /// Nodo raíz
    pub root_node_id: Option<u64>,
    /// Capability del volumen
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Siguiente ID de nodo
    pub next_node_id: u64,
}

impl BfsVolume {
    pub fn new(config: BfsVolumeConfig) -> Self {
        Self {
            config,
            state: BfsNodeState::Uninitialized,
            nodes: BTreeMap::new(),
            root_node_id: None,
            capability_id: None,
            graph_node_id: None,
            next_node_id: 1,
        }
    }

    /// Inicializar el volumen en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != BfsNodeState::Uninitialized {
            return Err(format!("Volumen ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("bfs_volume_{}", self.config.volume_id),
        );
        self.graph_node_id = Some(node_id);

        // Crear nodo raíz
        let mut root_node = BfsNode::new(0, String::from("/"), BfsNodeType::Directory);
        root_node.initialize(graph_kernel)?;
        self.nodes.insert(0, root_node);
        self.root_node_id = Some(0);

        self.state = BfsNodeState::Active;
        Ok(())
    }

    /// Crear un nuevo nodo
    pub fn create_node(&mut self, parent_id: u64, name: String, node_type: BfsNodeType) -> Result<u64, String> {
        if self.state != BfsNodeState::Active {
            return Err(format!("Volumen no está activo, estado actual: {:?}", self.state));
        }

        let node_id = self.next_node_id;
        let mut node = BfsNode::new(node_id, name, node_type);
        node.parent_id = Some(parent_id);

        // En un sistema real, esto inicializaría el nodo en el graph kernel
        // Por ahora, simulamos la inicialización
        node.state = BfsNodeState::Active;

        self.nodes.insert(node_id, node);
        self.next_node_id += 1;

        Ok(node_id)
    }

    /// Obtener un nodo por ID
    pub fn get_node(&self, node_id: u64) -> Option<&BfsNode> {
        self.nodes.get(&node_id)
    }

    /// Obtener un nodo mutable por ID
    pub fn get_node_mut(&mut self, node_id: u64) -> Option<&mut BfsNode> {
        self.nodes.get_mut(&node_id)
    }

    /// Eliminar un nodo
    pub fn delete_node(&mut self, node_id: u64) -> Result<(), String> {
        if let Some(node) = self.get_node_mut(node_id) {
            node.state = BfsNodeState::Deleted;
            Ok(())
        } else {
            Err(format!("Nodo con ID {} no encontrado", node_id))
        }
    }

    /// Listar nodos en un directorio
    pub fn list_directory(&self, parent_id: u64) -> Vec<&BfsNode> {
        self.nodes.values()
            .filter(|n| n.parent_id == Some(parent_id) && n.state == BfsNodeState::Active)
            .collect()
    }

    /// Obtener el nodo raíz
    pub fn get_root_node(&self) -> Option<&BfsNode> {
        self.root_node_id.and_then(|id| self.nodes.get(&id))
    }

    /// Obtener estado del volumen
    pub fn state(&self) -> &BfsNodeState {
        &self.state
    }
}

/// Gestor de BFS
#[derive(Debug, Clone)]
pub struct BfsManager {
    /// Volúmenes BFS registrados (keyed by volume_id)
    pub volumes: BTreeMap<u64, BfsVolume>,
    /// Estado del gestor
    pub state: BfsNodeState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del gestor
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de volumen
    pub next_volume_id: u64,
}

impl BfsManager {
    pub fn new() -> Self {
        Self {
            volumes: BTreeMap::new(),
            state: BfsNodeState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_volume_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = BfsNodeState::Active;
    }

    /// Crear un nuevo volumen BFS
    pub fn create_volume(&mut self, config: BfsVolumeConfig) -> Result<u64, String> {
        if self.state == BfsNodeState::Uninitialized {
            return Err(String::from("BfsManager no inicializado. Llamar a set_graph_kernel primero."));
        }

        let volume_id = config.volume_id;
        let mut volume = BfsVolume::new(config);

        // Inicializar el volumen en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                volume.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.volumes.insert(volume_id, volume);
        self.next_volume_id = volume_id + 1;

        Ok(volume_id)
    }

    /// Crear un volumen con configuración predeterminada
    pub fn create_default_volume(&mut self, name: String, size: u64) -> Result<u64, String> {
        let volume_id = self.next_volume_id;
        let config = BfsVolumeConfig::new(volume_id, name, size);
        self.create_volume(config)
    }

    /// Obtener un volumen por ID
    pub fn get_volume(&self, volume_id: u64) -> Option<&BfsVolume> {
        self.volumes.get(&volume_id)
    }

    /// Obtener un volumen mutable por ID
    pub fn get_volume_mut(&mut self, volume_id: u64) -> Option<&mut BfsVolume> {
        self.volumes.get_mut(&volume_id)
    }

    /// Crear nodo en un volumen
    pub fn create_node(&mut self, volume_id: u64, parent_id: u64, name: String, node_type: BfsNodeType) -> Result<u64, String> {
        if let Some(volume) = self.get_volume_mut(volume_id) {
            volume.create_node(parent_id, name, node_type)
        } else {
            Err(format!("Volumen con ID {} no encontrado", volume_id))
        }
    }

    /// Obtener nodo de un volumen
    pub fn get_node(&self, volume_id: u64, node_id: u64) -> Option<&BfsNode> {
        self.get_volume(volume_id).and_then(|v| v.get_node(node_id))
    }

    /// Eliminar nodo de un volumen
    pub fn delete_node(&mut self, volume_id: u64, node_id: u64) -> Result<(), String> {
        if let Some(volume) = self.get_volume_mut(volume_id) {
            volume.delete_node(node_id)
        } else {
            Err(format!("Volumen con ID {} no encontrado", volume_id))
        }
    }

    /// Listar directorio en un volumen
    pub fn list_directory(&self, volume_id: u64, parent_id: u64) -> Vec<&BfsNode> {
        self.get_volume(volume_id)
            .map(|v| v.list_directory(parent_id))
            .unwrap_or_default()
    }

    /// Obtener número de volúmenes
    pub fn volume_count(&self) -> usize {
        self.volumes.len()
    }

    /// Listar todos los volúmenes
    pub fn list_volumes(&self) -> Vec<&BfsVolume> {
        self.volumes.values().collect()
    }

    /// Obtener el estado del gestor
    pub fn state(&self) -> &BfsNodeState {
        &self.state
    }
}

impl Default for BfsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración BFS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BfsError {
    VolumeNotFound,
    VolumeAlreadyExists,
    VolumeNotInitialized,
    NodeNotFound,
    NodeAlreadyExists,
    AttributeNotFound,
    InvalidNodeType,
}

impl fmt::Display for BfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BfsError::VolumeNotFound => write!(f, "Volume not found"),
            BfsError::VolumeAlreadyExists => write!(f, "Volume already exists"),
            BfsError::VolumeNotInitialized => write!(f, "Volume not initialized"),
            BfsError::NodeNotFound => write!(f, "Node not found"),
            BfsError::NodeAlreadyExists => write!(f, "Node already exists"),
            BfsError::AttributeNotFound => write!(f, "Attribute not found"),
            BfsError::InvalidNodeType => write!(f, "Invalid node type"),
        }
    }
}
