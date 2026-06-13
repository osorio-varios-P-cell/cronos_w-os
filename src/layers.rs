//! 4-Layer Architecture - AEGIS, LUMEN, GENESIS, Kernel
//! 
//! This module organizes the system into the 4 core layers of CRONOS:
//! - AEGIS: Security layer
//! - LUMEN: Graphics layer
//! - GENESIS: Auto-creation layer
//! - Kernel: Core exokernel layer
//! 
//! Each layer is a sub-graph with its own capabilities.

use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType, EdgeId};
use crate::capability::{Capability, Cell, CapabilityRights, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::compositor::Compositor;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

/// System layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    /// Kernel layer - Core exokernel with graph management
    Kernel,
    /// AEGIS layer - Security and isolation
    Aegis,
    /// LUMEN layer - Graphics and compositor
    Lumen,
    /// GENESIS layer - Auto-creation and optimization
    Genesis,
}

impl Layer {
    pub fn name(&self) -> &str {
        match self {
            Layer::Kernel => "KERNEL",
            Layer::Aegis => "AEGIS",
            Layer::Lumen => "LUMEN",
            Layer::Genesis => "GENESIS",
        }
    }
}

/// Layer-specific capability
#[derive(Debug)]
pub struct LayerCapability {
    layer: Layer,
    capability_id: CapabilityId,
    rights: CapabilityRights,
}

