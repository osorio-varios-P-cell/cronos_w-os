//! Metacognition Module
//! 
//! This module implements metacognition and self-improvement capabilities for AI agents.
//! Based on Microsoft AI Agents for Beginners course - Lesson 9: Metacognition.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Tipo de pensamiento metacognitivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetacognitiveThoughtType {
    /// Planificación
    Planning,
    /// Monitoreo
    Monitoring,
    /// Evaluación
    Evaluation,
    /// Autorregulación
    SelfRegulation,
    /// Reflexión
    Reflection,
}

/// Pensamiento metacognitivo
#[derive(Debug, Clone)]
pub struct MetacognitiveThought {
    /// ID del pensamiento
    pub id: String,
    /// Tipo de pensamiento
    pub thought_type: MetacognitiveThoughtType,
    /// Contenido
    pub content: String,
    /// Timestamp
    pub timestamp: u64,
    /// Confianza (0-100)
    pub confidence: u8,
}

impl MetacognitiveThought {
    /// Crear nuevo pensamiento
    pub fn new(id: String, thought_type: MetacognitiveThoughtType, content: String, confidence: u8) -> Self {
        Self {
            id,
            thought_type,
            content,
            timestamp: 0, // En un sistema real, esto sería el tiempo actual
            confidence,
        }
    }
}

/// Métrica de desempeño
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    /// Nombre de la métrica
    pub name: String,
    /// Valor actual
    pub current_value: f64,
    /// Valor objetivo
    pub target_value: f64,
    /// Historial de valores
    pub history: Vec<f64>,
    /// Mejora esperada
    pub expected_improvement: f64,
}

impl PerformanceMetric {
    /// Crear nueva métrica
    pub fn new(name: String, current_value: f64, target_value: f64) -> Self {
        Self {
            name,
            current_value,
            target_value,
            history: Vec::new(),
            expected_improvement: 0.0,
        }
    }

    /// Actualizar valor
    pub fn update(&mut self, new_value: f64) {
        self.history.push(self.current_value);
        self.current_value = new_value;
    }

    /// Calcular progreso hacia objetivo
    pub fn progress(&self) -> f64 {
        if self.target_value == 0.0 {
            0.0
        } else {
            self.current_value / self.target_value
        }
    }

    /// Calcular tasa de mejora
    pub fn improvement_rate(&self) -> f64 {
        if self.history.len() < 2 {
            0.0
        } else {
            let recent = self.history.iter().last().unwrap();
            let older = self.history.iter().nth(self.history.len() / 2).unwrap();
            (self.current_value - older) / (older - recent).abs()
        }
    }
}

/// Estrategia de mejora
#[derive(Debug, Clone)]
pub struct ImprovementStrategy {
    /// Nombre de la estrategia
    pub name: String,
    /// Descripción
    pub description: String,
    /// Métricas afectadas
    pub affected_metrics: Vec<String>,
    /// Prioridad (0-100)
    pub priority: u8,
    /// Éxito esperado
    pub expected_success: f64,
    /// Implementada
    pub implemented: bool,
}

impl ImprovementStrategy {
    /// Crear nueva estrategia
    pub fn new(name: String, description: String, affected_metrics: Vec<String>, priority: u8) -> Self {
        Self {
            name,
            description,
            affected_metrics,
            priority,
            expected_success: 0.5,
            implemented: false,
        }
    }

    /// Marcar como implementada
    pub fn mark_implemented(&mut self) {
        self.implemented = true;
    }
}

/// Sistema de metacognición
pub struct MetacognitionSystem {
    /// Pensamientos metacognitivos
    pub thoughts: Vec<MetacognitiveThought>,
    /// Métricas de desempeño
    pub metrics: Vec<PerformanceMetric>,
    /// Estrategias de mejora
    pub strategies: Vec<ImprovementStrategy>,
    /// Autoevaluación actual
    pub self_evaluation: f64,
    /// Nivel de autoconciencia (0-100)
    pub self_awareness_level: u8,
}

impl MetacognitionSystem {
    /// Crear nuevo sistema de metacognición
    pub fn new() -> Self {
        Self {
            thoughts: Vec::new(),
            metrics: Vec::new(),
            strategies: Vec::new(),
            self_evaluation: 0.5,
            self_awareness_level: 50,
        }
    }

    /// Agregar pensamiento metacognitivo
    pub fn add_thought(&mut self, thought: MetacognitiveThought) {
        self.thoughts.push(thought);
    }

    /// Agregar métrica de desempeño
    pub fn add_metric(&mut self, metric: PerformanceMetric) {
        self.metrics.push(metric);
    }

    /// Agregar estrategia de mejora
    pub fn add_strategy(&mut self, strategy: ImprovementStrategy) {
        self.strategies.push(strategy);
    }

    /// Realizar autoevaluación
    pub fn self_evaluate(&mut self) -> f64 {
        if self.metrics.is_empty() {
            return 0.5;
        }

        let total_progress: f64 = self.metrics.iter()
            .map(|m| m.progress())
            .sum();

        let avg_progress = total_progress / self.metrics.len() as f64;
        self.self_evaluation = avg_progress;
        avg_progress
    }

    /// Generar plan de mejora
    pub fn generate_improvement_plan(&self) -> Vec<&ImprovementStrategy> {
        let mut strategies: Vec<&ImprovementStrategy> = self.strategies.iter()
            .filter(|s| !s.implemented)
            .collect();

        // Ordenar por prioridad
        strategies.sort_by(|a, b| b.priority.cmp(&a.priority));

        strategies
    }

    /// Implementar estrategia de mejora
    pub fn implement_strategy(&mut self, strategy_name: &str) -> Result<(), String> {
        let strategy = self.strategies.iter_mut()
            .find(|s| s.name == strategy_name)
            .ok_or_else(|| String::from("Strategy not found"))?;

        strategy.mark_implemented();
        Ok(())
    }

