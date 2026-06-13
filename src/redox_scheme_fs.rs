//! Scheme-based Filesystem de Redox (similar a Plan 9)
//!
//! Este módulo incorpora el sistema de archivos basado en esquemas de Redox OS,
//! inspirado en Plan 9, donde cada recurso es accesible a través de un esquema URI

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Esquema de URI (similar a Plan 9)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scheme {
    /// Sistema de archivos
    File,
    /// Dispositivos
    Device,
    /// Red
    Network,
    /// Memoria
    Memory,
    /// Procesos
    Process,
    /// Temporal
    Temp,
    /// Usuario
    User,
    /// Custom
    Custom(String),
}

impl Scheme {
    /// Parsear un esquema desde una string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "file" => Some(Scheme::File),
            "device" => Some(Scheme::Device),
            "network" => Some(Scheme::Network),
            "memory" => Some(Scheme::Memory),
            "process" => Some(Scheme::Process),
            "temp" => Some(Scheme::Temp),
            "user" => Some(Scheme::User),
            _ => Some(Scheme::Custom(String::from(s))),
        }
    }

    /// Convertir a string
    pub fn to_string(&self) -> String {
        match self {
            Scheme::File => String::from("file"),
            Scheme::Device => String::from("device"),
            Scheme::Network => String::from("network"),
            Scheme::Memory => String::from("memory"),
            Scheme::Process => String::from("process"),
            Scheme::Temp => String::from("temp"),
            Scheme::User => String::from("user"),
            Scheme::Custom(s) => s.clone(),
        }
    }
}

/// URI de esquema (similar a Plan 9)
#[derive(Debug, Clone)]
pub struct SchemeUri {
    pub scheme: Scheme,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl SchemeUri {
    pub fn new(scheme: Scheme, path: String) -> Self {
        Self {
            scheme,
            path,
            query: None,
            fragment: None,
        }
    }

    /// Parsear una URI desde una string
    pub fn parse(uri: &str) -> Option<Self> {
        // En un sistema real, aquí se parsearía la URI completa
        // scheme://path?query#fragment
        let parts: Vec<&str> = uri.splitn(2, "://").collect();
        if parts.len() != 2 {
            return None;
        }

        let scheme = Scheme::from_str(parts[0])?;
        let path = String::from(parts[1]);

        // Parsear query y fragment si existen
        let path_parts: Vec<&str> = path.splitn(2, '?').collect();
        let path = String::from(path_parts[0]);
        let query = if path_parts.len() > 1 {
            let query_parts: Vec<&str> = path_parts[1].splitn(2, '#').collect();
            Some(String::from(query_parts[0]))
        } else {
            None
        };

        let fragment = if path_parts.len() > 1 {
            let query_parts: Vec<&str> = path_parts[1].splitn(2, '#').collect();
            if query_parts.len() > 1 {
                Some(String::from(query_parts[1]))
            } else {
                None
            }
        } else {
            None
        };

        Some(Self {
            scheme,
            path,
            query,
            fragment,
        })
    }

    /// Convertir a string
    pub fn to_string(&self) -> String {
        let mut result = format!("{}://{}", self.scheme.to_string(), self.path);
        if let Some(ref query) = self.query {
            result.push('?');
            result.push_str(query);
        }
        if let Some(ref fragment) = self.fragment {
            result.push('#');
            result.push_str(fragment);
        }
        result
    }
}

/// Recurso de esquema
#[derive(Debug, Clone)]
pub struct SchemeResource {
    pub resource_id: u64,
    pub uri: SchemeUri,
    pub data: Vec<u8>,
    pub metadata: ResourceMetadata,
    pub permissions: ResourcePermissions,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

/// Metadata del recurso
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    pub size: u64,
    pub created_at: u64,
    pub modified_at: u64,
    pub content_type: String,
}

/// Permisos del recurso
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl ResourcePermissions {
    pub fn new(readable: bool, writable: bool, executable: bool) -> Self {
        Self {
            readable,
            writable,
            executable,
        }
    }
}

/// Gestor de filesystem basado en esquemas
pub struct SchemeFilesystem {
    pub resources: BTreeMap<u64, SchemeResource>,
    pub scheme_index: BTreeMap<String, Vec<u64>>, // scheme -> resource IDs
    pub path_index: BTreeMap<String, u64>, // path -> resource ID
    pub next_resource_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl SchemeFilesystem {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
            scheme_index: BTreeMap::new(),
            path_index: BTreeMap::new(),
            next_resource_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Registrar un recurso
    pub fn register_resource(&mut self, uri: SchemeUri, data: Vec<u8>, permissions: ResourcePermissions) -> u64 {
        let resource_id = self.next_resource_id;
        self.next_resource_id += 1;

        let metadata = ResourceMetadata {
            size: data.len() as u64,
            created_at: 0,
            modified_at: 0,
            content_type: String::from("application/octet-stream"),
        };

        let mut resource = SchemeResource {
            resource_id,
            uri: uri.clone(),
            data,
            metadata,
            permissions,
            graph_node_id: None,
        };

        // Registrar el recurso como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("scheme_resource_{}", resource_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            resource.graph_node_id = node_id;
        }

        // Indexar por esquema
        let scheme_key = uri.scheme.to_string();
        self.scheme_index.entry(scheme_key).or_insert_with(Vec::new).push(resource_id);

        // Indexar por path
        self.path_index.insert(uri.path.clone(), resource_id);

        self.resources.insert(resource_id, resource);
        resource_id
    }

