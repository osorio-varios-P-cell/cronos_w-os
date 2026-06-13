//! Pattern Learning Module
//! 
//! This module implements pattern learning for system optimization by analyzing
//! hardware usage patterns and providing adaptive recommendations.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de patrón
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Patrón de temperatura
    Temperature,
    /// Patrón de uso de CPU
    CpuUsage,
    /// Patrón de uso de memoria
    MemoryUsage,
    /// Patrón de actividad de I/O
    IoActivity,
    /// Patrón de energía
    Power,
}

/// Patrón aprendido
#[derive(Debug, Clone)]
pub struct LearnedPattern {
    /// Tipo de patrón
    pub pattern_type: PatternType,
    /// Nombre del patrón
    pub name: String,
    /// Valores del patrón
    pub values: Vec<f32>,
    /// Frecuencia del patrón (0.0 - 1.0)
    pub frequency: f32,
    /// Confianza del patrón (0.0 - 1.0)
    pub confidence: f32,
    /// Timestamp del último avistamiento
    pub last_seen: u64,
}

impl LearnedPattern {
    /// Crear un nuevo patrón aprendido
    pub fn new(pattern_type: PatternType, name: String, values: Vec<f32>) -> Self {
        Self {
            pattern_type,
            name,
            values,
            frequency: 0.0,
            confidence: 0.0,
            last_seen: 0,
        }
    }

    /// Actualizar la frecuencia del patrón
    pub fn update_frequency(&mut self, increment: f32) {
        self.frequency = (self.frequency + increment).min(1.0);
    }

    /// Actualizar la confianza del patrón
    pub fn update_confidence(&mut self, increment: f32) {
        self.confidence = (self.confidence + increment).min(1.0);
    }

    /// Verificar si el patrón es confiable
    pub fn is_reliable(&self) -> bool {
        self.confidence > 0.7 && self.frequency > 0.5
    }
}

/// Recomendación de optimización
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Tipo de recomendación
    pub recommendation_type: String,
    /// Descripción
    pub description: String,
    /// Impacto esperado (0.0 - 1.0)
    pub expected_impact: f32,
    /// Prioridad
    pub priority: u8,
}

impl OptimizationRecommendation {
    /// Crear una nueva recomendación
    pub fn new(recommendation_type: String, description: String, expected_impact: f32, priority: u8) -> Self {
        Self {
            recommendation_type,
            description,
            expected_impact,
            priority,
        }
    }
}

/// Estadísticas de uso
#[derive(Debug, Clone)]
pub struct UsageStatistics {
    /// Promedio
    pub average: f32,
    /// Mínimo
    pub minimum: f32,
    /// Máximo
    pub maximum: f32,
    /// Desviación estándar
    pub std_dev: f32,
    /// Mediana
    pub median: f32,
}

impl UsageStatistics {
    /// Calcular raíz cuadrada usando el método de Newton-Raphson
    fn sqrt_f32(n: f32) -> f32 {
        if n <= 0.0 {
            return 0.0;
        }
        
        let mut x = n;
        let mut y = 1.0;
        let e = 0.000001;
        
        while x - y > e {
            x = (x + y) / 2.0;
            y = n / x;
        }
        
        x
    }

    /// Calcular estadísticas de un conjunto de valores
    pub fn calculate(values: &[f32]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let sum: f32 = values.iter().sum();
        let average = sum / values.len() as f32;
        
        let minimum = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let maximum = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        
        let variance: f32 = values.iter()
            .map(|x| {
                let diff = x - average;
                diff * diff
            })
            .sum::<f32>() / values.len() as f32;
        let std_dev = Self::sqrt_f32(variance);
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[sorted_values.len() / 2 - 1] + sorted_values[sorted_values.len() / 2]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };

        Some(Self {
            average,
            minimum,
            maximum,
            std_dev,
            median,
        })
    }
}

/// Sistema de aprendizaje de patrones
pub struct PatternLearningSystem {
    /// Patrones aprendidos
    learned_patterns: Vec<LearnedPattern>,
    /// Historial de datos
    data_history: Vec<(PatternType, Vec<f32>)>,
    /// Habilitar aprendizaje
    learning_enabled: bool,
    /// Umbral de confianza para aplicar patrones
    confidence_threshold: f32,
}

impl PatternLearningSystem {
    /// Crear un nuevo sistema de aprendizaje
    pub fn new() -> Self {
        Self {
            learned_patterns: Vec::new(),
            data_history: Vec::new(),
            learning_enabled: true,
            confidence_threshold: 0.7,
        }
    }

    /// Agregar datos para aprendizaje
    pub fn add_data(&mut self, pattern_type: PatternType, values: Vec<f32>) {
        self.data_history.push((pattern_type, values.clone()));
        
        // Mantener solo los últimos 1000 entradas
        if self.data_history.len() > 1000 {
            self.data_history.drain(0..self.data_history.len() - 1000);
        }

        if self.learning_enabled {
            self.learn_from_data(pattern_type, values);
        }
    }

