//! Learning Loop Module
//! 
//! This module implements a self-improving learning loop for AI agents based on Hermes Agent architecture.
//! The learning loop allows agents to continuously improve from experience.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Tipo de evento de aprendizaje
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningEventType {
    /// Éxito
    Success,
    /// Fallo
    Failure,
    /// Descubrimiento
    Discovery,
    /// Corrección
    Correction,
    /// Optimización
    Optimization,
}

/// Evento de aprendizaje
#[derive(Debug, Clone)]
pub struct LearningEvent {
    /// ID del evento
    pub id: String,
    /// Tipo de evento
    pub event_type: LearningEventType,
    /// Contexto del evento
    pub context: String,
    /// Acción tomada
    pub action: String,
    /// Resultado
    pub result: String,
    /// Lección aprendida
    pub lesson: String,
    /// Timestamp
    pub timestamp: u64,
    /// Importancia (0-100)
    pub importance: u8,
}

impl LearningEvent {
    /// Crear nuevo evento de aprendizaje
    pub fn new(id: String, event_type: LearningEventType, context: String, action: String, result: String, lesson: String) -> Self {
        Self {
            id,
            event_type,
            context,
            action,
            result,
            lesson,
            timestamp: 0, // En un sistema real, esto sería el tiempo actual
            importance: 50,
        }
    }

    /// Establecer importancia
    pub fn set_importance(&mut self, importance: u8) {
        self.importance = importance.min(100);
    }
}

/// Patrón de aprendizaje
#[derive(Debug, Clone)]
pub struct LearningPattern {
    /// ID del patrón
    pub id: String,
    /// Nombre del patrón
    pub name: String,
    /// Condiciones del patrón
    pub conditions: Vec<String>,
    /// Acción recomendada
    pub recommended_action: String,
    /// Tasa de éxito del patrón
    pub success_rate: f64,
    /// Número de veces observado
    pub observation_count: u32,
}

impl LearningPattern {
    /// Crear nuevo patrón de aprendizaje
    pub fn new(id: String, name: String, conditions: Vec<String>, recommended_action: String) -> Self {
        Self {
            id,
            name,
            conditions,
            recommended_action,
            success_rate: 0.5,
            observation_count: 0,
        }
    }

    /// Actualizar tasa de éxito
    pub fn update_success_rate(&mut self, success: bool) {
        self.observation_count += 1;
        
        let current_rate = self.success_rate;
        let new_rate = if success { 1.0 } else { 0.0 };
        let updated_rate = (current_rate * (self.observation_count - 1) as f64 + new_rate) / self.observation_count as f64;
        self.success_rate = updated_rate;
    }
}

/// Ciclo de aprendizaje
#[derive(Debug, Clone)]
pub struct LearningLoop {
    /// Eventos de aprendizaje
    pub events: Vec<LearningEvent>,
    /// Patrones de aprendizaje
    pub patterns: Vec<LearningPattern>,
    /// Estado del ciclo
    pub active: bool,
    /// Frecuencia del ciclo (en segundos)
    pub cycle_frequency: u64,
    /// Última ejecución del ciclo
    pub last_cycle_time: u64,
    /// Número de ciclos ejecutados
    pub cycle_count: u32,
}

impl LearningLoop {
    /// Crear nuevo ciclo de aprendizaje
    pub fn new(cycle_frequency: u64) -> Self {
        Self {
            events: Vec::new(),
            patterns: Vec::new(),
            active: true,
            cycle_frequency,
            last_cycle_time: 0,
            cycle_count: 0,
        }
    }

    /// Agregar evento de aprendizaje
    pub fn add_event(&mut self, event: LearningEvent) {
        self.events.push(event);
    }

    /// Agregar patrón de aprendizaje
    pub fn add_pattern(&mut self, pattern: LearningPattern) {
        self.patterns.push(pattern);
    }

    /// Ejecutar ciclo de aprendizaje
    pub fn execute_cycle(&mut self) -> Result<String, String> {
        if !self.active {
            return Err(String::from("Learning loop is not active"));
        }

        self.cycle_count += 1;
        self.last_cycle_time = 0; // En un sistema real, esto sería el tiempo actual

        // Analizar eventos recientes
        let recent_events: Vec<LearningEvent> = self.events.iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        // Identificar patrones
        self.identify_patterns(&recent_events);

        // Consolidar lecciones
        let lessons_learned = self.consolidate_lessons(&recent_events);

        // Mejorar habilidades basadas en lecciones
        let improvements = self.generate_improvements(&lessons_learned);

        Ok(format!("Learning cycle {} executed. Patterns identified: {}, Improvements: {}", 
            self.cycle_count, self.patterns.len(), improvements.len()))
    }

