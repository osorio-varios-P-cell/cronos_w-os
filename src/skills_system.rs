//! Skills System Module
//! 
//! This module implements a skills system for AI agents based on Hermes Agent architecture.
//! Skills are reusable capabilities that agents can learn, improve, and share.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de habilidad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillType {
    /// Habilidad de análisis
    Analysis,
    /// Habilidad de planificación
    Planning,
    /// Habilidad de ejecución
    Execution,
    /// Habilidad de comunicación
    Communication,
    /// Habilidad de aprendizaje
    Learning,
    /// Habilidad de optimización
    Optimization,
    /// Habilidad personalizada
    Custom,
}

/// Estado de la habilidad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillState {
    /// No aprendida
    Unlearned,
    /// En aprendizaje
    Learning,
    /// Aprendida
    Learned,
    /// Mejorando
    Improving,
    /// Maestra
    Mastered,
}

/// Habilidad
#[derive(Debug, Clone)]
pub struct Skill {
    /// ID de la habilidad
    pub id: String,
    /// Nombre de la habilidad
    pub name: String,
    /// Tipo de habilidad
    pub skill_type: SkillType,
    /// Estado actual
    pub state: SkillState,
    /// Descripción
    pub description: String,
    /// Nivel de maestría (0-100)
    pub mastery_level: u8,
    /// Número de usos
    pub usage_count: u32,
    /// Tasa de éxito (0-100)
    pub success_rate: u8,
    /// Parámetros de la habilidad
    pub parameters: Vec<(String, String)>,
    /// Código de la habilidad (simulado)
    pub code: String,
    /// Timestamp de creación
    pub created_at: u64,
    /// Timestamp de última actualización
    pub updated_at: u64,
}

impl Skill {
    /// Crear nueva habilidad
    pub fn new(id: String, name: String, skill_type: SkillType, description: String) -> Self {
        Self {
            id,
            name,
            skill_type,
            state: SkillState::Unlearned,
            description,
            mastery_level: 0,
            usage_count: 0,
            success_rate: 0,
            parameters: Vec::new(),
            code: String::new(),
            created_at: 0,
            updated_at: 0,
        }
    }

    /// Agregar parámetro
    pub fn add_parameter(&mut self, key: String, value: String) {
        self.parameters.push((key, value));
    }

    /// Establecer código
    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }

    /// Usar habilidad
    pub fn use_skill(&mut self) -> Result<String, String> {
        if self.state == SkillState::Unlearned {
            return Err(String::from("Skill not learned yet"));
        }

        self.usage_count += 1;
        
        // En un sistema real, esto ejecutaría el código de la habilidad
        Ok(format!("Executed skill: {}", self.name))
    }

    /// Actualizar tasa de éxito
    pub fn update_success_rate(&mut self, success: bool) {
        let total = self.usage_count as f64;
        if total > 0.0 {
            let current_success = self.success_rate as f64 / 100.0 * (total - 1.0);
            let new_success = if success { 1.0 } else { 0.0 };
            let updated_rate = (current_success + new_success) / total;
            self.success_rate = (updated_rate * 100.0) as u8;
        }
    }

    /// Mejorar nivel de maestría
    pub fn improve_mastery(&mut self, improvement: u8) {
        self.mastery_level = (self.mastery_level + improvement).min(100);
        
        // Actualizar estado basado en nivel de maestría
        self.state = if self.mastery_level >= 90 {
            SkillState::Mastered
        } else if self.mastery_level >= 70 {
            SkillState::Improving
        } else if self.mastery_level >= 50 {
            SkillState::Learned
        } else if self.mastery_level >= 20 {
            SkillState::Learning
        } else {
            SkillState::Unlearned
        };
    }

    /// Calcular puntuación de calidad
    pub fn quality_score(&self) -> f64 {
        let mastery_factor = self.mastery_level as f64 / 100.0;
        let success_factor = self.success_rate as f64 / 100.0;
        let usage_factor = (self.usage_count as f64 + 1.0) / 10.0;
        
        mastery_factor * success_factor * (1.0 + usage_factor)
    }
}

/// Biblioteca de habilidades
pub struct SkillLibrary {
    /// Habilidades disponibles
    pub skills: Vec<Skill>,
    /// Índice por tipo
    pub type_index: Vec<(SkillType, usize)>,
    /// Índice por nombre
    pub name_index: Vec<(String, usize)>,
}