    /// Aprender de los datos
    fn learn_from_data(&mut self, pattern_type: PatternType, values: Vec<f32>) {
        // Buscar patrones existentes similares
        for pattern in &mut self.learned_patterns {
            if pattern.pattern_type == pattern_type {
                // Calcular similitud
                let similarity = Self::calculate_similarity(&pattern.values, &values);
                
                if similarity > 0.8 {
                    // Patrón similar encontrado, actualizar
                    pattern.update_frequency(0.1);
                    pattern.update_confidence(0.05);
                    pattern.last_seen = 0; // En un sistema real, timestamp actual
                    return;
                }
            }
        }

        // No se encontró patrón similar, crear uno nuevo
        let new_pattern = LearnedPattern::new(
            pattern_type,
            format!("{:?} pattern {}", pattern_type, self.learned_patterns.len()),
            values,
        );
        self.learned_patterns.push(new_pattern);
    }

    /// Calcular similitud entre dos conjuntos de valores
    fn calculate_similarity(values1: &[f32], values2: &[f32]) -> f32 {
        if values1.len() != values2.len() {
            return 0.0;
        }

        let mut sum_diff = 0.0;
        for (v1, v2) in values1.iter().zip(values2.iter()) {
            sum_diff += (v1 - v2).abs();
        }

        let avg_diff = sum_diff / values1.len() as f32;
        1.0 - (avg_diff / 100.0).min(1.0) // Normalizar
    }

    /// Obtener patrones por tipo
    pub fn get_patterns_by_type(&self, pattern_type: PatternType) -> Vec<&LearnedPattern> {
        self.learned_patterns.iter()
            .filter(|p| p.pattern_type == pattern_type)
            .collect()
    }

    /// Obtener patrones confiables
    pub fn get_reliable_patterns(&self) -> Vec<&LearnedPattern> {
        self.learned_patterns.iter()
            .filter(|p| p.is_reliable())
            .collect()
    }

    /// Generar recomendaciones de optimización
    pub fn generate_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        let reliable_patterns = self.get_reliable_patterns();

        for pattern in reliable_patterns {
            match pattern.pattern_type {
                PatternType::Temperature => {
                    if pattern.values.iter().any(|v| v > &80.0) {
                        recommendations.push(OptimizationRecommendation::new(
                            String::from("Thermal"),
                            String::from("Consider increasing fan speed or reducing CPU load during peak temperature periods"),
                            0.8,
                            8,
                        ));
                    }
                }
                PatternType::CpuUsage => {
                    let stats = UsageStatistics::calculate(&pattern.values);
                    if let Some(s) = stats {
                        if s.average > 80.0 {
                            recommendations.push(OptimizationRecommendation::new(
                                String::from("Performance"),
                                String::from("CPU usage is consistently high, consider optimizing workloads or adding more cores"),
                                0.7,
                                7,
                            ));
                        }
                    }
                }
                PatternType::MemoryUsage => {
                    let stats = UsageStatistics::calculate(&pattern.values);
                    if let Some(s) = stats {
                        if s.average > 80.0 {
                            recommendations.push(OptimizationRecommendation::new(
                                String::from("Memory"),
                                String::from("Memory usage is consistently high, consider adding more RAM or optimizing memory usage"),
                                0.6,
                                6,
                            ));
                        }
                    }
                }
                PatternType::IoActivity => {
                    let stats = UsageStatistics::calculate(&pattern.values);
                    if let Some(s) = stats {
                        if s.std_dev > s.average * 0.5 {
                            recommendations.push(OptimizationRecommendation::new(
                                String::from("I/O"),
                                String::from("I/O activity has high variance, consider implementing I/O scheduling optimizations"),
                                0.5,
                                5,
                            ));
                        }
                    }
                }
                PatternType::Power => {
                    let stats = UsageStatistics::calculate(&pattern.values);
                    if let Some(s) = stats {
                        if s.average > 80.0 {
                            recommendations.push(OptimizationRecommendation::new(
                                String::from("Power"),
                                String::from("Power consumption is consistently high, consider enabling power saving features"),
                                0.6,
                                6,
                            ));
                        }
                    }
                }
            }
        }

        recommendations
    }

    /// Predecir el siguiente valor basado en patrones
    pub fn predict_next_value(&self, pattern_type: PatternType, current_values: &[f32]) -> Option<f32> {
        let patterns = self.get_patterns_by_type(pattern_type);
        
        for pattern in patterns {
            if pattern.is_reliable() {
                let similarity = Self::calculate_similarity(&pattern.values, current_values);
                if similarity > 0.8 {
                    // Retornar el último valor del patrón como predicción
                    return pattern.values.last().copied();
                }
            }
        }

        None
    }

    /// Obtener estadísticas de un tipo de patrón
    pub fn get_pattern_statistics(&self, pattern_type: PatternType) -> Option<UsageStatistics> {
        let values: Vec<f32> = self.data_history.iter()
            .filter(|(pt, _)| *pt == pattern_type)
            .flat_map(|(_, v)| v.clone())
            .collect();

        UsageStatistics::calculate(&values)
    }

    /// Habilitar/deshabilitar aprendizaje
    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
    }

    /// Establecer el umbral de confianza
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold;
    }

    /// Obtener el número de patrones aprendidos
    pub fn pattern_count(&self) -> usize {
        self.learned_patterns.len()
    }

    /// Obtener el número de patrones confiables
    pub fn reliable_pattern_count(&self) -> usize {
        self.learned_patterns.iter()
            .filter(|p| p.is_reliable())
            .count()
    }
}

impl Default for PatternLearningSystem {
    fn default() -> Self {
        Self::new()
    }
}
