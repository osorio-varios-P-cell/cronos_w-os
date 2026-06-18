//! Knowledge Persistence Module
//! 
//! This module implements knowledge and context persistence for AI agents based on Hermes Agent architecture.
//! Allows agents to maintain knowledge across sessions and build deep context.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de conocimiento
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeType {
    /// Conocimiento factual
    Factual,
    /// Conocimiento procedimental
    Procedural,
    /// Conocimiento conceptual
    Conceptual,
    /// Conocimiento episódico
    Episodic,
    /// Conocimiento semántico
    Semantic,
}

/// Entrada de conocimiento
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    /// ID de la entrada
    pub id: String,
    /// Tipo de conocimiento
    pub knowledge_type: KnowledgeType,
    /// Título
    pub title: String,
    /// Contenido
    pub content: String,
    /// Etiquetas
    pub tags: Vec<String>,
    /// Fuentes
    pub sources: Vec<String>,
    /// Confianza (0-100)
    pub confidence: u8,
    /// Timestamp de creación
    pub created_at: u64,
    /// Timestamp de última actualización
    pub updated_at: u64,
    /// Número de accesos
    pub access_count: u32,
}

impl KnowledgeEntry {
    /// Crear nueva entrada de conocimiento
    pub fn new(id: String, knowledge_type: KnowledgeType, title: String, content: String) -> Self {
        Self {
            id,
            knowledge_type,
            title,
            content,
            tags: Vec::new(),
            sources: Vec::new(),
            confidence: 50,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
        }
    }

    /// Agregar etiqueta
    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }

    /// Agregar fuente
    pub fn add_source(&mut self, source: String) {
        self.sources.push(source);
    }

    /// Actualizar confianza
    pub fn update_confidence(&mut self, new_confidence: u8) {
        self.confidence = new_confidence.min(100);
        self.updated_at = 0; // En un sistema real, esto sería el tiempo actual
    }

    /// Incrementar contador de accesos
    pub fn increment_access(&mut self) {
        self.access_count += 1;
    }

    /// Calcular puntuación de relevancia
    pub fn relevance_score(&self) -> f64 {
        let confidence_factor = self.confidence as f64 / 100.0;
        let access_factor = (self.access_count as f64 + 1.0) / 10.0;
        let recency_factor = 1.0; // En un sistema real, esto consideraría el tiempo
        
        confidence_factor * access_factor * recency_factor
    }
}

/// Contexto del usuario
#[derive(Debug, Clone)]
pub struct UserContext {
    /// ID del contexto
    pub id: String,
    /// ID del usuario
    pub user_id: String,
    /// Preferencias del usuario
    pub preferences: Vec<(String, String)>,
    /// Historial de interacciones
    pub interaction_history: Vec<String>,
    /// Patrones de comportamiento
    pub behavior_patterns: Vec<String>,
    /// Timestamp de última actualización
    pub updated_at: u64,
}

impl UserContext {
    /// Crear nuevo contexto de usuario
    pub fn new(id: String, user_id: String) -> Self {
        Self {
            id,
            user_id,
            preferences: Vec::new(),
            interaction_history: Vec::new(),
            behavior_patterns: Vec::new(),
            updated_at: 0,
        }
    }

    /// Agregar preferencia
    pub fn add_preference(&mut self, key: String, value: String) {
        // Remover si ya existe
        self.preferences.retain(|(k, _)| k != &key);
        self.preferences.push((key, value));
        self.updated_at = 0;
    }

    /// Obtener preferencia
    pub fn get_preference(&self, key: &str) -> Option<&String> {
        self.preferences.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Agregar interacción al historial
    pub fn add_interaction(&mut self, interaction: String) {
        self.interaction_history.push(interaction);
        self.updated_at = 0;
    }

    /// Agregar patrón de comportamiento
    pub fn add_behavior_pattern(&mut self, pattern: String) {
        self.behavior_patterns.push(pattern);
        self.updated_at = 0;
    }
}

/// Sistema de persistencia de conocimiento
#[derive(Debug, Clone)]
pub struct KnowledgePersistenceSystem {
    /// Entradas de conocimiento
    pub knowledge_entries: Vec<KnowledgeEntry>,
    /// Contextos de usuario
    pub user_contexts: Vec<UserContext>,
    /// Índice de etiquetas
    pub tag_index: Vec<(String, usize)>,
    /// Índice de tipos
    pub type_index: Vec<(KnowledgeType, usize)>,
}

impl KnowledgePersistenceSystem {
    /// Crear nuevo sistema de persistencia
    pub fn new() -> Self {
        Self {
            knowledge_entries: Vec::new(),
            user_contexts: Vec::new(),
            tag_index: Vec::new(),
            type_index: Vec::new(),
        }
    }

    /// Agregar entrada de conocimiento
    pub fn add_knowledge_entry(&mut self, entry: KnowledgeEntry) {
        let entry_idx = self.knowledge_entries.len();
        
        // Agregar al índice de etiquetas
        for tag in &entry.tags {
            self.tag_index.push((tag.clone(), entry_idx));
        }
        
        // Agregar al índice de tipos
        self.type_index.push((entry.knowledge_type, entry_idx));
        
        self.knowledge_entries.push(entry);
    }