    /// Obtener un recurso por URI
    pub fn get_resource_by_uri(&self, uri: &SchemeUri) -> Option<&SchemeResource> {
        self.path_index.get(&uri.path).and_then(|id| self.resources.get(id))
    }

    /// Obtener un recurso por ID
    pub fn get_resource(&self, resource_id: u64) -> Option<&SchemeResource> {
        self.resources.get(&resource_id)
    }

    /// Listar recursos por esquema
    pub fn list_by_scheme(&self, scheme: &Scheme) -> Vec<&SchemeResource> {
        let scheme_key = scheme.to_string();
        self.scheme_index.get(&scheme_key)
            .map(|ids| ids.iter().filter_map(|id| self.resources.get(id)).collect())
            .unwrap_or_default()
    }

    /// Abrir un recurso (similar a Plan 9 open)
    pub fn open(&self, uri: &SchemeUri) -> Result<u64, String> {
        if let Some(resource) = self.get_resource_by_uri(uri) {
            if resource.permissions.readable {
                Ok(resource.resource_id)
            } else {
                Err(String::from("Permission denied"))
            }
        } else {
            Err(format!("Resource not found: {}", uri.to_string()))
        }
    }

    /// Leer un recurso
    pub fn read(&self, resource_id: u64, offset: u64, buffer: &mut [u8]) -> Result<usize, String> {
        if let Some(resource) = self.resources.get(&resource_id) {
            let start = offset as usize;
            if start < resource.data.len() {
                let end = (start + buffer.len()).min(resource.data.len());
                let len = end - start;
                buffer[..len].copy_from_slice(&resource.data[start..end]);
                Ok(len)
            } else {
                Ok(0)
            }
        } else {
            Err(format!("Resource {} not found", resource_id))
        }
    }

    /// Escribir a un recurso
    pub fn write(&mut self, resource_id: u64, offset: u64, buffer: &[u8]) -> Result<usize, String> {
        if let Some(resource) = self.resources.get_mut(&resource_id) {
            if !resource.permissions.writable {
                return Err(String::from("Permission denied"));
            }

            let start = offset as usize;
            let new_len = (start + buffer.len()).max(resource.data.len());
            
            if new_len > resource.data.len() {
                resource.data.resize(new_len, 0);
            }

            resource.data[start..start + buffer.len()].copy_from_slice(buffer);
            resource.metadata.size = resource.data.len() as u64;
            resource.metadata.modified_at = 0; // En un sistema real, timestamp actual

            Ok(buffer.len())
        } else {
            Err(format!("Resource {} not found", resource_id))
        }
    }

    /// Crear un nuevo recurso
    pub fn create(&mut self, uri: SchemeUri, data: Vec<u8>, permissions: ResourcePermissions) -> Result<u64, String> {
        if self.path_index.contains_key(&uri.path) {
            return Err(format!("Resource already exists: {}", uri.to_string()));
        }

        Ok(self.register_resource(uri, data, permissions))
    }

    /// Eliminar un recurso
    pub fn remove(&mut self, resource_id: u64) -> Result<(), String> {
        if let Some(resource) = self.resources.remove(&resource_id) {
            // Remover del índice de esquema
            let scheme_key = resource.uri.scheme.to_string();
            if let Some(ids) = self.scheme_index.get_mut(&scheme_key) {
                ids.retain(|id| *id != resource_id);
            }

            // Remover del índice de path
            self.path_index.remove(&resource.uri.path);

            Ok(())
        } else {
            Err(format!("Resource {} not found", resource_id))
        }
    }

    /// Montar un esquema
    pub fn mount_scheme(&mut self, scheme: Scheme, mount_point: String) -> Result<(), String> {
        // En un sistema real, aquí se montaría un esquema en un punto de montaje
        // similar a Plan 9 mount
        Ok(())
    }

    /// Desmontar un esquema
    pub fn unmount_scheme(&mut self, scheme: Scheme) -> Result<(), String> {
        // En un sistema real, aquí se desmontaría un esquema
        Ok(())
    }

    /// Obtener estadísticas
    pub fn stats(&self) -> SchemeFSStats {
        let total_resources = self.resources.len();
        let total_size = self.resources.values().map(|r| r.metadata.size).sum();
        let scheme_count = self.scheme_index.len();

        SchemeFSStats {
            total_resources,
            total_size,
            scheme_count,
        }
    }
}

impl Default for SchemeFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del filesystem de esquemas
#[derive(Debug, Clone)]
pub struct SchemeFSStats {
    pub total_resources: usize,
    pub total_size: u64,
    pub scheme_count: usize,
}

/// Errores del filesystem de esquemas
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeFSError {
    ResourceNotFound,
    PermissionDenied,
    InvalidUri,
    SchemeNotFound,
    MountFailed,
}

impl fmt::Display for SchemeFSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeFSError::ResourceNotFound => write!(f, "Resource not found"),
            SchemeFSError::PermissionDenied => write!(f, "Permission denied"),
            SchemeFSError::InvalidUri => write!(f, "Invalid URI"),
            SchemeFSError::SchemeNotFound => write!(f, "Scheme not found"),
            SchemeFSError::MountFailed => write!(f, "Mount failed"),
        }
    }
}
