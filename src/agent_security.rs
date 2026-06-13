//! Agent Security Module
//! 
//! This module implements security measures for AI agents.
//! Based on Microsoft AI Agents for Beginners course - Lesson 18: Securing AI Agents.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de amenaza de seguridad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityThreatType {
    /// Inyección de prompt
    PromptInjection,
    /// Exfiltración de datos
    DataExfiltration,
    /// Ataque de negación de servicio
    DenialOfService,
    /// Suplantación de identidad
    IdentitySpoofing,
    /// Acceso no autorizado
    UnauthorizedAccess,
    /// Amenaza personalizada
    Custom,
}

/// Nivel de seguridad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Bajo
    Low,
    /// Medio
    Medium,
    /// Alto
    High,
    /// Crítico
    Critical,
}

/// Política de seguridad
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// ID de la política
    pub policy_id: String,
    /// Nombre de la política
    pub name: String,
    /// Nivel de seguridad
    pub security_level: SecurityLevel,
    /// Reglas de la política
    pub rules: Vec<String>,
    /// Acciones permitidas
    pub allowed_actions: Vec<String>,
    /// Acciones prohibidas
    pub forbidden_actions: Vec<String>,
}

impl SecurityPolicy {
    /// Crear nueva política de seguridad
    pub fn new(policy_id: String, name: String, security_level: SecurityLevel) -> Self {
        Self {
            policy_id,
            name,
            security_level,
            rules: Vec::new(),
            allowed_actions: Vec::new(),
            forbidden_actions: Vec::new(),
        }
    }

    /// Agregar regla
    pub fn add_rule(&mut self, rule: String) {
        self.rules.push(rule);
    }

    /// Agregar acción permitida
    pub fn add_allowed_action(&mut self, action: String) {
        self.allowed_actions.push(action);
    }

    /// Agregar acción prohibida
    pub fn add_forbidden_action(&mut self, action: String) {
        self.forbidden_actions.push(action);
    }

    /// Verificar si acción está permitida
    pub fn is_action_allowed(&self, action: &str) -> bool {
        if self.forbidden_actions.iter().any(|a| a == action) {
            return false;
        }
        
        if self.allowed_actions.is_empty() {
            return true;
        }
        
        self.allowed_actions.iter().any(|a| a == action)
    }
}

/// Evento de seguridad
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// ID del evento
    pub event_id: String,
    /// Tipo de amenaza
    pub threat_type: SecurityThreatType,
    /// Descripción
    pub description: String,
    /// Severidad (0-100)
    pub severity: u8,
    /// ID del agente afectado
    pub affected_agent: String,
    /// Timestamp
    pub timestamp: u64,
    /// Resuelto
    pub resolved: bool,
}

impl SecurityEvent {
    /// Crear nuevo evento de seguridad
    pub fn new(event_id: String, threat_type: SecurityThreatType, description: String, severity: u8, affected_agent: String) -> Self {
        Self {
            event_id,
            threat_type,
            description,
            severity,
            affected_agent,
            timestamp: 0,
            resolved: false,
        }
    }

    /// Marcar como resuelto
    pub fn mark_resolved(&mut self) {
        self.resolved = true;
    }
}

/// Sistema de seguridad de agentes
pub struct AgentSecuritySystem {
    /// Políticas de seguridad
    pub policies: Vec<SecurityPolicy>,
    /// Eventos de seguridad
    pub security_events: Vec<SecurityEvent>,
    /// Nivel de seguridad actual
    pub current_security_level: SecurityLevel,
    /// Agentes bloqueados
    pub blocked_agents: Vec<String>,
}

impl AgentSecuritySystem {
    /// Crear nuevo sistema de seguridad
    pub fn new(security_level: SecurityLevel) -> Self {
        Self {
            policies: Vec::new(),
            security_events: Vec::new(),
            current_security_level: security_level,
            blocked_agents: Vec::new(),
        }
    }

    /// Agregar política de seguridad
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Obtener política por ID
    pub fn get_policy(&self, policy_id: &str) -> Option<&SecurityPolicy> {
        self.policies.iter().find(|p| p.policy_id == policy_id)
    }

    /// Verificar acción contra políticas
    pub fn verify_action(&self, action: &str) -> Result<(), String> {
        for policy in &self.policies {
            if !policy.is_action_allowed(action) {
                return Err(format!("Action '{}' not allowed by policy '{}'", action, policy.name));
            }
        }
        Ok(())
    }