impl SkillLibrary {
    /// Crear nueva biblioteca de habilidades
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            type_index: Vec::new(),
            name_index: Vec::new(),
        }
    }

    /// Agregar habilidad
    pub fn add_skill(&mut self, skill: Skill) {
        let skill_idx = self.skills.len();
        
        // Agregar al índice por tipo
        self.type_index.push((skill.skill_type, skill_idx));
        
        // Agregar al índice por nombre
        self.name_index.push((skill.name.clone(), skill_idx));
        
        self.skills.push(skill);
    }

    /// Obtener habilidad por ID
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    /// Obtener habilidad mutable por ID
    pub fn get_skill_mut(&mut self, id: &str) -> Option<&mut Skill> {
        self.skills.iter_mut().find(|s| s.id == id)
    }

    /// Obtener habilidades por tipo
    pub fn get_skills_by_type(&self, skill_type: SkillType) -> Vec<&Skill> {
        let mut indices = Vec::new();
        
        for (t, idx) in &self.type_index {
            if *t == skill_type {
                indices.push(*idx);
            }
        }
        
        indices.iter()
            .filter_map(|&idx| self.skills.get(idx))
            .collect()
    }

    /// Buscar habilidad por nombre
    pub fn search_by_name(&self, query: &str) -> Vec<&Skill> {
        let query_lower = query.to_lowercase();
        
        self.skills.iter()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Obtener habilidades maestras
    pub fn get_mastered_skills(&self) -> Vec<&Skill> {
        self.skills.iter()
            .filter(|s| s.state == SkillState::Mastered)
            .collect()
    }

    /// Obtener habilidades en aprendizaje
    pub fn get_learning_skills(&self) -> Vec<&Skill> {
        self.skills.iter()
            .filter(|s| s.state == SkillState::Learning || s.state == SkillState::Improving)
            .collect()
    }

    /// Consolidar habilidades similares
    pub fn consolidate_similar_skills(&mut self) {
        // En un sistema real, esto consolidaría habilidades similares
        let _ = self;
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Skill Library Status\n");
        report.push_str("===================\n\n");
        
        report.push_str(&format!("Total Skills: {}\n", self.skills.len()));
        report.push_str(&format!("Mastered: {}\n", self.get_mastered_skills().len()));
        report.push_str(&format!("Learning: {}\n", self.get_learning_skills().len()));
        report.push_str(&format!("Unlearned: {}\n\n", 
            self.skills.iter().filter(|s| s.state == SkillState::Unlearned).count()));
        
        for skill in &self.skills {
            report.push_str(&format!("Skill: {}\n", skill.name));
            report.push_str(&format!("  Type: {:?}\n", skill.skill_type));
            report.push_str(&format!("  State: {:?}\n", skill.state));
            report.push_str(&format!("  Mastery: {}\n", skill.mastery_level));
            report.push_str(&format!("  Success Rate: {}\n", skill.success_rate));
            report.push_str(&format!("  Usage Count: {}\n", skill.usage_count));
            report.push('\n');
        }
        
        report
    }
}

impl Default for SkillLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Sistema de habilidades
#[derive(Debug, Clone)]
pub struct SkillsSystem {
    /// Biblioteca de habilidades
    pub library: SkillLibrary,
    /// Habilidades activas
    pub active_skills: Vec<String>,
    /// Historial de uso
    pub usage_history: Vec<(String, u64, bool)>,
}

impl SkillsSystem {
    /// Crear nuevo sistema de habilidades
    pub fn new() -> Self {
        Self {
            library: SkillLibrary::new(),
            active_skills: Vec::new(),
            usage_history: Vec::new(),
        }
    }

    /// Registrar habilidad
    pub fn register_skill(&mut self, skill: Skill) {
        self.library.add_skill(skill);
    }

    /// Activar habilidad
    pub fn activate_skill(&mut self, skill_id: String) -> Result<(), String> {
        if self.library.get_skill(&skill_id).is_none() {
            return Err(String::from("Skill not found"));
        }
        
        if !self.active_skills.contains(&skill_id) {
            self.active_skills.push(skill_id);
        }
        
        Ok(())
    }

