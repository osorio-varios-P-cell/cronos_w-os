//! Agent Protocols Module
//! 
//! This module implements communication protocols between AI agents.
//! Based on Microsoft AI Agents for Beginners course - Lesson 11: Agentic Protocols.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Tipo de protocolo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// Protocolo de solicitud-respuesta
    RequestResponse,
    /// Protocolo de publicación-suscripción
    PubSub,
    /// Protocolo de flujo de trabajo
    Workflow,
    /// Protocolo de negociación
    Negotiation,
    /// Protocolo personalizado
    Custom,
}

/// Tipo de mensaje
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Solicitud
    Request,
    /// Respuesta
    Response,
    /// Notificación
    Notification,
    /// Comando
    Command,
    /// Estado
    Status,
    /// Error
    Error,
}

/// Mensaje de protocolo
#[derive(Debug, Clone)]
pub struct ProtocolMessage {
    /// ID del mensaje
    pub message_id: String,
    /// Tipo de mensaje
    pub message_type: MessageType,
    /// ID del agente emisor
    pub from_agent: String,
    /// ID del agente receptor
    pub to_agent: String,
    /// Contenido del mensaje
    pub content: String,
    /// Metadatos
    pub metadata: Vec<(String, String)>,
    /// Timestamp
    pub timestamp: u64,
    /// Prioridad (0-100)
    pub priority: u8,
}

impl ProtocolMessage {
    /// Crear nuevo mensaje
    pub fn new(message_id: String, message_type: MessageType, from_agent: String, to_agent: String, content: String) -> Self {
        Self {
            message_id,
            message_type,
            from_agent,
            to_agent,
            content,
            metadata: Vec::new(),
            timestamp: 0,
            priority: 50,
        }
    }

    /// Agregar metadato
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.push((key, value));
    }

    /// Establecer prioridad
    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority.min(100);
    }
}

/// Definición de protocolo
#[derive(Debug, Clone)]
pub struct ProtocolDefinition {
    /// ID del protocolo
    pub protocol_id: String,
    /// Nombre del protocolo
    pub name: String,
    /// Tipo de protocolo
    pub protocol_type: ProtocolType,
    /// Descripción
    pub description: String,
    /// Esquema de mensajes
    pub message_schema: Vec<String>,
    /// Reglas de validación
    pub validation_rules: Vec<String>,
}

impl ProtocolDefinition {
    /// Crear nueva definición de protocolo
    pub fn new(protocol_id: String, name: String, protocol_type: ProtocolType, description: String) -> Self {
        Self {
            protocol_id,
            name,
            protocol_type,
            description,
            message_schema: Vec::new(),
            validation_rules: Vec::new(),
        }
    }

    /// Agregar esquema de mensaje
    pub fn add_message_schema(&mut self, schema: String) {
        self.message_schema.push(schema);
    }

    /// Agregar regla de validación
    pub fn add_validation_rule(&mut self, rule: String) {
        self.validation_rules.push(rule);
    }
}

/// Sistema de protocolos de agentes
pub struct AgentProtocolSystem {
    /// Protocolos definidos
    pub protocols: Vec<ProtocolDefinition>,
    /// Mensajes enviados
    pub sent_messages: Vec<ProtocolMessage>,
    /// Mensajes recibidos
    pub received_messages: Vec<ProtocolMessage>,
    /// Colas de mensajes por agente
    pub message_queues: Vec<(String, Vec<ProtocolMessage>)>,
}

impl AgentProtocolSystem {
    /// Crear nuevo sistema de protocolos
    pub fn new() -> Self {
        Self {
            protocols: Vec::new(),
            sent_messages: Vec::new(),
            received_messages: Vec::new(),
            message_queues: Vec::new(),
        }
    }

    /// Registrar protocolo
    pub fn register_protocol(&mut self, protocol: ProtocolDefinition) {
        self.protocols.push(protocol);
    }

    /// Obtener protocolo por ID
    pub fn get_protocol(&self, protocol_id: &str) -> Option<&ProtocolDefinition> {
        self.protocols.iter().find(|p| p.protocol_id == protocol_id)
    }

    /// Enviar mensaje
    pub fn send_message(&mut self, message: ProtocolMessage) -> Result<(), String> {
        // Validar mensaje
        if !self.validate_message(&message) {
            return Err(String::from("Message validation failed"));
        }
        
        self.sent_messages.push(message.clone());
        
        // Agregar a la cola del receptor
        self.add_to_queue(message.to_agent.clone(), message);
        
        Ok(())
    }