    /// Registrar evento de seguridad
    pub fn register_security_event(&mut self, event: SecurityEvent) {
        // Si la severidad es alta, aumentar nivel de seguridad
        if event.severity >= 80 {
            self.current_security_level = SecurityLevel::Critical;
        } else if event.severity >= 60 {
            self.current_security_level = SecurityLevel::High;
        }
        
        self.security_events.push(event);
    }

    /// Bloquear agente
    pub fn block_agent(&mut self, agent_id: String) {
        if !self.blocked_agents.contains(&agent_id) {
            self.blocked_agents.push(agent_id);
        }
    }

    /// Desbloquear agente
    pub fn unblock_agent(&mut self, agent_id: String) -> Result<(), String> {
        if let Some(pos) = self.blocked_agents.iter().position(|id| id == &agent_id) {
            self.blocked_agents.remove(pos);
            Ok(())
        } else {
            Err(String::from("Agent not blocked"))
        }
    }

    /// Verificar si agente está bloqueado
    pub fn is_agent_blocked(&self, agent_id: &str) -> bool {
        self.blocked_agents.iter().any(|id| id == agent_id)
    }

    /// Obtener eventos no resueltos
    pub fn get_unresolved_events(&self) -> Vec<&SecurityEvent> {
        self.security_events.iter()
            .filter(|e| !e.resolved)
            .collect()
    }

    /// Obtener eventos por tipo de amenaza
    pub fn get_events_by_threat_type(&self, threat_type: SecurityThreatType) -> Vec<&SecurityEvent> {
        self.security_events.iter()
            .filter(|e| e.threat_type == threat_type)
            .collect()
    }

    /// Establecer nivel de seguridad
    pub fn set_security_level(&mut self, level: SecurityLevel) {
        self.current_security_level = level;
    }

    /// Generar reporte de seguridad
    pub fn generate_security_report(&self) -> String {
        let mut report = String::from("Agent Security Report\n");
        report.push_str("=====================\n\n");
        
        report.push_str(&format!("Current Security Level: {:?}\n", self.current_security_level));
        report.push_str(&format!("Total Policies: {}\n", self.policies.len()));
        report.push_str(&format!("Security Events: {}\n", self.security_events.len()));
        report.push_str(&format!("Unresolved Events: {}\n", self.get_unresolved_events().len()));
        report.push_str(&format!("Blocked Agents: {}\n\n", self.blocked_agents.len()));
        
        report.push_str("Recent Security Events:\n");
        for event in self.security_events.iter().rev().take(5) {
            report.push_str(&format!("  - {:?}: {} (Severity: {}, Resolved: {})\n", 
                event.threat_type, event.description, event.severity, event.resolved));
        }
        
        report
    }
}

impl Default for AgentSecuritySystem {
    fn default() -> Self {
        Self::new(SecurityLevel::Medium)
    }
}

/// Utilidades de seguridad de agentes
pub struct AgentSecurityUtils;

impl AgentSecurityUtils {
    /// Crear sistema de seguridad por defecto
    pub fn create_default_security_system() -> AgentSecuritySystem {
        let mut system = AgentSecuritySystem::new(SecurityLevel::Medium);
        
        // Agregar políticas por defecto
        let mut policy = SecurityPolicy::new(
            String::from("default_policy"),
            String::from("Default Security Policy"),
            SecurityLevel::Medium,
        );
        policy.add_rule(String::from("All actions must be authenticated"));
        policy.add_rule(String::from("Data exfiltration is prohibited"));
        policy.add_allowed_action(String::from("read"));
        policy.add_allowed_action(String::from("write"));
        policy.add_forbidden_action(String::from("exfiltrate"));
        policy.add_forbidden_action(String::from("inject"));
        system.add_policy(policy);
        
        system
    }

    /// Crear política de seguridad por defecto
    pub fn create_default_security_policy(policy_id: String, name: String, security_level: SecurityLevel) -> SecurityPolicy {
        let mut policy = SecurityPolicy::new(policy_id, name, security_level);
        policy.add_rule(String::from("Default security rule"));
        policy
    }

    /// Crear evento de seguridad por defecto
    pub fn create_default_security_event(event_id: String, threat_type: SecurityThreatType, description: String, severity: u8, affected_agent: String) -> SecurityEvent {
        SecurityEvent::new(event_id, threat_type, description, severity, affected_agent)
    }
}