    /// Desactivar habilidad
    pub fn deactivate_skill(&mut self, skill_id: String) -> Result<(), String> {
        if let Some(pos) = self.active_skills.iter().position(|id| id == &skill_id) {
            self.active_skills.remove(pos);
            Ok(())
        } else {
            Err(String::from("Skill not active"))
        }
    }

    /// Usar habilidad
    pub fn use_skill(&mut self, skill_id: String) -> Result<String, String> {
        let skill = self.library.get_skill_mut(&skill_id)
            .ok_or_else(|| String::from("Skill not found"))?;
        
        let result = skill.use_skill()?;
        
        // Registrar en historial
        self.usage_history.push((skill_id.clone(), 0, true));
        
        Ok(result)
    }

    /// Usar habilidad con resultado
    pub fn use_skill_with_result(&mut self, skill_id: String, success: bool) -> Result<String, String> {
        let skill = self.library.get_skill_mut(&skill_id)
            .ok_or_else(|| String::from("Skill not found"))?;
        
        let result = skill.use_skill()?;
        skill.update_success_rate(success);
        
        // Registrar en historial
        self.usage_history.push((skill_id.clone(), 0, success));
        
        Ok(result)
    }

    /// Mejorar habilidad
    pub fn improve_skill(&mut self, skill_id: String, improvement: u8) -> Result<(), String> {
        let skill = self.library.get_skill_mut(&skill_id)
            .ok_or_else(|| String::from("Skill not found"))?;
        
        skill.improve_mastery(improvement);
        Ok(())
    }

    /// Obtener habilidades activas
    pub fn get_active_skills(&self) -> Vec<&Skill> {
        self.active_skills.iter()
            .filter_map(|id| self.library.get_skill(id))
            .collect()
    }

    /// Obtener historial de uso
    pub fn get_usage_history(&self) -> &[(String, u64, bool)] {
        &self.usage_history
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Skills System Status\n");
        report.push_str("====================\n\n");
        
        report.push_str(&format!("Active Skills: {}\n", self.active_skills.len()));
        report.push_str(&format!("Usage History Entries: {}\n\n", self.usage_history.len()));
        
        report.push_str(&self.library.generate_status_report());
        
        report
    }
}

impl Default for SkillsSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de habilidades
pub struct SkillsUtils;

impl SkillsUtils {
    /// Crear sistema de habilidades por defecto
    pub fn create_default_skills_system() -> SkillsSystem {
        let mut system = SkillsSystem::new();
        
        // Agregar habilidades por defecto
        let mut analysis_skill = Skill::new(
            String::from("analysis_basic"),
            String::from("Basic Analysis"),
            SkillType::Analysis,
            String::from("Analyze data and patterns"),
        );
        analysis_skill.set_code(String::from("function analyze(data) { return analyze_patterns(data); }"));
        system.register_skill(analysis_skill);
        
        let mut planning_skill = Skill::new(
            String::from("planning_basic"),
            String::from("Basic Planning"),
            SkillType::Planning,
            String::from("Plan tasks and sequences"),
        );
        planning_skill.set_code(String::from("function plan(tasks) { return create_sequence(tasks); }"));
        system.register_skill(planning_skill);
        
        let mut execution_skill = Skill::new(
            String::from("execution_basic"),
            String::from("Basic Execution"),
            SkillType::Execution,
            String::from("Execute tasks and commands"),
        );
        execution_skill.set_code(String::from("function execute(task) { return run_task(task); }"));
        system.register_skill(execution_skill);
        
        let mut communication_skill = Skill::new(
            String::from("communication_basic"),
            String::from("Basic Communication"),
            SkillType::Communication,
            String::from("Communicate with other agents"),
        );
        communication_skill.set_code(String::from("function communicate(message, target) { return send_message(message, target); }"));
        system.register_skill(communication_skill);
        
        let mut learning_skill = Skill::new(
            String::from("learning_basic"),
            String::from("Basic Learning"),
            SkillType::Learning,
            String::from("Learn from experience"),
        );
        learning_skill.set_code(String::from("function learn(experience) { return update_knowledge(experience); }"));
        system.register_skill(learning_skill);
        
        system
    }

    /// Crear habilidad personalizada
    pub fn create_custom_skill(id: String, name: String, description: String, code: String) -> Skill {
        let mut skill = Skill::new(id, name, SkillType::Custom, description);
        skill.set_code(code);
        skill
    }
}