    /// Identificar patrones en eventos
    fn identify_patterns(&mut self, events: &[LearningEvent]) {
        // En un sistema real, esto identificaría patrones complejos
        // Por ahora, implementamos una versión simplificada
        
        let success_events: Vec<&LearningEvent> = events.iter()
            .filter(|e| e.event_type == LearningEventType::Success)
            .collect();

        if success_events.len() >= 3 {
            // Detectar patrón de éxito
            let pattern = LearningPattern::new(
                format!("pattern_{}", self.patterns.len()),
                String::from("Success Pattern"),
                vec![String::from("similar_context")],
                String::from("repeat_action"),
            );
            self.patterns.push(pattern);
        }
    }

    /// Consolidar lecciones de eventos
    fn consolidate_lessons(&self, events: &[LearningEvent]) -> Vec<String> {
        let mut lessons = Vec::new();
        
        for event in events {
            if !event.lesson.is_empty() {
                lessons.push(event.lesson.clone());
            }
        }
        
        lessons
    }

    /// Generar mejoras basadas en lecciones
    fn generate_improvements(&self, lessons: &[String]) -> Vec<String> {
        let mut improvements = Vec::new();
        
        for lesson in lessons {
            improvements.push(format!("Improve based on: {}", lesson));
        }
        
        improvements
    }

    /// Activar ciclo de aprendizaje
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Desactivar ciclo de aprendizaje
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Establecer frecuencia del ciclo
    pub fn set_cycle_frequency(&mut self, frequency: u64) {
        self.cycle_frequency = frequency;
    }

    /// Obtener eventos por tipo
    pub fn get_events_by_type(&self, event_type: LearningEventType) -> Vec<&LearningEvent> {
        self.events.iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Obtener eventos por importancia
    pub fn get_events_by_importance(&self, min_importance: u8) -> Vec<&LearningEvent> {
        self.events.iter()
            .filter(|e| e.importance >= min_importance)
            .collect()
    }

    /// Obtener patrones por tasa de éxito
    pub fn get_patterns_by_success_rate(&self, min_rate: f64) -> Vec<&LearningPattern> {
        self.patterns.iter()
            .filter(|p| p.success_rate >= min_rate)
            .collect()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Learning Loop Status\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("Active: {}\n", self.active));
        report.push_str(&format!("Cycle Frequency: {}s\n", self.cycle_frequency));
        report.push_str(&format!("Cycle Count: {}\n", self.cycle_count));
        report.push_str(&format!("Learning Events: {}\n", self.events.len()));
        report.push_str(&format!("Learning Patterns: {}\n\n", self.patterns.len()));
        
        report.push_str("Recent Events:\n");
        for event in self.events.iter().rev().take(5) {
            report.push_str(&format!("  - {:?}: {} (Importance: {})\n", 
                event.event_type, event.lesson, event.importance));
        }
        
        report.push('\n');
        
        report.push_str("Top Patterns:\n");
        let mut patterns: Vec<&LearningPattern> = self.patterns.iter().collect();
        patterns.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
        
        for pattern in patterns.iter().take(5) {
            report.push_str(&format!("  - {} (Success Rate: {:.2}, Observations: {})\n", 
                pattern.name, pattern.success_rate, pattern.observation_count));
        }
        
        report
    }
}

impl Default for LearningLoop {
    fn default() -> Self {
        Self::new(60) // 60 segundos por defecto
    }
}

/// Utilidades del ciclo de aprendizaje
pub struct LearningLoopUtils;

impl LearningLoopUtils {
    /// Crear ciclo de aprendizaje por defecto
    pub fn create_default_learning_loop() -> LearningLoop {
        LearningLoop::new(60)
    }

    /// Crear evento de éxito
    pub fn create_success_event(id: String, context: String, action: String, result: String, lesson: String) -> LearningEvent {
        let mut event = LearningEvent::new(id, LearningEventType::Success, context, action, result, lesson);
        event.set_importance(70);
        event
    }

    /// Crear evento de fallo
    pub fn create_failure_event(id: String, context: String, action: String, result: String, lesson: String) -> LearningEvent {
        let mut event = LearningEvent::new(id, LearningEventType::Failure, context, action, result, lesson);
        event.set_importance(80);
        event
    }

    /// Crear evento de descubrimiento
    pub fn create_discovery_event(id: String, context: String, action: String, result: String, lesson: String) -> LearningEvent {
        let mut event = LearningEvent::new(id, LearningEventType::Discovery, context, action, result, lesson);
        event.set_importance(90);
        event
    }
}
