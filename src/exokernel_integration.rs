//! Exokernel Graph Integration for CRONOS W-OS
//! 
//! This module integrates the exokernel graph concepts from F:\proyectos 2\cronos
//! with the existing capability-based architecture, adding advanced graph metrics,
//! security policies, and memory graph concepts.

use crate::graph_kernel::GraphKernel;
use crate::memory::{MemoryManager, MemoryRegion, MemoryRegionType};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;

/// Exokernel Graph Metrics (inspired by exokernel_graph.rs)
#[derive(Debug, Clone)]
pub struct ExokernelGraphMetrics {
    /// Total nodes in the graph
    pub total_nodes: u32,
    /// Total edges in the graph
    pub total_edges: u32,
    /// Graph density
    pub density: f32,
    /// Average path length
    pub average_path_length: f32,
    /// Clustering coefficient
    pub clustering_coefficient: f32,
    /// Average centrality
    pub average_centrality: f32,
}

impl Default for ExokernelGraphMetrics {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            total_edges: 0,
            density: 0.0,
            average_path_length: 0.0,
            clustering_coefficient: 0.0,
            average_centrality: 0.0,
        }
    }
}

/// Security Policy for the exokernel (inspired by exokernel_graph.rs)
#[derive(Debug, Clone)]
pub struct ExokernelSecurityPolicy {
    /// Base security level (0-10)
    pub base_security_level: u8,
    /// Mandatory isolation
    pub mandatory_isolation: bool,
    /// Integrity verification
    pub integrity_verification: bool,
    /// Event auditing
    pub event_auditing: bool,
}

impl Default for ExokernelSecurityPolicy {
    fn default() -> Self {
        Self {
            base_security_level: 5,
            mandatory_isolation: true,
            integrity_verification: true,
            event_auditing: true,
        }
    }
}

/// Memory Graph Integration (inspired by graph_memory.rs)
pub struct MemoryGraphIntegration {
    /// Memory regions as graph nodes
    pub memory_regions: BTreeMap<MemoryNodeId, MemoryRegionNode>,
    /// Memory dependencies as edges
    pub memory_dependencies: BTreeMap<MemoryEdgeId, MemoryDependency>,
    /// Memory graph metrics
    pub metrics: MemoryGraphMetrics,
}

/// Memory Node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryNodeId(pub u64);

/// Memory Edge ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryEdgeId(pub u64);

/// Memory Region as a Graph Node
#[derive(Debug, Clone)]
pub struct MemoryRegionNode {
    /// Node ID
    pub id: MemoryNodeId,
    /// Memory region
    pub region: MemoryRegion,
    /// Node type
    pub node_type: MemoryNodeType,
    /// Access patterns
    pub access_frequency: f32,
    /// Cache hit rate
    pub cache_hit_rate: f32,
}

/// Memory Node Type
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryNodeType {
    /// Physical memory
    Physical,
    /// Virtual memory
    Virtual,
    /// Shared memory
    Shared,
    /// Isolated memory
    Isolated,
    /// Kernel memory
    Kernel,
    /// User memory
    User,
    /// Device memory
    Device,
}

/// Memory Dependency Edge
#[derive(Debug, Clone)]
pub struct MemoryDependency {
    /// Edge ID
    pub id: MemoryEdgeId,
    /// Source node
    pub source: MemoryNodeId,
    /// Destination node
    pub destination: MemoryNodeId,
    /// Dependency type
    pub dependency_type: MemoryDependencyType,
    /// Weight (cost)
    pub weight: f32,
}

/// Memory Dependency Type
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryDependencyType {
    /// Mapping dependency
    Mapping,
    /// Sharing dependency
    Sharing,
    /// Isolation dependency
    Isolation,
    /// Copy dependency
    Copy,
}

/// Memory Graph Metrics
#[derive(Debug, Clone)]
pub struct MemoryGraphMetrics {
    /// Total memory nodes
    pub total_nodes: u32,
    /// Total memory edges
    pub total_edges: u32,
    /// Access efficiency
    pub access_efficiency: f32,
    /// Cache hit rate
    pub cache_hit_rate: f32,
    /// Fragmentation percentage
    pub fragmentation_percentage: f32,
}