    /// Analizar patrones de desempeño
    pub fn analyze_performance_patterns(&self) -> String {
        let mut analysis = String::from("Performance Pattern Analysis\n");
        analysis.push_str("============================\n\n");

        for metric in &self.metrics {
            analysis.push_str(&format!("Metric: {}\n", metric.name));
            analysis.push_str(&format!("  Current: {:.2}\n", metric.current_value));
            analysis.push_str(&format!("  Target: {:.2}\n", metric.target_value));
            analysis.push_str(&format!("  Progress: {:.2}%\n", metric.progress() * 100.0));
            analysis.push_str(&format!("  Improvement Rate: {:.2}\n\n", metric.improvement_rate()));
        }

        analysis
    }

    /// Generar reporte de autoconciencia
    pub fn generate_self_awareness_report(&self) -> String {
        let mut report = String::from("Self-Awareness Report\n");
        report.push_str("=====================\n\n");

        report.push_str(&format!("Self-Awareness Level: {}\n", self.self_awareness_level));
        report.push_str(&format!("Self-Evaluation: {:.2}\n\n", self.self_evaluation));

        report.push_str("Recent Metacognitive Thoughts:\n");
        for thought in self.thoughts.iter().rev().take(5) {
            report.push_str(&format!("  - {:?}: {} (Confidence: {})\n", 
                thought.thought_type, thought.content, thought.confidence));
        }

        report.push('\n');

        report.push_str("Improvement Strategies:\n");
        for strategy in &self.strategies {
            report.push_str(&format!("  - {} (Priority: {}, Implemented: {})\n", 
                strategy.name, strategy.priority, strategy.implemented));
        }

        report
    }

    /// Actualizar nivel de autoconciencia
    pub fn update_self_awareness(&mut self, new_level: u8) {
        self.self_awareness_level = new_level.min(100);
    }

    /// Obtener pensamientos por tipo
    pub fn get_thoughts_by_type(&self, thought_type: MetacognitiveThoughtType) -> Vec<&MetacognitiveThought> {
        self.thoughts.iter()
            .filter(|t| t.thought_type == thought_type)
            .collect()
    }

    /// Generar reflexión sobre desempeño
    pub fn generate_reflection(&self) -> String {
        let mut reflection = String::from("Performance Reflection\n");
        reflection.push_str("=====================\n\n");

        reflection.push_str(&format!("Overall Self-Evaluation: {:.2}\n", self.self_evaluation));
        reflection.push_str(&format!("Self-Awareness Level: {}\n\n", self.self_awareness_level));

        reflection.push_str("Key Insights:\n");
        
        // Analizar métricas con bajo progreso
        let low_progress: Vec<&PerformanceMetric> = self.metrics.iter()
            .filter(|m| m.progress() < 0.5)
            .collect();

        if !low_progress.is_empty() {
            reflection.push_str("  - Areas needing improvement:\n");
            for metric in low_progress {
                reflection.push_str(&format!("    * {} (Progress: {:.2}%)\n", 
                    metric.name, metric.progress() * 100.0));
            }
        }

        // Analizar métricas con alto progreso
        let high_progress: Vec<&PerformanceMetric> = self.metrics.iter()
            .filter(|m| m.progress() >= 0.8)
            .collect();

        if !high_progress.is_empty() {
            reflection.push_str("  - Areas performing well:\n");
            for metric in high_progress {
                reflection.push_str(&format!("    * {} (Progress: {:.2}%)\n", 
                    metric.name, metric.progress() * 100.0));
            }
        }

        reflection
    }
}

impl Default for MetacognitionSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de metacognición
pub struct MetacognitionUtils;

impl MetacognitionUtils {
    /// Crear sistema de metacognición por defecto
    pub fn create_default_metacognition_system() -> MetacognitionSystem {
        let mut system = MetacognitionSystem::new();

        // Agregar métricas por defecto
        system.add_metric(PerformanceMetric::new(
            String::from("task_completion_rate"),
            0.5,
            0.9,
        ));

        system.add_metric(PerformanceMetric::new(
            String::from("decision_accuracy"),
            0.6,
            0.95,
        ));

        system.add_metric(PerformanceMetric::new(
            String::from("learning_rate"),
            0.4,
            0.8,
        ));

        // Agregar estrategias por defecto
        system.add_strategy(ImprovementStrategy::new(
            String::from("increase_practice"),
            String::from("Practice more tasks to improve completion rate"),
            vec![String::from("task_completion_rate")],
            80,
        ));

        system.add_strategy(ImprovementStrategy::new(
            String::from("improve_context_analysis"),
            String::from("Analyze context more carefully for better decisions"),
            vec![String::from("decision_accuracy")],
            90,
        ));

        system.add_strategy(ImprovementStrategy::new(
            String::from("accelerate_learning"),
            String::from("Use more efficient learning methods"),
            vec![String::from("learning_rate")],
            85,
        ));

        system
    }

    /// Crear pensamiento de planificación
    pub fn create_planning_thought(id: String, content: String) -> MetacognitiveThought {
        MetacognitiveThought::new(id, MetacognitiveThoughtType::Planning, content, 70)
    }

    /// Crear pensamiento de monitoreo
    pub fn create_monitoring_thought(id: String, content: String) -> MetacognitiveThought {
        MetacognitiveThought::new(id, MetacognitiveThoughtType::Monitoring, content, 80)
    }

    /// Crear pensamiento de evaluación
    pub fn create_evaluation_thought(id: String, content: String) -> MetacognitiveThought {
        MetacognitiveThought::new(id, MetacognitiveThoughtType::Evaluation, content, 75)
    }
}
