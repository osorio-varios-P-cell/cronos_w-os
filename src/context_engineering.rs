//! Context Engineering Module
//! 
//! This module implements advanced context engineering for AI agents.
//! Based on Microsoft AI Agents for Beginners course - Lesson 12: Context Engineering.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de contexto
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    /// Contexto de sistema
    System,
    /// Contexto de usuario
    User,
    /// Contexto de tarea
    Task,
    /// Contexto de conversación
    Conversation,
    /// Contexto de entorno
    Environment,
    /// Contexto personalizado
    Custom,
}

/// Componente de contexto
#[derive(Debug, Clone)]
pub struct ContextComponent {
    /// ID del componente
    pub id: String,
    /// Tipo de contexto
    pub context_type: ContextType,
    /// Clave
    pub key: String,
    /// Valor
    pub value: String,
    /// Peso (0-100)
    pub weight: u8,
    /// TTL (time-to-live) en segundos
    pub ttl: u64,
    /// Timestamp de creación
    pub created_at: u64,
}

impl ContextComponent {
    /// Crear nuevo componente de contexto
    pub fn new(id: String, context_type: ContextType, key: String, value: String, weight: u8, ttl: u64) -> Self {
        Self {
            id,
            context_type,
            key,
            value,
            weight,
            ttl,
            created_at: 0,
        }
    }

    /// Calcular puntuación de relevancia
    pub fn relevance_score(&self, current_time: u64) -> f64 {
        let age = current_time - self.created_at;
        let ttl_factor = if age < self.ttl { 1.0 } else { 0.5 };
        let weight_factor = self.weight as f64 / 100.0;
        
        ttl_factor * weight_factor
    }

    /// Verificar si está expirado
    pub fn is_expired(&self, current_time: u64) -> bool {
        (current_time - self.created_at) > self.ttl
    }
}

/// Plantilla de contexto
#[derive(Debug, Clone)]
pub struct ContextTemplate {
    /// ID de la plantilla
    pub id: String,
    /// Nombre de la plantilla
    pub name: String,
    /// Descripción
    pub description: String,
    /// Componentes de contexto
    pub components: Vec<ContextComponent>,
    /// Variables de plantilla
    pub variables: Vec<String>,
}

impl ContextTemplate {
    /// Crear nueva plantilla de contexto
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            components: Vec::new(),
            variables: Vec::new(),
        }
    }

    /// Agregar componente
    pub fn add_component(&mut self, component: ContextComponent) {
        self.components.push(component);
    }

    /// Agregar variable
    pub fn add_variable(&mut self, variable: String) {
        self.variables.push(variable);
    }

    /// Renderizar plantilla con valores
    pub fn render(&self, values: &[(String, String)]) -> String {
        let mut context = String::from(&self.description);
        context.push('\n');
        
        for component in &self.components {
            context.push_str(&format!("{}: {}\n", component.key, component.value));
        }
        
        // Reemplazar variables
        for (var_name, var_value) in values {
            context = context.replace(&format!("{{{}}}", var_name), var_value);
        }
        
        context
    }
}

/// Sistema de ingeniería de contexto
#[derive(Debug, Clone)]
pub struct ContextEngineeringSystem {
    /// Componentes de contexto activos
    pub active_components: Vec<ContextComponent>,
    /// Plantillas de contexto
    pub templates: Vec<ContextTemplate>,
    /// Historial de contextos
    pub context_history: Vec<String>,
    /// Tiempo actual
    pub current_time: u64,
}

impl ContextEngineeringSystem {
    /// Crear nuevo sistema de ingeniería de contexto
    pub fn new() -> Self {
        Self {
            active_components: Vec::new(),
            templates: Vec::new(),
            context_history: Vec::new(),
            current_time: 0,
        }
    }

    /// Agregar componente de contexto
    pub fn add_component(&mut self, component: ContextComponent) {
        self.active_components.push(component);
    }

    /// Agregar plantilla de contexto
    pub fn add_template(&mut self, template: ContextTemplate) {
        self.templates.push(template);
    }

    /// Obtener componente por clave
    pub fn get_component(&self, key: &str) -> Option<&ContextComponent> {
        self.active_components.iter().find(|c| c.key == key)
    }