impl LayerCapability {
    pub fn new(layer: Layer, capability_id: CapabilityId, rights: CapabilityRights) -> Self {
        Self {
            layer,
            capability_id,
            rights,
        }
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn capability_id(&self) -> CapabilityId {
        self.capability_id
    }

    pub fn rights(&self) -> CapabilityRights {
        self.rights
    }
}

/// A single layer in the 4-layer architecture
#[derive(Debug, Clone)]
pub struct SystemLayer {
    pub layer_type: Layer,
    pub root_node: Option<NodeId>,
    pub capabilities: BTreeSet<CapabilityId>,
    pub child_nodes: BTreeSet<NodeId>,
    pub metadata: BTreeMap<String, String>,
}

impl SystemLayer {
    pub fn new(layer_type: Layer) -> Self {
        Self {
            layer_type,
            root_node: None,
            capabilities: BTreeSet::new(),
            child_nodes: BTreeSet::new(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn set_root_node(&mut self, node_id: NodeId) {
        self.root_node = Some(node_id);
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

    pub fn add_child_node(&mut self, node_id: NodeId) {
        self.child_nodes.insert(node_id);
    }

    pub fn remove_child_node(&mut self, node_id: &NodeId) {
        self.child_nodes.remove(node_id);
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// The 4-layer architecture manager
#[derive(Clone)]
pub struct LayerArchitecture {
    graph_kernel: GraphKernel,
    layers: BTreeMap<Layer, SystemLayer>,
    layer_edges: BTreeMap<(Layer, Layer), EdgeId>,
    bridge_capabilities: BTreeMap<CapabilityId, (Layer, Layer)>,
}

impl LayerArchitecture {
    pub fn new(graph_kernel: GraphKernel) -> Self {
        let mut layers = BTreeMap::new();
        layers.insert(Layer::Kernel, SystemLayer::new(Layer::Kernel));
        layers.insert(Layer::Aegis, SystemLayer::new(Layer::Aegis));
        layers.insert(Layer::Lumen, SystemLayer::new(Layer::Lumen));
        layers.insert(Layer::Genesis, SystemLayer::new(Layer::Genesis));

        Self {
            graph_kernel,
            layers,
            layer_edges: BTreeMap::new(),
            bridge_capabilities: BTreeMap::new(),
        }
    }

    /// Initialize the 4-layer architecture
    pub fn initialize(&mut self) {
        // Create root nodes for each layer
        self.create_layer_root(Layer::Kernel, String::from("kernel_layer"));
        self.create_layer_root(Layer::Aegis, String::from("aegis_layer"));
        self.create_layer_root(Layer::Lumen, String::from("lumen_layer"));
        self.create_layer_root(Layer::Genesis, String::from("genesis_layer"));

        // Connect layers in hierarchy
        self.connect_layers(Layer::Kernel, Layer::Aegis);
        self.connect_layers(Layer::Kernel, Layer::Lumen);
        self.connect_layers(Layer::Kernel, Layer::Genesis);
        self.connect_layers(Layer::Aegis, Layer::Lumen);
        self.connect_layers(Layer::Aegis, Layer::Genesis);
        self.connect_layers(Layer::Lumen, Layer::Genesis);
    }

    /// Create root node for a layer
    fn create_layer_root(&mut self, layer: Layer, name: String) {
        let node_id = self.graph_kernel.create_node(
            NodeType::Generic(name.clone()),
            name,
        );

        if let Some(system_layer) = self.layers.get_mut(&layer) {
            system_layer.set_root_node(node_id);
        }
    }

    /// Connect two layers with an edge
    fn connect_layers(&mut self, from: Layer, to: Layer) {
        if let (Some(from_layer), Some(to_layer)) = (
            self.layers.get(&from).and_then(|l| l.root_node),
            self.layers.get(&to).and_then(|l| l.root_node),
        ) {
            let edge_id = self.graph_kernel.create_edge(
                from_layer,
                to_layer,
                EdgeType::ControlFlow,
            );
            self.layer_edges.insert((from, to), edge_id);
        }
    }

    /// Get a layer by type
    pub fn get_layer(&self, layer: Layer) -> Option<&SystemLayer> {
        self.layers.get(&layer)
    }

    /// Get a mutable layer by type
    pub fn get_layer_mut(&mut self, layer: Layer) -> Option<&mut SystemLayer> {
        self.layers.get_mut(&layer)
    }

    /// Add a node to a layer
    pub fn add_node_to_layer(&mut self, layer: Layer, node_id: NodeId) -> bool {
        if let Some(system_layer) = self.layers.get_mut(&layer) {
            if let Some(root_node) = system_layer.root_node {
                // Connect to layer root
                self.graph_kernel.create_edge(root_node, node_id, EdgeType::Ownership);
                system_layer.add_child_node(node_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Add a capability to a layer
    pub fn add_capability_to_layer(&mut self, layer: Layer, cap_id: CapabilityId) -> bool {
        if let Some(system_layer) = self.layers.get_mut(&layer) {
            system_layer.add_capability(cap_id);
            true
        } else {
            false
        }
    }

    /// Create a bridge capability between layers
    pub fn create_bridge_capability(&mut self, from: Layer, to: Layer, cap_id: CapabilityId) {
        self.bridge_capabilities.insert(cap_id, (from, to));
        
        // Add capability to both layers
        self.add_capability_to_layer(from, cap_id);
        self.add_capability_to_layer(to, cap_id);
    }

    /// Check if a capability can bridge between layers
    pub fn can_bridge(&self, cap_id: CapabilityId, from: Layer, to: Layer) -> bool {
        self.bridge_capabilities
            .get(&cap_id)
            .map(|(f, t)| *f == from && *t == to)
            .unwrap_or(false)
    }

    /// Get the graph kernel
    pub fn graph_kernel(&self) -> &GraphKernel {
        &self.graph_kernel
    }

    /// Get mutable graph kernel
    pub fn graph_kernel_mut(&mut self) -> &mut GraphKernel {
        &mut self.graph_kernel
    }

    /// Get all layer root nodes
    pub fn layer_roots(&self) -> BTreeMap<Layer, NodeId> {
        self.layers
            .iter()
            .filter_map(|(layer, system_layer)| {
                system_layer.root_node.map(|node_id| (*layer, node_id))
            })
            .collect()
    }

    /// Get layer statistics
    pub fn layer_stats(&self) -> BTreeMap<Layer, LayerStats> {
        self.layers
            .iter()
            .map(|(layer, system_layer)| {
                let stats = LayerStats {
                    node_count: system_layer.child_nodes.len(),
                    capability_count: system_layer.capabilities.len(),
                    has_root: system_layer.root_node.is_some(),
                };
                (*layer, stats)
            })
            .collect()
    }
}

/// Layer statistics
#[derive(Debug, Clone, Default)]
pub struct LayerStats {
    pub node_count: usize,
    pub capability_count: usize,
    pub has_root: bool,
}

/// Kernel layer implementation
pub struct KernelLayer {
    architecture: Cell<LayerArchitecture>,
}

impl KernelLayer {
    pub fn new(architecture: LayerArchitecture) -> Self {
        Self {
            architecture: Cell::new(architecture),
        }
    }

    /// Get the architecture
    pub fn architecture(&self) -> Capability<LayerArchitecture> {
        self.architecture.capability()
    }

    /// Initialize kernel layer
    pub fn initialize(&self) {
        invoke_capability_mut(&self.architecture(), |arch| {
            if let Some(kernel_layer) = arch.get_layer_mut(Layer::Kernel) {
                kernel_layer.set_metadata(String::from("status"), String::from("initialized"));
                kernel_layer.set_metadata(String::from("version"), String::from("2.0.0"));
            }
        });
    }
}

/// AEGIS layer implementation (Security)
pub struct AegisLayer {
    architecture: Cell<LayerArchitecture>,
}

impl AegisLayer {
    pub fn new(architecture: LayerArchitecture) -> Self {
        Self {
            architecture: Cell::new(architecture),
        }
    }

    pub fn architecture(&self) -> Capability<LayerArchitecture> {
        self.architecture.capability()
    }

    /// Initialize AEGIS security layer
    pub fn initialize(&self) {
        invoke_capability_mut(&self.architecture(), |arch| {
            if let Some(aegis_layer) = arch.get_layer_mut(Layer::Aegis) {
                aegis_layer.set_metadata(String::from("status"), String::from("active"));
                aegis_layer.set_metadata(String::from("isolation_level"), String::from("perfect"));
            }
        });
    }
}

/// LUMEN layer implementation (Graphics)
pub struct LumenLayer {
    architecture: Cell<LayerArchitecture>,
    compositor: Cell<Compositor>,
}

impl LumenLayer {
    pub fn new(architecture: LayerArchitecture, compositor: Compositor) -> Self {
        Self {
            architecture: Cell::new(architecture),
            compositor: Cell::new(compositor),
        }
    }

    pub fn architecture(&self) -> Capability<LayerArchitecture> {
        self.architecture.capability()
    }

    pub fn compositor(&self) -> Capability<Compositor> {
        self.compositor.capability()
    }

    /// Initialize LUMEN graphics layer
    pub fn initialize(&self) {
        invoke_capability_mut(&self.architecture(), |arch| {
            if let Some(lumen_layer) = arch.get_layer_mut(Layer::Lumen) {
                lumen_layer.set_metadata(String::from("status"), String::from("active"));
                lumen_layer.set_metadata(String::from("renderer"), String::from("lumen"));
                
                // Add compositor node to LUMEN layer
                let compositor_node = invoke_capability(&self.compositor(), |c| c.compositor_node())
                    .unwrap_or_else(|| {
                        // Create a default node if compositor node doesn't exist
                        Some(crate::graph_kernel::NodeId::new())
                    })
                    .unwrap_or_else(|| crate::graph_kernel::NodeId::new());
                arch.add_node_to_layer(Layer::Lumen, compositor_node);
            }
        });
    }
}

/// GENESIS layer implementation (Auto-creation)
pub struct GenesisLayer {
    architecture: Cell<LayerArchitecture>,
}

impl GenesisLayer {
    pub fn new(architecture: LayerArchitecture) -> Self {
        Self {
            architecture: Cell::new(architecture),
        }
    }

    pub fn architecture(&self) -> Capability<LayerArchitecture> {
        self.architecture.capability()
    }

    /// Initialize GENESIS auto-creation layer
    pub fn initialize(&self) {
        invoke_capability_mut(&self.architecture(), |arch| {
            if let Some(genesis_layer) = arch.get_layer_mut(Layer::Genesis) {
                genesis_layer.set_metadata(String::from("status"), String::from("active"));
                genesis_layer.set_metadata(String::from("auto_optimization"), String::from("enabled"));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_kernel::GraphKernel;

    #[test]
    fn test_layer_architecture_creation() {
        let graph_kernel = GraphKernel::new();
        let mut arch = LayerArchitecture::new(graph_kernel);
        arch.initialize();
        
        assert!(arch.get_layer(Layer::Kernel).is_some());
        assert!(arch.get_layer(Layer::Aegis).is_some());
        assert!(arch.get_layer(Layer::Lumen).is_some());
        assert!(arch.get_layer(Layer::Genesis).is_some());
    }

    #[test]
    fn test_layer_connection() {
        let graph_kernel = GraphKernel::new();
        let mut arch = LayerArchitecture::new(graph_kernel);
        arch.initialize();
        
        let roots = arch.layer_roots();
        assert_eq!(roots.len(), 4);
    }
}