impl Default for MemoryGraphMetrics {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            total_edges: 0,
            access_efficiency: 0.0,
            cache_hit_rate: 0.0,
            fragmentation_percentage: 0.0,
        }
    }
}

impl MemoryGraphIntegration {
    /// Create a new memory graph integration
    pub fn new() -> Self {
        Self {
            memory_regions: BTreeMap::new(),
            memory_dependencies: BTreeMap::new(),
            metrics: MemoryGraphMetrics::default(),
        }
    }

    /// Initialize memory graph from memory manager
    pub fn initialize_from_memory_manager(&mut self, memory_manager: &MemoryManager) {

        // Create memory region nodes from available memory
        let stats = memory_manager.get_memory_stats();
        
        // Create physical memory nodes
        self.create_physical_memory_nodes(stats.total_mb);
        
        // Create virtual memory nodes
        self.create_virtual_memory_nodes(stats.free_mb);

        // Create initial dependencies
        self.create_initial_dependencies();

        // Calculate metrics
        self.calculate_metrics();

    }

    /// Create physical memory nodes
    fn create_physical_memory_nodes(&mut self, total_mb: u64) {
        let regions = vec![
            ("Low Memory", 0, total_mb / 4),
            ("Medium Memory", total_mb / 4, total_mb / 2),
            ("High Memory", total_mb / 2, total_mb * 3 / 4),
            ("Reserved Memory", total_mb * 3 / 4, total_mb),
        ];

        for (i, (name, start_mb, end_mb)) in regions.iter().enumerate() {
            let node_id = MemoryNodeId(i as u64 + 1);
            
            let node = MemoryRegionNode {
                id: node_id,
                region: MemoryRegion {
                    range: crate::memory::MemoryRange {
                        start_frame_number: (start_mb * 1024 * 1024 / 4096),
                        end_frame_number: (end_mb * 1024 * 1024 / 4096),
                    },
                    region_type: MemoryRegionType::Usable,
                },
                node_type: MemoryNodeType::Physical,
                access_frequency: 0.0,
                cache_hit_rate: 0.0,
            };

            self.memory_regions.insert(node_id, node);
        }
    }

    /// Create virtual memory nodes
    fn create_virtual_memory_nodes(&mut self, free_mb: u64) {
        let regions = vec![
            ("Kernel Space", MemoryNodeType::Kernel),
            ("User Space", MemoryNodeType::User),
            ("Device Space", MemoryNodeType::Device),
        ];

        let next_id = self.memory_regions.len() as u64;

        for (i, (name, node_type)) in regions.iter().enumerate() {
            let node_id = MemoryNodeId(next_id + i as u64 + 1);
            
            let node = MemoryRegionNode {
                id: node_id,
                region: MemoryRegion {
                    range: crate::memory::MemoryRange {
                        start_frame_number: 0,
                        end_frame_number: (free_mb * 1024 * 1024 / 4096),
                    },
                    region_type: MemoryRegionType::Usable,
                },
                node_type: node_type.clone(),
                access_frequency: 0.0,
                cache_hit_rate: 0.0,
            };

            self.memory_regions.insert(node_id, node);
        }
    }

    /// Create initial dependencies
    fn create_initial_dependencies(&mut self) {
        // Connect physical memory to virtual memory
        let physical_nodes: Vec<MemoryNodeId> = self.memory_regions
            .iter()
            .filter(|(_, node)| node.node_type == MemoryNodeType::Physical)
            .map(|(id, _)| *id)
            .collect();

        let virtual_nodes: Vec<MemoryNodeId> = self.memory_regions
            .iter()
            .filter(|(_, node)| matches!(node.node_type, MemoryNodeType::Kernel | MemoryNodeType::User | MemoryNodeType::Device))
            .map(|(id, _)| *id)
            .collect();

        for (i, physical_id) in physical_nodes.iter().enumerate() {
            if i < virtual_nodes.len() {
                let virtual_id = virtual_nodes[i];
                
                let edge_id = MemoryEdgeId(self.memory_dependencies.len() as u64 + 1);
                
                let edge = MemoryDependency {
                    id: edge_id,
                    source: *physical_id,
                    destination: virtual_id,
                    dependency_type: MemoryDependencyType::Mapping,
                    weight: 1.0,
                };

                self.memory_dependencies.insert(edge_id, edge);
            }
        }
    }

