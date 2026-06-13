//! Agent Memory Module
//! 
//! This module implements agent memory systems (short-term and long-term).
//! Based on Microsoft AI Agents for Beginners course - Lesson 13: Agent Memory.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de memoria
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// Memoria a corto plazo
    ShortTerm,
    /// Memoria a largo plazo
    LongTerm,
    /// Memoria de trabajo
    Working,
    /// Memoria episódica
    Episodic,
}

/// Entrada de memoria
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// ID de la entrada
    pub id: String,
    /// Tipo de memoria
    pub memory_type: MemoryType,
    /// Contenido
    pub content: String,
    /// Timestamp
    pub timestamp: u64,
    /// Importancia (0-100)
    pub importance: u8,
    /// Etiquetas
    pub tags: Vec<String>,
    /// Número de accesos
    pub access_count: u32,
}

impl MemoryEntry {
    /// Crear nueva entrada de memoria
    pub fn new(id: String, memory_type: MemoryType, content: String, importance: u8) -> Self {
        Self {
            id,
            memory_type,
            content,
            timestamp: 0, // En un sistema real, esto sería el tiempo actual
            importance,
            tags: Vec::new(),
            access_count: 0,
        }
    }

    /// Agregar etiqueta
    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }

    /// Incrementar contador de accesos
    pub fn increment_access(&mut self) {
        self.access_count += 1;
    }

    /// Calcular puntuación de relevancia
    pub fn relevance_score(&self) -> f64 {
        let time_factor = 1.0; // En un sistema real, esto consideraría el tiempo
        let importance_factor = self.importance as f64 / 100.0;
        let access_factor = (self.access_count as f64 + 1.0) / 10.0;
        
        time_factor * importance_factor * access_factor
    }
}

/// Memoria a corto plazo
#[derive(Debug, Clone)]
pub struct ShortTermMemory {
    /// Entradas de memoria
    pub entries: Vec<MemoryEntry>,
    /// Capacidad máxima
    pub capacity: usize,
}

impl ShortTermMemory {
    /// Crear nueva memoria a corto plazo
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }

    /// Agregar entrada
    pub fn add(&mut self, entry: MemoryEntry) -> Result<(), String> {
        if self.entries.len() >= self.capacity {
            // Remover la entrada menos relevante
            self.evict_least_relevant();
        }
        
        self.entries.push(entry);
        Ok(())
    }

    /// Obtener entrada por ID
    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Buscar por contenido
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.entries.iter()
            .filter(|e| e.content.contains(query))
            .collect()
    }

    /// Buscar por etiqueta
    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        self.entries.iter()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Remover entrada menos relevante
    fn evict_least_relevant(&mut self) {
        if let Some(least_relevant_idx) = self.entries.iter()
            .enumerate()
            .min_by(|a, b| a.1.relevance_score().partial_cmp(&b.1.relevance_score()).unwrap())
            .map(|(idx, _)| idx)
        {
            self.entries.remove(least_relevant_idx);
        }
    }

    /// Limpiar memoria
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Memoria a largo plazo
#[derive(Debug, Clone)]
pub struct LongTermMemory {
    /// Entradas de memoria
    pub entries: Vec<MemoryEntry>,
    /// Índice de etiquetas
    pub tag_index: Vec<(String, usize)>,
}