    /// Recibir mensaje
    pub fn receive_message(&mut self, agent_id: &str) -> Option<ProtocolMessage> {
        if let Some(queue) = self.message_queues.iter_mut().find(|(id, _)| id == agent_id) {
            if let Some(message) = queue.1.pop() {
                self.received_messages.push(message.clone());
                return Some(message);
            }
        }
        None
    }

    /// Agregar mensaje a la cola
    fn add_to_queue(&mut self, agent_id: String, message: ProtocolMessage) {
        if let Some(queue) = self.message_queues.iter_mut().find(|(id, _)| id == &agent_id) {
            queue.1.push(message);
        } else {
            self.message_queues.push((agent_id, vec![message]));
        }
    }

    /// Validar mensaje
    fn validate_message(&self, message: &ProtocolMessage) -> bool {
        // En un sistema real, esto validaría el mensaje contra los protocolos
        true
    }

    /// Obtener mensajes pendientes para un agente
    pub fn get_pending_messages(&self, agent_id: &str) -> Vec<&ProtocolMessage> {
        if let Some(queue) = self.message_queues.iter().find(|(id, _)| id == agent_id) {
            queue.1.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Crear cola de mensajes para un agente
    pub fn create_queue(&mut self, agent_id: String) {
        if !self.message_queues.iter().any(|(id, _)| id == &agent_id) {
            self.message_queues.push((agent_id, Vec::new()));
        }
    }

    /// Obtener protocolos por tipo
    pub fn get_protocols_by_type(&self, protocol_type: ProtocolType) -> Vec<&ProtocolDefinition> {
        self.protocols.iter()
            .filter(|p| p.protocol_type == protocol_type)
            .collect()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Agent Protocol System Status\n");
        report.push_str("===========================\n\n");
        
        report.push_str(&format!("Total Protocols: {}\n", self.protocols.len()));
        report.push_str(&format!("Sent Messages: {}\n", self.sent_messages.len()));
        report.push_str(&format!("Received Messages: {}\n", self.received_messages.len()));
        report.push_str(&format!("Message Queues: {}\n\n", self.message_queues.len()));
        
        report.push_str("Protocols:\n");
        for protocol in &self.protocols {
            report.push_str(&format!("  - {} ({:?})\n", protocol.name, protocol.protocol_type));
        }
        
        report.push('\n');
        
        report.push_str("Message Queues:\n");
        for (agent_id, queue) in &self.message_queues {
            report.push_str(&format!("  Agent {}: {} pending messages\n", agent_id, queue.len()));
        }
        
        report
    }
}

impl Default for AgentProtocolSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de protocolos de agentes
pub struct AgentProtocolUtils;

impl AgentProtocolUtils {
    /// Crear sistema de protocolos por defecto
    pub fn create_default_protocol_system() -> AgentProtocolSystem {
        let mut system = AgentProtocolSystem::new();
        
        // Agregar protocolos por defecto
        let mut req_resp = ProtocolDefinition::new(
            String::from("req_resp"),
            String::from("Request-Response"),
            ProtocolType::RequestResponse,
            String::from("Standard request-response protocol"),
        );
        req_resp.add_message_schema(String::from("request"));
        req_resp.add_message_schema(String::from("response"));
        system.register_protocol(req_resp);
        
        let mut pubsub = ProtocolDefinition::new(
            String::from("pubsub"),
            String::from("Publish-Subscribe"),
            ProtocolType::PubSub,
            String::from("Publish-subscribe protocol for notifications"),
        );
        pubsub.add_message_schema(String::from("publish"));
        pubsub.add_message_schema(String::from("subscribe"));
        system.register_protocol(pubsub);
        
        system
    }

    /// Crear mensaje por defecto
    pub fn create_default_message(message_id: String, from_agent: String, to_agent: String, content: String) -> ProtocolMessage {
        ProtocolMessage::new(message_id, MessageType::Request, from_agent, to_agent, content)
    }

    /// Crear definición de protocolo por defecto
    pub fn create_default_protocol(protocol_id: String, name: String, protocol_type: ProtocolType) -> ProtocolDefinition {
        ProtocolDefinition::new(protocol_id, name, protocol_type, String::from("Default protocol"))
    }
}