    /// Calculate memory graph metrics
    fn calculate_metrics(&mut self) {
        self.metrics.total_nodes = self.memory_regions.len() as u32;
        self.metrics.total_edges = self.memory_dependencies.len() as u32;
        
        // Calculate access efficiency
        if self.metrics.total_nodes > 0 {
            self.metrics.access_efficiency = self.metrics.total_edges as f32 / self.metrics.total_nodes as f32;
        }
    }

    /// Get memory graph metrics
    pub fn get_metrics(&self) -> MemoryGraphMetrics {
        self.metrics.clone()
    }
}

/// Exokernel Integration - combines graph kernel with exokernel concepts
pub struct ExokernelIntegration {
    /// Graph kernel reference
    pub graph_kernel: GraphKernel,
    /// Exokernel metrics
    pub exokernel_metrics: ExokernelGraphMetrics,
    /// Security policy
    pub security_policy: ExokernelSecurityPolicy,
    /// Memory graph integration
    pub memory_graph: MemoryGraphIntegration,
}

impl ExokernelIntegration {
    /// Create a new exokernel integration
    pub fn new(graph_kernel: GraphKernel) -> Self {
        Self {
            graph_kernel,
            exokernel_metrics: ExokernelGraphMetrics::default(),
            security_policy: ExokernelSecurityPolicy::default(),
            memory_graph: MemoryGraphIntegration::new(),
        }
    }

    /// Initialize the exokernel integration
    pub fn initialize(&mut self, memory_manager: &MemoryManager) {

        // Initialize memory graph
        self.memory_graph.initialize_from_memory_manager(memory_manager);

        // Calculate exokernel metrics from graph kernel
        self.calculate_exokernel_metrics();

    }

    /// Calculate exokernel metrics from graph kernel
    fn calculate_exokernel_metrics(&mut self) {
        // Get node count from graph kernel
        let node_count = invoke_capability(&self.graph_kernel.graph_capability(), |graph| {
            graph.nodes.len()
        }).unwrap_or(0);
        self.exokernel_metrics.total_nodes = node_count as u32;
        
        // Get edge count from graph kernel
        let edge_count = invoke_capability(&self.graph_kernel.graph_capability(), |graph| {
            graph.edges.len()
        }).unwrap_or(0);
        self.exokernel_metrics.total_edges = edge_count as u32;
        
        // Calculate density
        if self.exokernel_metrics.total_nodes > 1 {
            let max_edges = self.exokernel_metrics.total_nodes * (self.exokernel_metrics.total_nodes - 1);
            self.exokernel_metrics.density = self.exokernel_metrics.total_edges as f32 / max_edges as f32;
        }

        // For now, set other metrics to default values
        // In a full implementation, these would be calculated from the graph topology
        self.exokernel_metrics.average_path_length = 2.0;
        self.exokernel_metrics.clustering_coefficient = 0.5;
        self.exokernel_metrics.average_centrality = 0.5;
    }

    /// Get exokernel metrics
    pub fn get_exokernel_metrics(&self) -> ExokernelGraphMetrics {
        self.exokernel_metrics.clone()
    }

    /// Get memory graph metrics
    pub fn get_memory_graph_metrics(&self) -> MemoryGraphMetrics {
        self.memory_graph.get_metrics()
    }

    /// Set security policy
    pub fn set_security_policy(&mut self, policy: ExokernelSecurityPolicy) {
        self.security_policy = policy;
    }

    /// Get security policy
    pub fn get_security_policy(&self) -> ExokernelSecurityPolicy {
        self.security_policy.clone()
    }
}
