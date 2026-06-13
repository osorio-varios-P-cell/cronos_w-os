//! RAG System Module
//! 
//! This module implements a Retrieval Augmented Generation system for AI agents.
//! Based on Microsoft AI Agents for Beginners course - Lesson 5: Agentic RAG.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Documento en la base de conocimiento
#[derive(Debug, Clone)]
pub struct Document {
    /// ID del documento
    pub id: String,
    /// Título
    pub title: String,
    /// Contenido
    pub content: String,
    /// Metadatos
    pub metadata: Vec<(String, String)>,
    /// Embedding (simulado)
    pub embedding: Vec<f32>,
}

impl Document {
    /// Crear nuevo documento
    pub fn new(id: String, title: String, content: String) -> Self {
        Self {
            id,
            title,
            content,
            metadata: Vec::new(),
            embedding: Vec::new(),
        }
    }

    /// Agregar metadato
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.push((key, value));
    }

    /// Calcular similitud con query
    pub fn similarity(&self, query: &str) -> f64 {
        // En un sistema real, esto usaría embeddings y cosine similarity
        let content_lower = self.content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut matches = 0;
        
        for word in &query_words {
            if content_lower.contains(word) {
                matches += 1;
            }
        }
        
        if query_words.is_empty() {
            0.0
        } else {
            matches as f64 / query_words.len() as f64
        }
    }
}

/// Resultado de búsqueda RAG
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// Documento
    pub document: Document,
    /// Puntuación de relevancia
    pub relevance_score: f64,
    /// Fragmento relevante
    pub snippet: String,
}

impl RetrievalResult {
    /// Crear nuevo resultado
    pub fn new(document: Document, relevance_score: f64, snippet: String) -> Self {
        Self {
            document,
            relevance_score,
            snippet,
        }
    }
}

/// Base de conocimiento
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    /// Documentos
    pub documents: Vec<Document>,
    /// Índice de búsqueda
    pub search_index: Vec<(String, usize)>,
}

