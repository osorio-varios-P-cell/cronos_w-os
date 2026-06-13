//! User Modeling Module
//! 
//! This module implements deep user modeling for AI agents based on Hermes Agent architecture.
//! Allows agents to build a comprehensive model of user preferences, behaviors, and patterns.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Tipo de preferencia de usuario
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserPreferenceType {
    /// Preferencia de interfaz
    Interface,
    /// Preferencia de comunicación
    Communication,
    /// Preferencia de estilo
    Style,
    /// Preferencia de contenido
    Content,
    /// Preferencia de tiempo
    Timing,
    /// Preferencia personalizada
    Custom,
}

/// Preferencia de usuario
#[derive(Debug, Clone)]
pub struct UserPreference {
    /// Tipo de preferencia
    pub preference_type: UserPreferenceType,
    /// Clave
    pub key: String,
    /// Valor
    pub value: String,
    /// Peso (0-100)
    pub weight: u8,
    /// Confianza (0-100)
    pub confidence: u8,
}

impl UserPreference {
    /// Crear nueva preferencia
    pub fn new(preference_type: UserPreferenceType, key: String, value: String, weight: u8) -> Self {
        Self {
            preference_type,
            key,
            value,
            weight,
            confidence: 50,
        }
    }

    /// Actualizar confianza
    pub fn update_confidence(&mut self, new_confidence: u8) {
        self.confidence = new_confidence.min(100);
    }

    /// Calcular puntuación de importancia
    pub fn importance_score(&self) -> f64 {
        (self.weight as f64 / 100.0) * (self.confidence as f64 / 100.0)
    }
}

/// Patrón de comportamiento
#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    /// ID del patrón
    pub id: String,
    /// Nombre del patrón
    pub name: String,
    /// Descripción
    pub description: String,
    /// Condiciones del patrón
    pub conditions: Vec<String>,
    /// Acción esperada
    pub expected_action: String,
    /// Frecuencia del patrón
    pub frequency: f64,
    /// Confianza en el patrón
    pub confidence: f64,
}

impl BehaviorPattern {
    /// Crear nuevo patrón de comportamiento
    pub fn new(id: String, name: String, description: String, conditions: Vec<String>, expected_action: String) -> Self {
        Self {
            id,
            name,
            description,
            conditions,
            expected_action,
            frequency: 0.0,
            confidence: 0.5,
        }
    }

    /// Actualizar frecuencia
    pub fn update_frequency(&mut self, new_frequency: f64) {
        self.frequency = new_frequency;
    }

    /// Actualizar confianza
    pub fn update_confidence(&mut self, new_confidence: f64) {
        self.confidence = new_confidence.min(1.0);
    }
}

/// Perfil de usuario
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// ID del usuario
    pub user_id: String,
    /// Nombre del usuario
    pub name: String,
    /// Preferencias del usuario
    pub preferences: Vec<UserPreference>,
    /// Patrones de comportamiento
    pub behavior_patterns: Vec<BehaviorPattern>,
    /// Historial de interacciones
    pub interaction_history: Vec<String>,
    /// Metadatos del usuario
    pub metadata: Vec<(String, String)>,
    /// Timestamp de creación
    pub created_at: u64,
    /// Timestamp de última actualización
    pub updated_at: u64,
}

impl UserProfile {
    /// Crear nuevo perfil de usuario
    pub fn new(user_id: String, name: String) -> Self {
        Self {
            user_id,
            name,
            preferences: Vec::new(),
            behavior_patterns: Vec::new(),
            interaction_history: Vec::new(),
            metadata: Vec::new(),
            created_at: 0,
            updated_at: 0,
        }
    }

    /// Agregar preferencia
    pub fn add_preference(&mut self, preference: UserPreference) {
        // Remover si ya existe
        self.preferences.retain(|p| p.key != preference.key);
        self.preferences.push(preference);
        self.updated_at = 0;
    }

    /// Obtener preferencia por clave
    pub fn get_preference(&self, key: &str) -> Option<&UserPreference> {
        self.preferences.iter().find(|p| p.key == key)
    }

    /// Agregar patrón de comportamiento
    pub fn add_behavior_pattern(&mut self, pattern: BehaviorPattern) {
        self.behavior_patterns.push(pattern);
        self.updated_at = 0;
    }

    /// Agregar interacción al historial
    pub fn add_interaction(&mut self, interaction: String) {
        self.interaction_history.push(interaction);
        self.updated_at = 0;
    }

    /// Agregar metadato
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.push((key, value));
    }

    /// Obtener metadato
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// Modelo profundo del usuario
pub struct DeepUserModel {
    /// Perfil del usuario
    pub profile: UserProfile,
    /// Modelo de personalidad
    pub personality_model: Vec<(String, f64)>,
    /// Modelo de estilo de comunicación
    pub communication_style: Vec<(String, f64)>,
    /// Modelo de toma de decisiones
    pub decision_model: Vec<(String, f64)>,
    /// Modelo de aprendizaje
    pub learning_model: Vec<(String, f64)>,
    /// Nivel de confianza del modelo (0-100)
    pub model_confidence: u8,
}

