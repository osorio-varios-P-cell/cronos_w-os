//! GraphKernel - Real graph-based resource management with capabilities
//! 
//! This module implements the core graph kernel where resources are nodes,
//! operations are edges, and the graph is a live engine that can be invoked
//! through capabilities.

use crate::capability::{Capability, Cell, CapabilityId, CapabilityRights, invoke_capability, invoke_capability_mut};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Global kernel tick counter (BUG #9 corregido)
pub static KERNEL_TICK: AtomicU64 = AtomicU64::new(0);

/// Get the current kernel tick
pub fn get_kernel_tick() -> u64 {
    KERNEL_TICK.load(Ordering::SeqCst)
}

/// Increment the kernel tick
pub fn increment_kernel_tick() {
    KERNEL_TICK.fetch_add(1, Ordering::SeqCst);
}

/// Unique identifier for a graph node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        NodeId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Unique identifier for a graph edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(pub u64);

impl EdgeId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        EdgeId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Type of graph node representing different resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Kernel root node
    KernelRoot,
    /// Hardware device
    HardwareDevice(HardwareType),
    /// Memory region
    MemoryRegion,
    /// Process
    Process,
    /// Thread
    Thread,
    /// File
    File,
    /// Network interface
    NetworkInterface,
    /// GPU resource
    GpuResource,
    /// Window (for compositor)
    Window,
    /// Capability reference
    Capability,
    /// Security subject
    SecuritySubject,
    /// Security object
    SecurityObject,
    /// Virtualization resource (Hypervisor/VM/Container)
    VirtualizationResource,
    /// FASE 2.4: Knowledge Node (Second Brain - Obsidian Style)
    KnowledgeNode {
        category: String,
        tags: Vec<String>,
    },
    /// Generic resource
    Generic(String),
}

/// Hardware device types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareType {
    Cpu,
    Gpu,
    Nvme,
    Xhci,
    Wifi,
    Audio,
    Network,
    Input,
    Storage,
    Acpi,
}

/// Type of graph edge representing operations between resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    /// Data flow
    DataFlow,
    /// Control flow
    ControlFlow,
    /// Dependency
    Dependency,
    /// Ownership
    Ownership,
    /// Capability grant
    CapabilityGrant,
    /// Memory mapping
    MemoryMapping,
    /// Interrupt routing
    InterruptRouting,
    /// DMA mapping
    DmaMapping,
    /// Virtualization mapping (EPT/NPT)
    VirtualMapping,
    /// FASE 2.4: Bidirectional reference (Knowledge Graph)
    BidirectionalLink,
    /// Generic operation
    Generic(String),
}

/// A graph node representing a resource
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
    pub metadata: BTreeMap<String, String>,
    pub capabilities: BTreeSet<CapabilityId>,
    pub created_at: u64,
}

impl GraphNode {
    pub fn new(node_type: NodeType, name: String) -> Self {
        Self {
            id: NodeId::new(),
            node_type,
            name,
            metadata: BTreeMap::new(),
            capabilities: BTreeSet::new(),
            created_at: get_kernel_tick(), // BUG #9 corregido: usar KERNEL_TICK real
        }
    }

    pub fn add_capability(&mut self, cap_id: CapabilityId) {
        self.capabilities.insert(cap_id);
    }

    pub fn remove_capability(&mut self, cap_id: &CapabilityId) {
        self.capabilities.remove(cap_id);
    }