impl KnowledgeBase {
    /// Crear nueva base de conocimiento
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            search_index: Vec::new(),
        }
    }

    /// Agregar documento
    pub fn add_document(&mut self, document: Document) {
        let doc_idx = self.documents.len();
        
        // Agregar al índice de búsqueda
        let words: Vec<&str> = document.content.split_whitespace().collect();
        for word in words {
            self.search_index.push((word.to_lowercase(), doc_idx));
        }
        
        self.documents.push(document);
    }

    /// Buscar documentos por query
    pub fn search(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        let mut results = Vec::new();
        
        for document in &self.documents {
            let score = document.similarity(query);
            if score > 0.0 {
                let snippet = self.extract_snippet(&document.content, query);
                results.push(RetrievalResult::new(document.clone(), score, snippet));
            }
        }
        
        // Ordenar por relevancia
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        // Retornar top_k resultados
        results.into_iter().take(top_k).collect()
    }

    /// Extraer fragmento relevante
    fn extract_snippet(&self, content: &str, query: &str) -> String {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = if pos > 50 { pos - 50 } else { 0 };
            let end = if pos + 100 < content.len() { pos + 100 } else { content.len() };
            String::from(&content[start..end])
        } else {
            String::from(&content[..content.len().min(200)])
        }
    }

    /// Obtener documento por ID
    pub fn get_document(&self, id: &str) -> Option<&Document> {
        self.documents.iter().find(|d| d.id == id)
    }

    /// Eliminar documento
    pub fn remove_document(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.documents.iter().position(|d| d.id == id) {
            self.documents.remove(pos);
            // Reconstruir índice
            self.rebuild_index();
            Ok(())
        } else {
            Err(String::from("Document not found"))
        }
    }

    /// Reconstruir índice de búsqueda
    fn rebuild_index(&mut self) {
        self.search_index.clear();
        
        for (doc_idx, document) in self.documents.iter().enumerate() {
            let words: Vec<&str> = document.content.split_whitespace().collect();
            for word in words {
                self.search_index.push((word.to_lowercase(), doc_idx));
            }
        }
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Knowledge Base Status\n");
        report.push_str("=====================\n\n");
        
        report.push_str(&format!("Total Documents: {}\n", self.documents.len()));
        report.push_str(&format!("Search Index Entries: {}\n\n", self.search_index.len()));
        
        for document in &self.documents {
            report.push_str(&format!("Document: {}\n", document.title));
            report.push_str(&format!("  ID: {}\n", document.id));
            report.push_str(&format!("  Content Length: {}\n", document.content.len()));
            report.push_str(&format!("  Metadata: {}\n", document.metadata.len()));
            report.push('\n');
        }
        
        report
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

/// Sistema RAG
pub struct RagSystem {
    /// Base de conocimiento
    pub knowledge_base: KnowledgeBase,
    /// Número de resultados a recuperar
    pub top_k: usize,
    /// Umbral de relevancia
    pub relevance_threshold: f64,
}

impl RagSystem {
    /// Crear nuevo sistema RAG
    pub fn new(top_k: usize, relevance_threshold: f64) -> Self {
        Self {
            knowledge_base: KnowledgeBase::new(),
            top_k,
            relevance_threshold,
        }
    }

    /// Agregar documento a la base de conocimiento
    pub fn add_document(&mut self, document: Document) {
        self.knowledge_base.add_document(document);
    }

    /// Recuperar documentos relevantes
    pub fn retrieve(&self, query: &str) -> Vec<RetrievalResult> {
        let results = self.knowledge_base.search(query, self.top_k);
        
        // Filtrar por umbral de relevancia
        results.into_iter()
            .filter(|r| r.relevance_score >= self.relevance_threshold)
            .collect()
    }

    /// Generar contexto aumentado
    pub fn generate_augmented_context(&self, query: &str) -> String {
        let results = self.retrieve(query);
        
        if results.is_empty() {
            return String::from("No relevant information found.");
        }
        
        let mut context = String::from("Relevant Information:\n\n");
        
        for (i, result) in results.iter().enumerate() {
            context.push_str(&format!("{}. {}\n", i + 1, result.document.title));
            context.push_str(&format!("   {}\n\n", result.snippet));
        }
        
        context
    }

    /// Generar respuesta con contexto aumentado
    pub fn generate_response(&self, query: &str) -> String {
        let context = self.generate_augmented_context(query);
        
        // En un sistema real, esto usaría un LLM para generar la respuesta
        format!("Query: {}\n\nContext:\n{}\n\nResponse: [LLM would generate response here]", query, context)
    }

    /// Actualizar parámetros de búsqueda
    pub fn update_search_params(&mut self, top_k: usize, relevance_threshold: f64) {
        self.top_k = top_k;
        self.relevance_threshold = relevance_threshold;
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("RAG System Status\n");
        report.push_str("================\n\n");
        
        report.push_str(&format!("Top K: {}\n", self.top_k));
        report.push_str(&format!("Relevance Threshold: {}\n\n", self.relevance_threshold));
        
        report.push_str(&self.knowledge_base.generate_status_report());
        
        report
    }
}

impl Default for RagSystem {
    fn default() -> Self {
        Self::new(5, 0.3)
    }
}

/// Utilidades RAG
pub struct RagUtils;

impl RagUtils {
    /// Crear sistema RAG por defecto
    pub fn create_default_rag_system() -> RagSystem {
        RagSystem::new(5, 0.3)
    }

    /// Crear documento por defecto
    pub fn create_default_document(id: String, title: String, content: String) -> Document {
        let mut doc = Document::new(id, title, content);
        doc.add_metadata(String::from("source"), String::from("default"));
        doc
    }

    /// Crear base de conocimiento por defecto
    pub fn create_default_knowledge_base() -> KnowledgeBase {
        let mut kb = KnowledgeBase::new();
        
        // Agregar documentos de ejemplo
        kb.add_document(Self::create_default_document(
            String::from("doc1"),
            String::from("Cronos W-OS Architecture"),
            String::from("Cronos W-OS is a kernel written in Rust with no_std environment. It implements advanced features like AI agents, hardware monitoring, and self-preservation systems."),
        ));
        
        kb.add_document(Self::create_default_document(
            String::from("doc2"),
            String::from("Hive AI"),
            String::from("Hive AI is the artificial intelligence system integrated into Cronos W-OS. It provides autonomous decision-making capabilities and learns from system interactions."),
        ));
        
        kb.add_document(Self::create_default_document(
            String::from("doc3"),
            String::from("Hardware Monitoring"),
            String::from("The kernel includes comprehensive hardware monitoring for temperature, voltage, fan speed, and storage health. It uses SMBus/I2C drivers to communicate with sensors."),
        ));
        
        kb
    }
}