    /// Obtener componentes por tipo
    pub fn get_components_by_type(&self, context_type: ContextType) -> Vec<&ContextComponent> {
        self.active_components.iter()
            .filter(|c| c.context_type == context_type)
            .collect()
    }

    /// Eliminar componentes expirados
    pub fn remove_expired_components(&mut self) {
        self.active_components.retain(|c| !c.is_expired(self.current_time));
    }

    /// Construir contexto para un tipo específico
    pub fn build_context(&mut self, context_type: ContextType) -> String {
        let components: Vec<&ContextComponent> = self.get_components_by_type(context_type);
        
        let mut context = String::new();
        
        // Ordenar por peso
        let mut sorted_components: Vec<&ContextComponent> = components.into_iter().collect();
        sorted_components.sort_by(|a, b| b.weight.cmp(&a.weight));
        
        for component in sorted_components {
            if !component.is_expired(self.current_time) {
                context.push_str(&format!("{}: {}\n", component.key, component.value));
            }
        }
        
        // Guardar en historial
        self.context_history.push(context.clone());
        
        context
    }

    /// Construir contexto desde plantilla
    pub fn build_context_from_template(&self, template_id: &str, values: &[(String, String)]) -> Result<String, String> {
        let template = self.templates.iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| String::from("Template not found"))?;
        
        Ok(template.render(values))
    }

    /// Optimizar contexto
    pub fn optimize_context(&mut self) {
        // Eliminar componentes expirados
        self.remove_expired_components();
        
        // Eliminar componentes duplicados
        let mut seen_keys = Vec::new();
        self.active_components.retain(|c| {
            if seen_keys.contains(&c.key) {
                false
            } else {
                seen_keys.push(c.key.clone());
                true
            }
        });
    }

    /// Actualizar tiempo
    pub fn update_time(&mut self, new_time: u64) {
        self.current_time = new_time;
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Context Engineering Status\n");
        report.push_str("=========================\n\n");
        
        report.push_str(&format!("Active Components: {}\n", self.active_components.len()));
        report.push_str(&format!("Templates: {}\n", self.templates.len()));
        report.push_str(&format!("Context History: {}\n\n", self.context_history.len()));
        
        report.push_str("Components by Type:\n");
        for context_type in &[ContextType::System, ContextType::User, ContextType::Task, ContextType::Conversation, ContextType::Environment] {
            let count = self.get_components_by_type(*context_type).len();
            report.push_str(&format!("  {:?}: {}\n", context_type, count));
        }
        
        report
    }
}

impl Default for ContextEngineeringSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de ingeniería de contexto
pub struct ContextEngineeringUtils;

impl ContextEngineeringUtils {
    /// Crear sistema de ingeniería de contexto por defecto
    pub fn create_default_context_system() -> ContextEngineeringSystem {
        let mut system = ContextEngineeringSystem::new();
        
        // Agregar componentes de contexto por defecto
        system.add_component(ContextComponent::new(
            String::from("sys_1"),
            ContextType::System,
            String::from("os_name"),
            String::from("Cronos W-OS"),
            90,
            3600,
        ));
        
        system.add_component(ContextComponent::new(
            String::from("sys_2"),
            ContextType::System,
            String::from("kernel_version"),
            String::from("1.0.0"),
            80,
            3600,
        ));
        
        system.add_component(ContextComponent::new(
            String::from("user_1"),
            ContextType::User,
            String::from("user_preference"),
            String::from("default"),
            70,
            1800,
        ));
        
        // Agregar plantilla por defecto
        let mut template = ContextTemplate::new(
            String::from("template_1"),
            String::from("Standard Context"),
            String::from("Standard context template for general use"),
        );
        template.add_variable(String::from("user_name"));
        template.add_variable(String::from("task"));
        system.add_template(template);
        
        system
    }

    /// Crear componente de contexto por defecto
    pub fn create_default_component(id: String, context_type: ContextType, key: String, value: String) -> ContextComponent {
        ContextComponent::new(id, context_type, key, value, 50, 3600)
    }

    /// Crear plantilla de contexto por defecto
    pub fn create_default_template(id: String, name: String, description: String) -> ContextTemplate {
        ContextTemplate::new(id, name, description)
    }
}
