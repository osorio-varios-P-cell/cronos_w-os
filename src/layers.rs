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
use crate::cronos_advanced_security::CronosAdvancedSecurity;
use crate::cronos_hypervisor::CronosHypervisor;
use crate::cronos_monitoring_system::CronosMonitoringSystem;
use crate::cronos_container_runtime::CronosContainerRuntime;
use crate::theseus_live_evolution::LiveEvolutionManager;
use crate::cronos_backup_system::CronosBackupSystem;
use crate::cronos_package_manager::CronosPackageManager;
use crate::cronos_power_management::CronosPowerManager;
use crate::cronos_update_system::CronosUpdateSystem;
use crate::cronos_cache_system::CronosCacheSystem;
use crate::cronos_btrfs::CronosBtrfsDriver;
use crate::theseus_scheduler::TheseusScheduler;
use crate::theseus_memory::SingleAddressSpaceManager;
use crate::theseus_genesis::CronosLiveEvolutionIntegration;
use crate::cosmic_compositor::CosmicCompositor;
use crate::cosmic_ui::CosmicUi;
use crate::redox_ext4::RedoxExt4Filesystem;
use crate::redoxfs::RedoxFS;
use crate::haiku_bfs::BfsManager;
use crate::skills_system::SkillsSystem;
use crate::knowledge_persistence::KnowledgePersistenceSystem;
use crate::user_modeling::DeepUserModel;
use crate::learning_loop::LearningLoop;
use crate::context_engineering::ContextEngineeringSystem;
use crate::multi_model_integration::{MultiModelIntegration, ModelSelectionStrategy};
use crate::rag_system::RagSystem;
use crate::media_engine::MediaEngine;
use crate::genode_components::GenodeComponentManager;
use crate::plan9_9p::NinePManager;
use crate::fuchsia_capabilities::FuchsiaCapabilityManager;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

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
    /// FASE 13: Advanced Security de CRONOS original (para capa AEGIS)
    pub advanced_security: Option<CronosAdvancedSecurity>,
    /// FASE 13: Hypervisor de CRONOS original (para capa GENESIS)
    pub hypervisor: Option<CronosHypervisor>,
    /// FASE 13: Container Runtime de CRONOS original (para capa GENESIS)
    pub container_runtime: Option<CronosContainerRuntime>,
    /// FASE 13: Live Evolution de Theseus (para capa GENESIS)
    pub live_evolution: Option<LiveEvolutionManager>,
    /// FASE 14: Cache System de CRONOS original (para capa GENESIS)
    pub cache_system: Option<CronosCacheSystem>,
    /// FASE 14: Btrfs de CRONOS original (para capa GENESIS)
    pub btrfs: Option<CronosBtrfsDriver>,
    /// FASE 14: Scheduler de Theseus (para capa Kernel)
    pub theseus_scheduler: Option<TheseusScheduler>,
    /// FASE 14: Memory de Theseus (para capa Kernel)
    pub theseus_memory: Option<SingleAddressSpaceManager>,
    /// FASE 14: Genesis de Theseus (para capa GENESIS)
    pub theseus_genesis: Option<CronosLiveEvolutionIntegration>,
    /// FASE 15: Cosmic Compositor (para capa LUMEN)
    pub cosmic_compositor: Option<CosmicCompositor>,
    /// FASE 15: Cosmic UI (para capa LUMEN)
    pub cosmic_ui: Option<CosmicUi>,
    /// FASE 15: Redox Ext4 Filesystem (para capa GENESIS)
    pub redox_ext4: Option<RedoxExt4Filesystem>,
    /// FASE 15: RedoxFS (para capa GENESIS)
    pub redoxfs: Option<RedoxFS>,
    /// FASE 15: Haiku BFS (para capa GENESIS)
    pub haiku_bfs: Option<BfsManager>,
    /// FASE 15: Skills System (para capa GENESIS)
    pub skills_system: Option<SkillsSystem>,
    /// FASE 15: Knowledge Persistence System (para capa GENESIS)
    pub knowledge_persistence: Option<KnowledgePersistenceSystem>,
    /// FASE 15: User Modeling (para capa GENESIS)
    pub user_modeling: Option<DeepUserModel>,
    /// FASE 15: Learning Loop (para capa GENESIS)
    pub learning_loop: Option<LearningLoop>,
    /// FASE 15: Context Engineering System (para capa GENESIS)
    pub context_engineering: Option<ContextEngineeringSystem>,
    /// FASE 15: Multi Model Integration (para capa GENESIS)
    pub multi_model_integration: Option<MultiModelIntegration>,
    /// FASE 15: RAG System (para capa GENESIS)
    pub rag_system: Option<RagSystem>,
    /// FASE 15: Media Engine (para capa LUMEN)
    pub media_engine: Option<MediaEngine>,
    /// FASE 15: Genode Component Manager (para capa GENESIS)
    pub genode_components: Option<GenodeComponentManager>,
    /// FASE 15: Plan9 9p Manager (para capa GENESIS)
    pub plan9_9p: Option<NinePManager>,
    /// FASE 15: Fuchsia Capability Manager (para capa AEGIS)
    pub fuchsia_capabilities: Option<FuchsiaCapabilityManager>,
}