    /// Agregar contexto de usuario
    pub fn add_user_context(&mut self, context: UserContext) {
        self.user_contexts.push(context);
    }

    /// Obtener entrada de conocimiento por ID
    pub fn get_knowledge_entry(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.knowledge_entries.iter().find(|e| e.id == id)
    }

    /// Obtener entrada mutable de conocimiento por ID
    pub fn get_knowledge_entry_mut(&mut self, id: &str) -> Option<&mut KnowledgeEntry> {
        self.knowledge_entries.iter_mut().find(|e| e.id == id)
    }

    /// Obtener contexto de usuario por ID
    pub fn get_user_context(&self, user_id: &str) -> Option<&UserContext> {
        self.user_contexts.iter().find(|c| c.user_id == user_id)
    }

    /// Obtener contexto mutable de usuario por ID
    pub fn get_user_context_mut(&mut self, user_id: &str) -> Option<&mut UserContext> {
        self.user_contexts.iter_mut().find(|c| c.user_id == user_id)
    }

    /// Buscar conocimiento por etiqueta
    pub fn search_by_tag(&self, tag: &str) -> Vec<&KnowledgeEntry> {
        let mut indices = Vec::new();
        
        for (t, idx) in &self.tag_index {
            if t == tag {
                indices.push(*idx);
            }
        }
        
        indices.iter()
            .filter_map(|&idx| self.knowledge_entries.get(idx))
            .collect()
    }

    /// Buscar conocimiento por tipo
    pub fn search_by_type(&self, knowledge_type: KnowledgeType) -> Vec<&KnowledgeEntry> {
        let mut indices = Vec::new();
        
        for (t, idx) in &self.type_index {
            if *t == knowledge_type {
                indices.push(*idx);
            }
        }
        
        indices.iter()
            .filter_map(|&idx| self.knowledge_entries.get(idx))
            .collect()
    }

    /// Buscar conocimiento por contenido
    pub fn search_by_content(&self, query: &str) -> Vec<&KnowledgeEntry> {
        let query_lower = query.to_lowercase();
        
        self.knowledge_entries.iter()
            .filter(|e| e.content.to_lowercase().contains(&query_lower) || 
                       e.title.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Consolidar conocimiento similar
    pub fn consolidate_similar_knowledge(&mut self) {
        // En un sistema real, esto consolidaría entradas similares
        let _ = self;
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Knowledge Persistence Status\n");
        report.push_str("===========================\n\n");
        
        report.push_str(&format!("Knowledge Entries: {}\n", self.knowledge_entries.len()));
        report.push_str(&format!("User Contexts: {}\n", self.user_contexts.len()));
        report.push_str(&format!("Tag Index Entries: {}\n", self.tag_index.len()));
        report.push_str(&format!("Type Index Entries: {}\n\n", self.type_index.len()));
        
        report.push_str("Knowledge by Type:\n");
        for knowledge_type in &[KnowledgeType::Factual, KnowledgeType::Procedural, KnowledgeType::Conceptual, KnowledgeType::Episodic, KnowledgeType::Semantic] {
            let count = self.search_by_type(*knowledge_type).len();
            report.push_str(&format!("  {:?}: {}\n", knowledge_type, count));
        }
        
        report
    }
}

impl Default for KnowledgePersistenceSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de persistencia de conocimiento
pub struct KnowledgePersistenceUtils;

impl KnowledgePersistenceUtils {
    /// Crear sistema de persistencia por defecto
    pub fn create_default_persistence_system() -> KnowledgePersistenceSystem {
        let mut system = KnowledgePersistenceSystem::new();
        
        // Agregar entradas de conocimiento por defecto
        let mut cronos_entry = KnowledgeEntry::new(
            String::from("cronos_arch"),
            KnowledgeType::Factual,
            String::from("Cronos W-OS Architecture"),
            String::from("Cronos W-OS is a kernel written in Rust with no_std environment. It implements advanced features like AI agents, hardware monitoring, and self-preservation systems."),
        );
        cronos_entry.add_tag(String::from("kernel"));
        cronos_entry.add_tag(String::from("rust"));
        cronos_entry.add_tag(String::from("architecture"));
        system.add_knowledge_entry(cronos_entry);
        
        let mut hive_entry = KnowledgeEntry::new(
            String::from("hive_ai"),
            KnowledgeType::Conceptual,
            String::from("Hive AI System"),
            String::from("Hive AI is the artificial intelligence system integrated into Cronos W-OS. It provides autonomous decision-making capabilities and learns from system interactions."),
        );
        hive_entry.add_tag(String::from("ai"));
        hive_entry.add_tag(String::from("autonomous"));
        system.add_knowledge_entry(hive_entry);
        
        system
    }

    /// Crear entrada de conocimiento por defecto
    pub fn create_default_knowledge_entry(id: String, title: String, content: String) -> KnowledgeEntry {
        let mut entry = KnowledgeEntry::new(id, KnowledgeType::Factual, title, content);
        entry.update_confidence(70);
        entry
    }

    /// Crear contexto de usuario por defecto
    pub fn create_default_user_context(user_id: String) -> UserContext {
        UserContext::new(format!("ctx_{}", user_id), user_id)
    }
}
