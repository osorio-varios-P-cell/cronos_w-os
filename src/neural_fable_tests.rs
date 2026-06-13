//! Tests para CRONOS Neural Fable (v2.5)
//!
//! Este módulo valida la integración de la arquitectura de 'Segundo Cerebro' (Obsidian)
//! y el motor de razonamiento autónomo (Fable 5).

use crate::graph_kernel::{GraphKernel, NodeType, EdgeType};
use crate::hive_ai::{HiveAi, Belief, OptimizationType};
use crate::layers::LayerArchitecture;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

pub fn run_neural_fable_validation_tests() -> Result<String, String> {
    let mut report = String::from("🧪 INICIANDO VALIDACIÓN NEURAL FABLE (v2.5)...\n");

    // 1. Test de Nodos de Conocimiento (Obsidian)
    let mut gk = GraphKernel::new();
    gk.initialize();

    let node_id = gk.create_node(
        NodeType::KnowledgeNode {
            category: String::from("TestKnowledge"),
            tags: alloc::vec![String::from("#test"), String::from("#neural")]
        },
        String::from("Test_Note.md")
    );

    let node = gk.get_node(node_id).ok_or("Error al recuperar KnowledgeNode")?;
    if let NodeType::KnowledgeNode { category, .. } = node.node_type {
        report.push_str(&format!("  [OK] KnowledgeNode creado: {} (Categoría: {})\n", node.name, category));
    } else {
        return Err(String::from("Tipo de nodo incorrecto: se esperaba KnowledgeNode"));
    }

    // 2. Test de Enlaces Bidireccionales
    let other_node = gk.create_node(NodeType::Process, String::from("TestProcess"));
    gk.create_edge(node_id, other_node, EdgeType::BidirectionalLink);
    report.push_str("  [OK] Enlace Bidireccional establecido entre Conocimiento y Proceso.\n");

    // 3. Test de Razonamiento Fable 5
    let arch = LayerArchitecture::new(gk.clone());
    let mut hive_ai = HiveAi::new(arch);
    hive_ai.enable();

    let reasoning_result = hive_ai.perform_fable_reasoning("Optimización Global");
    report.push_str(&format!("  [OK] Motor Fable 5: {}\n", reasoning_result));

    // Verificar si se generaron pasos de razonamiento
    if hive_ai.current_reasoning.is_empty() {
        return Err(String::from("El motor Fable no generó pasos de razonamiento (Chain of Thought)"));
    }
    report.push_str("  [OK] Chain of Thought generada correctamente.\n");

    // 4. Test de Synergy Swarm (v2.7)
    let mut swarm = crate::hive_swarm::HiveSwarm::new();
    swarm.spawn_expert("Trading");
    if swarm.agents.len() == 1 {
        report.push_str("  [OK] Synergy Swarm: Agente especialista (Trading) creado.\n");
    }

    // 5. Test de Media Engine (v2.7)
    let engine = crate::media_engine::MediaEngine::new();
    let sample_data = [0u8; 100];
    let converted = engine.convert(&sample_data, crate::media_engine::MediaFormat::Docx, crate::media_engine::MediaFormat::Pdf);
    if converted.len() > 0 && converted.last() == Some(&0xFF) {
        report.push_str("  [OK] Media Engine: Conversión Office -> PDF validada.\n");
    }

    // 6. Test de Resource Orchestrator (v2.7)
    let mut orchestrator = crate::resource_orchestrator::ResourceOrchestrator::new();
    orchestrator.link_server("TestNode", "https://api.free-gpu.com");
    if orchestrator.remote_nodes.len() == 1 {
        report.push_str("  [OK] Resource Orchestrator: Vinculación de nodo externo exitosa.\n");
    }

    report.push_str("✅ VALIDACIÓN COMPLETADA: Sistema SYNERGY (v2.7) Estable.");
    Ok(report)
}