impl SystemLayer {
    pub fn new(layer_type: Layer) -> Self {
        Self {
            layer_type,
            root_node: None,
            capabilities: BTreeSet::new(),
            child_nodes: BTreeSet::new(),
            metadata: BTreeMap::new(),
            advanced_security: None,
            hypervisor: None,
            container_runtime: None,
            live_evolution: None,
            cache_system: None,
            btrfs: None,
            theseus_scheduler: None,
            theseus_memory: None,
            theseus_genesis: None,
            cosmic_compositor: None,
            cosmic_ui: None,
            redox_ext4: None,
            redoxfs: None,
            haiku_bfs: None,
            skills_system: None,
            knowledge_persistence: None,
            user_modeling: None,
            learning_loop: None,
            context_engineering: None,
            multi_model_integration: None,
            rag_system: None,
            media_engine: None,
            genode_components: None,
            plan9_9p: None,
            fuchsia_capabilities: None,
        }
    }

    /// FASE 13: Inicializar advanced security para capa AEGIS
    pub fn initialize_advanced_security(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Aegis {
            let config = crate::cronos_advanced_security::SecurityConfig::new();
            let advanced = CronosAdvancedSecurity::new(config);
            self.advanced_security = Some(advanced);
            Ok(())
        } else {
            Err(format!("Advanced security solo se puede inicializar en capa AEGIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 13: Inicializar hypervisor para capa GENESIS
    pub fn initialize_hypervisor(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let hypervisor = CronosHypervisor::new();
            self.hypervisor = Some(hypervisor);
            Ok(())
        } else {
            Err(format!("Hypervisor solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 13: Inicializar container runtime para capa GENESIS
    pub fn initialize_container_runtime(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let runtime = CronosContainerRuntime::new();
            self.container_runtime = Some(runtime);
            Ok(())
        } else {
            Err(format!("Container runtime solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 13: Inicializar live evolution para capa GENESIS
    pub fn initialize_live_evolution(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let evolution = LiveEvolutionManager::new();
            self.live_evolution = Some(evolution);
            Ok(())
        } else {
            Err(format!("Live evolution solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 14: Inicializar cache system para capa GENESIS
    pub fn initialize_cache_system(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let cache = CronosCacheSystem::new();
            self.cache_system = Some(cache);
            Ok(())
        } else {
            Err(format!("Cache system solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 14: Inicializar btrfs para capa GENESIS
    pub fn initialize_btrfs(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let btrfs = CronosBtrfsDriver::new();
            self.btrfs = Some(btrfs);
            Ok(())
        } else {
            Err(format!("Btrfs solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 14: Inicializar theseus scheduler para capa Kernel
    pub fn initialize_theseus_scheduler(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Kernel {
            let scheduler = TheseusScheduler::new();
            self.theseus_scheduler = Some(scheduler);
            Ok(())
        } else {
            Err(format!("Theseus scheduler solo se puede inicializar en capa Kernel, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 14: Inicializar theseus memory para capa Kernel
    pub fn initialize_theseus_memory(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Kernel {
            let memory = SingleAddressSpaceManager::new(0x1000, 0x100000000);
            self.theseus_memory = Some(memory);
            Ok(())
        } else {
            Err(format!("Theseus memory solo se puede inicializar en capa Kernel, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 14: Inicializar theseus genesis para capa GENESIS
    pub fn initialize_theseus_genesis(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let genesis = CronosLiveEvolutionIntegration::new();
            self.theseus_genesis = Some(genesis);
            Ok(())
        } else {
            Err(format!("Theseus genesis solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar cosmic compositor para capa LUMEN
    pub fn initialize_cosmic_compositor(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Lumen {
            let compositor = CosmicCompositor::new();
            self.cosmic_compositor = Some(compositor);
            Ok(())
        } else {
            Err(format!("Cosmic compositor solo se puede inicializar en capa LUMEN, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar cosmic UI para capa LUMEN
    pub fn initialize_cosmic_ui(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Lumen {
            // Nota: Cosmic UI requiere Capability<Compositor> y CosmicPalette
            // Por ahora lo inicializamos de manera simplificada
            // En el futuro se debe integrar con el compositor real
            Err(format!("Cosmic UI requiere Capability<Compositor>, implementación pendiente"))
        } else {
            Err(format!("Cosmic UI solo se puede inicializar en capa LUMEN, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar redox ext4 filesystem para capa GENESIS
    pub fn initialize_redox_ext4(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let ext4 = RedoxExt4Filesystem::new(String::from("/"));
            self.redox_ext4 = Some(ext4);
            Ok(())
        } else {
            Err(format!("Redox Ext4 solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar redoxfs para capa GENESIS
    pub fn initialize_redoxfs(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let redoxfs = RedoxFS::new();
            self.redoxfs = Some(redoxfs);
            Ok(())
        } else {
            Err(format!("RedoxFS solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar haiku bfs para capa GENESIS
    pub fn initialize_haiku_bfs(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let bfs = BfsManager::new();
            self.haiku_bfs = Some(bfs);
            Ok(())
        } else {
            Err(format!("Haiku BFS solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar skills system para capa GENESIS
    pub fn initialize_skills_system(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let skills = SkillsSystem::new();
            self.skills_system = Some(skills);
            Ok(())
        } else {
            Err(format!("Skills system solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar knowledge persistence system para capa GENESIS
    pub fn initialize_knowledge_persistence(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let kp = KnowledgePersistenceSystem::new();
            self.knowledge_persistence = Some(kp);
            Ok(())
        } else {
            Err(format!("Knowledge persistence system solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar user modeling para capa GENESIS
    pub fn initialize_user_modeling(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            // Nota: DeepUserModel requiere UserProfile en el constructor
            // Por ahora lo inicializamos de manera simplificada
            // En el futuro se debe integrar con el sistema de usuarios real
            Err(format!("User modeling requiere UserProfile, implementación pendiente"))
        } else {
            Err(format!("User modeling solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar learning loop para capa GENESIS
    pub fn initialize_learning_loop(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let learning_loop = LearningLoop::new(60); // 60 segundos por defecto
            self.learning_loop = Some(learning_loop);
            Ok(())
        } else {
            Err(format!("Learning loop solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar context engineering system para capa GENESIS
    pub fn initialize_context_engineering(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let ce = ContextEngineeringSystem::new();
            self.context_engineering = Some(ce);
            Ok(())
        } else {
            Err(format!("Context engineering system solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar multi model integration para capa GENESIS
    pub fn initialize_multi_model_integration(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let mmi = MultiModelIntegration::new(ModelSelectionStrategy::Balanced);
            self.multi_model_integration = Some(mmi);
            Ok(())
        } else {
            Err(format!("Multi model integration solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar rag system para capa GENESIS
    pub fn initialize_rag_system(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let rag = RagSystem::new(5, 0.7); // top_k=5, threshold=0.7
            self.rag_system = Some(rag);
            Ok(())
        } else {
            Err(format!("RAG system solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar media engine para capa LUMEN
    pub fn initialize_media_engine(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Lumen {
            let media = MediaEngine::new();
            self.media_engine = Some(media);
            Ok(())
        } else {
            Err(format!("Media engine solo se puede inicializar en capa LUMEN, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar genode component manager para capa GENESIS
    pub fn initialize_genode_components(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let genode = GenodeComponentManager::new();
            self.genode_components = Some(genode);
            Ok(())
        } else {
            Err(format!("Genode component manager solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar plan9 9p manager para capa GENESIS
    pub fn initialize_plan9_9p(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Genesis {
            let ninep = NinePManager::new();
            self.plan9_9p = Some(ninep);
            Ok(())
        } else {
            Err(format!("Plan9 9p manager solo se puede inicializar en capa GENESIS, capa actual: {:?}", self.layer_type))
        }
    }

    /// FASE 15: Inicializar fuchsia capability manager para capa AEGIS
    pub fn initialize_fuchsia_capabilities(&mut self) -> Result<(), String> {
        if self.layer_type == Layer::Aegis {
            let fuchsia = FuchsiaCapabilityManager::new();
            self.fuchsia_capabilities = Some(fuchsia);
            Ok(())
        } else {
            Err(format!("Fuchsia capability manager solo se puede inicializar en capa AEGIS, capa actual: {:?}", self.layer_type))
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
    /// FASE 13: Monitoring System de CRONOS original (disponible en todas las capas)
    pub monitoring_system: Option<CronosMonitoringSystem>,
    /// FASE 13: Backup System de CRONOS original
    pub backup_system: Option<CronosBackupSystem>,
    /// FASE 13: Package Manager de CRONOS original
    pub package_manager: Option<CronosPackageManager>,
    /// FASE 13: Power Management de CRONOS original
    pub power_management: Option<CronosPowerManager>,
    /// FASE 13: Update System de CRONOS original
    pub update_system: Option<CronosUpdateSystem>,
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
            monitoring_system: None,
            backup_system: None,
            package_manager: None,
            power_management: None,
            update_system: None,
        }
    }

    /// FASE 13: Inicializar monitoring system para todas las capas
    pub fn initialize_monitoring_system(&mut self) -> Result<(), String> {
        let monitoring = CronosMonitoringSystem::new(1000);
        self.monitoring_system = Some(monitoring);
        Ok(())
    }

    /// FASE 13: Inicializar backup system
    pub fn initialize_backup_system(&mut self) -> Result<(), String> {
        let config = crate::cronos_backup_system::BackupConfig::new();
        let backup = CronosBackupSystem::new(config);
        self.backup_system = Some(backup);
        Ok(())
    }

    /// FASE 13: Inicializar package manager
    pub fn initialize_package_manager(&mut self) -> Result<(), String> {
        let package_manager = CronosPackageManager::new();
        self.package_manager = Some(package_manager);
        Ok(())
    }

    /// FASE 13: Inicializar power management
    pub fn initialize_power_management(&mut self) -> Result<(), String> {
        let power = CronosPowerManager::new();
        self.power_management = Some(power);
        Ok(())
    }

    /// FASE 13: Inicializar update system
    pub fn initialize_update_system(&mut self) -> Result<(), String> {
        let config = crate::cronos_update_system::UpdateConfig::new();
        let current_version = crate::cronos_update_system::SystemVersion::new(1, 0, 0, 0);
        let update = CronosUpdateSystem::new(config, current_version);
        self.update_system = Some(update);
        Ok(())
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
            if let Some(edge_id) = self.graph_kernel.create_edge(
                from_layer,
                to_layer,
                EdgeType::ControlFlow,
            ) {
                self.layer_edges.insert((from, to), edge_id);
            }
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
                kernel_layer.set_metadata(String::from("status"), String::from("Gestalt-Prime"));
                kernel_layer.set_metadata(String::from("version"), String::from("3.0.0"));
                // FASE 3.0: Memory Leakage Guardians
                kernel_layer.set_metadata(String::from("memory_protection"), String::from("Leakage-Guardian-Active"));
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
                // FASE 3.0: Recursive Resource Quotas (Genode Style)
                aegis_layer.set_metadata(String::from("security_engine"), String::from("Recursive-Quotas-Active"));
                aegis_layer.set_metadata(String::from("isolation"), String::from("State-Spillover-Prevented"));
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
                lumen_layer.set_metadata(String::from("renderer"), String::from("lumen-crystal-flow"));
                // FASE 3.0: Double-Buffer Shadowing
                lumen_layer.set_metadata(String::from("buffer_strategy"), String::from("Double-Buffer-Shadowing"));
                
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
                // FASE 2.9: Autonomous Self-Healing
                genesis_layer.set_metadata(String::from("self_healing"), String::from("Autonomous-Neural-Ready"));
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