    pub fn has_capability(&self, cap_id: &CapabilityId) -> bool {
        self.capabilities.contains(cap_id)
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// A graph edge representing an operation between resources
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: EdgeType,
    pub metadata: BTreeMap<String, String>,
    pub weight: u32,
}

impl GraphEdge {
    pub fn new(source: NodeId, target: NodeId, edge_type: EdgeType) -> Self {
        Self {
            id: EdgeId::new(),
            source,
            target,
            edge_type,
            metadata: BTreeMap::new(),
            weight: 1,
        }
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// The resource graph containing all nodes and edges
#[derive(Debug, Clone)]
pub struct ResourceGraph {
    pub nodes: BTreeMap<NodeId, GraphNode>,
    pub edges: BTreeMap<EdgeId, GraphEdge>,
    pub adjacency_list: BTreeMap<NodeId, Vec<EdgeId>>,
    pub reverse_adjacency_list: BTreeMap<NodeId, Vec<EdgeId>>,
}

impl ResourceGraph {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            adjacency_list: BTreeMap::new(),
            reverse_adjacency_list: BTreeMap::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.id;
        self.nodes.insert(id, node);
        self.adjacency_list.insert(id, Vec::new());
        self.reverse_adjacency_list.insert(id, Vec::new());
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let id = edge.id;
        let source = edge.source;
        let target = edge.target;

        self.edges.insert(id, edge);
        
        self.adjacency_list
            .entry(source)
            .or_insert_with(Vec::new)
            .push(id);
        
        self.reverse_adjacency_list
            .entry(target)
            .or_insert_with(Vec::new)
            .push(id);
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<&GraphNode> {
        self.nodes.get(&node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut GraphNode> {
        self.nodes.get_mut(&node_id)
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> Option<&GraphEdge> {
        self.edges.get(&edge_id)
    }

    pub fn get_neighbors(&self, node_id: NodeId) -> Vec<NodeId> {
        self.adjacency_list
            .get(&node_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|edge_id| {
                        self.edges.get(edge_id).map(|edge| edge.target)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_predecessors(&self, node_id: NodeId) -> Vec<NodeId> {
        self.reverse_adjacency_list
            .get(&node_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|edge_id| {
                        self.edges.get(edge_id).map(|edge| edge.source)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Option<GraphNode> {
        // Remove all edges connected to this node
        let outgoing_edges: Vec<EdgeId> = self.adjacency_list
            .get(&node_id)
            .cloned()
            .unwrap_or_default();
        
        let incoming_edges: Vec<EdgeId> = self.reverse_adjacency_list
            .get(&node_id)
            .cloned()
            .unwrap_or_default();

        for edge_id in outgoing_edges.iter().chain(incoming_edges.iter()) {
            if let Some(edge) = self.edges.remove(edge_id) {
                self.adjacency_list
                    .get_mut(&edge.source)
                    .map(|edges| edges.retain(|e| e != edge_id));
                self.reverse_adjacency_list
                    .get_mut(&edge.target)
                    .map(|edges| edges.retain(|e| e != edge_id));
            }
        }

        self.adjacency_list.remove(&node_id);
        self.reverse_adjacency_list.remove(&node_id);
        self.nodes.remove(&node_id)
    }

    pub fn remove_edge(&mut self, edge_id: EdgeId) -> Option<GraphEdge> {
        if let Some(edge) = self.edges.remove(&edge_id) {
            self.adjacency_list
                .get_mut(&edge.source)
                .map(|edges| edges.retain(|e| e != &edge_id));
            self.reverse_adjacency_list
                .get_mut(&edge.target)
                .map(|edges| edges.retain(|e| e != &edge_id));
            Some(edge)
        } else {
            None
        }
    }
}

/// The GraphKernel - core of the exokernel with live graph engine
#[derive(Clone)]
pub struct GraphKernel {
    graph: Cell<ResourceGraph>,
    root_node: Option<NodeId>,
    next_node_id: NodeId,
    next_edge_id: EdgeId,
}

impl GraphKernel {
    pub fn new() -> Self {
        Self {
            graph: Cell::new(ResourceGraph::new()),
            root_node: None,
            next_node_id: NodeId(0),
            next_edge_id: EdgeId(0),
        }
    }

    /// Initialize the graph kernel
    pub fn initialize(&mut self) {
        // Create kernel root node
        let root_node = GraphNode::new(
            NodeType::KernelRoot,
            String::from("kernel_root"),
        );
        
        let root_id = root_node.id;
        
        invoke_capability_mut(&self.graph.capability(), |graph| {
            graph.add_node(root_node);
        });

        self.root_node = Some(root_id);
    }

    /// Get capability to access the graph
    pub fn graph_capability(&self) -> Capability<ResourceGraph> {
        self.graph.capability()
    }

    /// Create a new node in the graph
    pub fn create_node(&self, node_type: NodeType, name: String) -> NodeId {
        let node = GraphNode::new(node_type, name);
        let node_id = node.id;

        invoke_capability_mut(&self.graph.capability(), |graph| {
            graph.add_node(node);
        });

        node_id
    }

    /// Create a new edge in the graph
    pub fn create_edge(&self, source: NodeId, target: NodeId, edge_type: EdgeType) -> EdgeId {
        let edge = GraphEdge::new(source, target, edge_type);
        let edge_id = edge.id;

        invoke_capability_mut(&self.graph.capability(), |graph| {
            graph.add_edge(edge);
        });

        edge_id
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: NodeId) -> Option<GraphNode> {
        invoke_capability(&self.graph.capability(), |graph| {
            graph.get_node(node_id).cloned()
        }).flatten()
    }

    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: EdgeId) -> Option<GraphEdge> {
        invoke_capability(&self.graph.capability(), |graph| {
            graph.get_edge(edge_id).cloned()
        }).flatten()
    }

    /// Get neighbors of a node
    pub fn get_neighbors(&self, node_id: NodeId) -> Vec<NodeId> {
        invoke_capability(&self.graph.capability(), |graph| {
            graph.get_neighbors(node_id)
        }).unwrap_or_default()
    }

    /// Remove a node from the graph
    pub fn remove_node(&self, node_id: NodeId) -> bool {
        invoke_capability_mut(&self.graph.capability(), |graph| {
            graph.remove_node(node_id).is_some()
        }).unwrap_or(false)
    }

    /// Remove an edge from the graph
    pub fn remove_edge(&self, edge_id: EdgeId) -> bool {
        invoke_capability_mut(&self.graph.capability(), |graph| {
            graph.remove_edge(edge_id).is_some()
        }).unwrap_or(false)
    }

    /// Attach a capability to a node
    pub fn attach_capability(&self, node_id: NodeId, cap_id: CapabilityId) -> bool {
        invoke_capability_mut(&self.graph.capability(), |graph| {
            if let Some(node) = graph.get_node_mut(node_id) {
                node.add_capability(cap_id);
                true
            } else {
                false
            }
        }).unwrap_or(false)
    }

    /// Invoke an operation on a node through its capabilities
    pub fn invoke_node_operation<T, R, F>(&self, node_id: NodeId, f: F) -> Option<R>
    where
        F: FnOnce(&GraphNode) -> R,
    {
        invoke_capability(&self.graph.capability(), |graph| {
            graph.get_node(node_id).map(f)
        }).flatten()
    }

    /// Invoke a mutable operation on a node through its capabilities
    pub fn invoke_node_operation_mut<T, R, F>(&self, node_id: NodeId, f: F) -> Option<R>
    where
        F: FnOnce(&mut GraphNode) -> R,
    {
        invoke_capability_mut(&self.graph.capability(), |graph| {
            graph.get_node_mut(node_id).map(f)
        }).flatten()
    }

    /// Optimize the graph topology
    pub fn optimize_topology(&self) {
        // BUG #8 corregido: implementar optimización real del grafo
        invoke_capability_mut(&self.graph.capability(), |graph| {
            // 1. Eliminar nodos aislados (sin aristas entrantes ni salientes)
            let isolated_nodes: Vec<NodeId> = graph
                .nodes
                .iter()
                .filter(|(id, _)| {
                    graph.adjacency_list.get(id).map_or(true, |edges| edges.is_empty())
                        && graph.reverse_adjacency_list.get(id).map_or(true, |edges| edges.is_empty())
                })
                .map(|(id, _)| *id)
                .collect();

            for node_id in isolated_nodes {
                // No eliminar el nodo raíz
                if Some(node_id) != self.root_node {
                    graph.remove_node(node_id);
                }
            }

            // 2. Eliminar aristas redundantes (transitividad)
            // Si A->B y B->C con el mismo tipo, y A->C existe, eliminar A->C si es redundante
            let redundant_edges: Vec<EdgeId> = graph
                .edges
                .iter()
                .filter(|(_, edge)| {
                    // Para cada arista A->C, verificar si existe A->B->C con el mismo tipo
                    let a = edge.source;
                    let c = edge.target;
                    let edge_type = &edge.edge_type;

                    // Obtener vecinos de A
                    if let Some(b_neighbors) = graph.adjacency_list.get(&a) {
                        for b_edge_id in b_neighbors {
                            if let Some(b_edge) = graph.edges.get(b_edge_id) {
                                let b = b_edge.target;
                                // Verificar si B->C existe con el mismo tipo
                                if let Some(c_neighbors) = graph.adjacency_list.get(&b) {
                                    for c_edge_id in c_neighbors {
                                        if let Some(c_edge) = graph.edges.get(c_edge_id) {
                                            if c_edge.target == c && &c_edge.edge_type == edge_type {
                                                // A->B->C existe, A->C es redundante
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    false
                })
                .map(|(id, _)| *id)
                .collect();

            for edge_id in redundant_edges {
                graph.remove_edge(edge_id);
            }

            // 3. Fusionar nodos duplicados (mismo tipo y nombre)
            let mut nodes_to_remove: Vec<NodeId> = Vec::new();
            let mut processed: BTreeSet<NodeId> = BTreeSet::new();
            
            for (node_id_a, node_a) in &graph.nodes {
                if processed.contains(node_id_a) {
                    continue;
                }
                processed.insert(*node_id_a);
                
                for (node_id_b, node_b) in &graph.nodes {
                    if *node_id_a == *node_id_b || processed.contains(node_id_b) {
                        continue;
                    }
                    
                    // Comparar tipo y nombre
                    if node_a.node_type == node_b.node_type && node_a.name == node_b.name {
                        // Fusionar node_b en node_a
                        let incoming: Vec<EdgeId> = graph
                            .reverse_adjacency_list
                            .get(node_id_b)
                            .cloned()
                            .unwrap_or_default();
                        
                        let outgoing: Vec<EdgeId> = graph
                            .adjacency_list
                            .get(node_id_b)
                            .cloned()
                            .unwrap_or_default();

                        // Redirigir aristas entrantes
                        for edge_id in incoming {
                            if let Some(edge) = graph.edges.get_mut(&edge_id) {
                                edge.target = *node_id_a;
                                graph.adjacency_list
                                    .entry(edge.source)
                                    .or_insert_with(Vec::new)
                                    .push(edge_id);
                            }
                        }

                        // Redirigir aristas salientes
                        for edge_id in outgoing {
                            if let Some(edge) = graph.edges.get_mut(&edge_id) {
                                edge.source = *node_id_a;
                                graph.reverse_adjacency_list
                                    .entry(edge.target)
                                    .or_insert_with(Vec::new)
                                    .push(edge_id);
                            }
                        }

                        nodes_to_remove.push(*node_id_b);
                        processed.insert(*node_id_b);
                    }
                }
            }

            // Eliminar nodos fusionados
            for node_id in nodes_to_remove {
                graph.remove_node(node_id);
            }
        });
    }

    /// Get the root node ID
    pub fn root_node(&self) -> Option<NodeId> {
        self.root_node
    }

    /// Get graph statistics
    pub fn get_stats(&self) -> GraphStats {
        invoke_capability(&self.graph.capability(), |graph| {
            GraphStats {
                node_count: graph.nodes.len(),
                edge_count: graph.edges.len(),
                isolated_nodes: graph
                    .nodes
                    .iter()
                    .filter(|(id, _)| {
                        graph.adjacency_list.get(id).map_or(true, |edges| edges.is_empty())
                            && graph.reverse_adjacency_list.get(id).map_or(true, |edges| edges.is_empty())
                    })
                    .count(),
            }
        }).unwrap_or_default()
    }
}

/// Graph statistics
#[derive(Debug, Default, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub isolated_nodes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_kernel_initialization() {
        let mut kernel = GraphKernel::new();
        kernel.initialize();
        assert!(kernel.root_node().is_some());
    }

    #[test]
    fn test_node_creation() {
        let kernel = GraphKernel::new();
        let node_id = kernel.create_node(
            NodeType::HardwareDevice(HardwareType::Cpu),
            String::from("cpu0"),
        );
        let node = kernel.get_node(node_id);
        assert!(node.is_some());
    }

    #[test]
    fn test_edge_creation() {
        let kernel = GraphKernel::new();
        let node1 = kernel.create_node(NodeType::MemoryRegion, String::from("mem1"));
        let node2 = kernel.create_node(NodeType::Process, String::from("proc1"));
        let edge_id = kernel.create_edge(node1, node2, EdgeType::DataFlow);
        let edge = kernel.get_edge(edge_id);
        assert!(edge.is_some());
    }

    #[test]
    fn test_neighbors() {
        let kernel = GraphKernel::new();
        let node1 = kernel.create_node(NodeType::MemoryRegion, String::from("mem1"));
        let node2 = kernel.create_node(NodeType::Process, String::from("proc1"));
        kernel.create_edge(node1, node2, EdgeType::DataFlow);
        let neighbors = kernel.get_neighbors(node1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], node2);
    }
}