impl DeepUserModel {
    /// Crear nuevo modelo profundo de usuario
    pub fn new(profile: UserProfile) -> Self {
        Self {
            profile,
            personality_model: Vec::new(),
            communication_style: Vec::new(),
            decision_model: Vec::new(),
            learning_model: Vec::new(),
            model_confidence: 50,
        }
    }

    /// Actualizar modelo de personalidad
    pub fn update_personality_model(&mut self, trait_name: String, value: f64) {
        // Remover si ya existe
        self.personality_model.retain(|(t, _)| t != &trait_name);
        self.personality_model.push((trait_name, value));
    }

    /// Actualizar modelo de comunicación
    pub fn update_communication_style(&mut self, style_name: String, value: f64) {
        self.communication_style.retain(|(s, _)| s != &style_name);
        self.communication_style.push((style_name, value));
    }

    /// Actualizar modelo de toma de decisiones
    pub fn update_decision_model(&mut self, decision_name: String, value: f64) {
        self.decision_model.retain(|(d, _)| d != &decision_name);
        self.decision_model.push((decision_name, value));
    }

    /// Actualizar modelo de aprendizaje
    pub fn update_learning_model(&mut self, learning_name: String, value: f64) {
        self.learning_model.retain(|(l, _)| l != &learning_name);
        self.learning_model.push((learning_name, value));
    }

    /// Actualizar confianza del modelo
    pub fn update_model_confidence(&mut self, new_confidence: u8) {
        self.model_confidence = new_confidence.min(100);
    }

    /// Predecir comportamiento del usuario
    pub fn predict_behavior(&self, context: &str) -> String {
        // En un sistema real, esto usaría los modelos para predecir comportamiento
        let _ = context;
        format!("Predicted behavior based on model confidence: {}", self.model_confidence)
    }

    /// Generar recomendación personalizada
    pub fn generate_recommendation(&self) -> String {
        let mut recommendation = String::from("Personalized Recommendation:\n");
        
        // Basado en preferencias
        for pref in &self.profile.preferences {
            if pref.importance_score() > 0.5 {
                recommendation.push_str(&format!("  - {}: {}\n", pref.key, pref.value));
            }
        }
        
        recommendation
    }

    /// Generar reporte del modelo
    pub fn generate_model_report(&self) -> String {
        let mut report = String::from("Deep User Model Report\n");
        report.push_str("======================\n\n");
        
        report.push_str(&format!("User ID: {}\n", self.profile.user_id));
        report.push_str(&format!("User Name: {}\n", self.profile.name));
        report.push_str(&format!("Model Confidence: {}\n\n", self.model_confidence));
        
        report.push_str("Personality Model:\n");
        for (trait_name, value) in &self.personality_model {
            report.push_str(&format!("  {}: {:.2}\n", trait_name, value));
        }
        
        report.push('\n');
        
        report.push_str("Communication Style:\n");
        for (style_name, value) in &self.communication_style {
            report.push_str(&format!("  {}: {:.2}\n", style_name, value));
        }
        
        report.push('\n');
        
        report.push_str("Decision Model:\n");
        for (decision_name, value) in &self.decision_model {
            report.push_str(&format!("  {}: {:.2}\n", decision_name, value));
        }
        
        report.push('\n');
        
        report.push_str("Learning Model:\n");
        for (learning_name, value) in &self.learning_model {
            report.push_str(&format!("  {}: {:.2}\n", learning_name, value));
        }
        
        report
    }
}

/// Utilidades de modelado de usuario
pub struct UserModelingUtils;

impl UserModelingUtils {
    /// Crear modelo profundo de usuario por defecto
    pub fn create_default_deep_user_model(user_id: String, name: String) -> DeepUserModel {
        let profile = UserProfile::new(user_id.clone(), name);
        let mut model = DeepUserModel::new(profile);
        
        // Inicializar modelos por defecto
        model.update_personality_model(String::from("openness"), 0.7);
        model.update_personality_model(String::from("conscientiousness"), 0.6);
        model.update_personality_model(String::from("extraversion"), 0.5);
        
        model.update_communication_style(String::from("formal"), 0.4);
        model.update_communication_style(String::from("direct"), 0.6);
        
        model.update_decision_model(String::from("analytical"), 0.7);
        model.update_decision_model(String::from("intuitive"), 0.5);
        
        model.update_learning_model(String::from("visual"), 0.6);
        model.update_learning_model(String::from("auditory"), 0.4);
        
        model
    }

    /// Crear preferencia por defecto
    pub fn create_default_preference(key: String, value: String) -> UserPreference {
        UserPreference::new(UserPreferenceType::Custom, key, value, 50)
    }

    /// Crear patrón de comportamiento por defecto
    pub fn create_default_behavior_pattern(id: String, name: String, description: String) -> BehaviorPattern {
        BehaviorPattern::new(
            id,
            name,
            description,
            vec![String::from("default_condition")],
            String::from("default_action"),
        )
    }
}