impl LongTermMemory {
    /// Crear nueva memoria a largo plazo
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            tag_index: Vec::new(),
        }
    }

    /// Agregar entrada
    pub fn add(&mut self, mut entry: MemoryEntry) {
        // Agregar al índice de etiquetas
        for tag in &entry.tags {
            self.tag_index.push((tag.clone(), self.entries.len()));
        }
        
        self.entries.push(entry);
    }

    /// Obtener entrada por ID
    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Buscar por contenido
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.entries.iter()
            .filter(|e| e.content.contains(query))
            .collect()
    }

    /// Buscar por etiqueta
    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        let mut indices = Vec::new();
        
        for (t, idx) in &self.tag_index {
            if t == tag {
                indices.push(*idx);
            }
        }
        
        indices.iter()
            .filter_map(|&idx| self.entries.get(idx))
            .collect()
    }

    /// Buscar por importancia mínima
    pub fn search_by_importance(&self, min_importance: u8) -> Vec<&MemoryEntry> {
        self.entries.iter()
            .filter(|e| e.importance >= min_importance)
            .collect()
    }

    /// Consolidar entradas similares
    pub fn consolidate(&mut self) {
        // En un sistema real, esto consolidaría entradas similares
        let _ = self;
    }

    /// Limpiar memoria
    pub fn clear(&mut self) {
        self.entries.clear();
        self.tag_index.clear();
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Sistema de memoria del agente
pub struct AgentMemorySystem {
    /// Memoria a corto plazo
    pub short_term: ShortTermMemory,
    /// Memoria a largo plazo
    pub long_term: LongTermMemory,
    /// Memoria de trabajo
    pub working_memory: Vec<String>,
    /// Capacidad de memoria de trabajo
    pub working_capacity: usize,
}

impl AgentMemorySystem {
    /// Crear nuevo sistema de memoria
    pub fn new(short_term_capacity: usize, working_capacity: usize) -> Self {
        Self {
            short_term: ShortTermMemory::new(short_term_capacity),
            long_term: LongTermMemory::new(),
            working_memory: Vec::new(),
            working_capacity,
        }
    }

    /// Agregar a memoria a corto plazo
    pub fn add_short_term(&mut self, entry: MemoryEntry) -> Result<(), String> {
        self.short_term.add(entry)
    }

    /// Agregar a memoria a largo plazo
    pub fn add_long_term(&mut self, entry: MemoryEntry) {
        self.long_term.add(entry);
    }

    /// Agregar a memoria de trabajo
    pub fn add_working(&mut self, item: String) -> Result<(), String> {
        if self.working_memory.len() >= self.working_capacity {
            // Remover el item más antiguo
            self.working_memory.remove(0);
        }
        
        self.working_memory.push(item);
        Ok(())
    }

    /// Transferir de corto a largo plazo
    pub fn promote_to_long_term(&mut self, id: &str) -> Result<(), String> {
        if let Some(entry) = self.short_term.get(id) {
            let mut entry_clone = entry.clone();
            entry_clone.memory_type = MemoryType::LongTerm;
            self.long_term.add(entry_clone);
            Ok(())
        } else {
            Err(String::from("Entry not found in short-term memory"))
        }
    }

    /// Buscar en todas las memorias
    pub fn search_all(&self, query: &str) -> Vec<&MemoryEntry> {
        let mut results = Vec::new();
        
        results.extend(self.short_term.search(query));
        results.extend(self.long_term.search(query));
        
        results
    }

    /// Buscar por etiqueta en todas las memorias
    pub fn search_by_tag_all(&self, tag: &str) -> Vec<&MemoryEntry> {
        let mut results = Vec::new();
        
        results.extend(self.short_term.search_by_tag(tag));
        results.extend(self.long_term.search_by_tag(tag));
        
        results
    }

    /// Obtener memoria de trabajo
    pub fn get_working_memory(&self) -> &[String] {
        &self.working_memory
    }

    /// Limpiar memoria de trabajo
    pub fn clear_working_memory(&mut self) {
        self.working_memory.clear();
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Agent Memory System Status\n");
        report.push_str("=========================\n\n");
        
        report.push_str(&format!("Short-term Memory: {} / {}\n", 
            self.short_term.entries.len(), self.short_term.capacity));
        report.push_str(&format!("Long-term Memory: {}\n", self.long_term.entries.len()));
        report.push_str(&format!("Working Memory: {} / {}\n", 
            self.working_memory.len(), self.working_capacity));
        report.push_str(&format!("Tag Index Entries: {}\n\n", self.long_term.tag_index.len()));
        
        report.push_str("Short-term Entries:\n");
        for entry in &self.short_term.entries {
            report.push_str(&format!("  - {} (Importance: {})\n", entry.id, entry.importance));
        }
        
        report.push('\n');
        
        report.push_str("Long-term Entries:\n");
        for entry in &self.long_term.entries {
            report.push_str(&format!("  - {} (Importance: {})\n", entry.id, entry.importance));
        }
        
        report
    }
}

impl Default for AgentMemorySystem {
    fn default() -> Self {
        Self::new(100, 10)
    }
}

/// Utilidades de memoria
pub struct MemoryUtils;

impl MemoryUtils {
    /// Crear sistema de memoria por defecto
    pub fn create_default_memory_system() -> AgentMemorySystem {
        AgentMemorySystem::new(100, 10)
    }

    /// Crear entrada de memoria por defecto
    pub fn create_default_entry(id: String, content: String) -> MemoryEntry {
        MemoryEntry::new(id, MemoryType::ShortTerm, content, 50)
    }
}
