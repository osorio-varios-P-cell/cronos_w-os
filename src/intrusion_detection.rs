//! Intrusion Detection para CRONOS W-OS (AEGIS Layer)
//!
//! Este módulo implementa detección de intrusos en la capa AEGIS,
//! permitiendo detectar y prevenir ataques de seguridad

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del sistema de detección de intrusos
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrusionDetectionState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Monitoreando
    Monitoring,
    /// Analizando
    Analyzing,
    /// Detectando amenaza
    DetectingThreat,
    /// Respondiendo
    Responding,
    /// Error
    Error(String),
}

/// Tipo de amenaza
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatType {
    /// Acceso no autorizado
    UnauthorizedAccess,
    /// Elevación de privilegios
    PrivilegeEscalation,
    /// Inyección de código
    CodeInjection,
    /// Denegación de servicio
    DenialOfService,
    /// Ataque de fuerza bruta
    BruteForce,
    /// Escaneo de puertos
    PortScanning,
    /// Malware
    Malware,
    /// Rootkit
    Rootkit,
    /// Custom
    Custom,
}

/// Severidad de amenaza
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatSeverity {
    /// Baja
    Low,
    /// Media
    Medium,
    /// Alta
    High,
    /// Crítica
    Critical,
}

/// Tipo de respuesta
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    /// Log only
    LogOnly,
    /// Bloquear capability
    BlockCapability,
    /// Revocar capability
    RevokeCapability,
    /// Aislar proceso
    IsolateProcess,
    /// Terminar proceso
    TerminateProcess,
    /// Bloquear IP
    BlockIp,
    /// Alerta administrador
    AlertAdmin,
}

/// Configuración de regla de detección
#[derive(Debug, Clone)]
pub struct DetectionRuleConfig {
    /// ID único de la regla
    pub rule_id: u64,
    /// Nombre de la regla
    pub name: String,
    /// Tipo de amenaza
    pub threat_type: ThreatType,
    /// Patrón a detectar
    pub pattern: String,
    /// Severidad mínima
    pub min_severity: ThreatSeverity,
    /// Tipo de respuesta
    pub response_type: ResponseType,
    /// Habilitada
    pub enabled: bool,
}

impl DetectionRuleConfig {
    pub fn new(rule_id: u64, name: String, threat_type: ThreatType, pattern: String) -> Self {
        Self {
            rule_id,
            name,
            threat_type,
            pattern,
            min_severity: ThreatSeverity::Medium,
            response_type: ResponseType::LogOnly,
            enabled: true,
        }
    }

    pub fn with_severity(mut self, severity: ThreatSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    pub fn with_response(mut self, response: ResponseType) -> Self {
        self.response_type = response;
        self
    }
}

/// Amenaza detectada
#[derive(Debug, Clone)]
pub struct Threat {
    /// ID único de la amenaza
    pub threat_id: u64,
    /// Tipo de amenaza
    pub threat_type: ThreatType,
    /// Severidad
    pub severity: ThreatSeverity,
    /// Origen (IP, proceso, etc.)
    pub source: String,
    /// Destino
    pub target: String,
    /// Descripción
    pub description: String,
    /// Timestamp
    pub timestamp: u64,
    /// Respondida
    pub responded: bool,
}

/// Resultado de respuesta
#[derive(Debug, Clone)]
pub struct ResponseResult {
    /// ID de la amenaza respondida
    pub threat_id: u64,
    /// Tipo de respuesta
    pub response_type: ResponseType,
    /// Exitoso
    pub successful: bool,
    /// Tiempo de respuesta (ms)
    pub response_time_ms: u64,
    /// Mensaje de error (si falló)
    pub error_message: Option<String>,
}

/// Métricas de detección de intrusos
#[derive(Debug, Clone)]
pub struct IntrusionDetectionMetrics {
    /// Total de amenazas detectadas
    pub total_threats: u64,
    /// Amenazas respondidas exitosamente
    pub successful_responses: u64,
    /// Amenazas fallidas al responder
    pub failed_responses: u64,
    /// Tiempo total de monitoreo (ms)
    pub total_monitoring_time_ms: u64,
    /// Tiempo total de respuesta (ms)
    pub total_response_time_ms: u64,
}

impl Default for IntrusionDetectionMetrics {
    fn default() -> Self {
        Self {
            total_threats: 0,
            successful_responses: 0,
            failed_responses: 0,
            total_monitoring_time_ms: 0,
            total_response_time_ms: 0,
        }
    }
}

/// Motor de detección de intrusos
pub struct IntrusionDetectionEngine {
    /// Reglas de detección
    pub rules: BTreeMap<u64, DetectionRuleConfig>,
    /// Amenazas detectadas
    pub threats: Vec<Threat>,
    /// Estado actual
    pub state: IntrusionDetectionState,
    /// Capability del motor
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// Métricas
    pub metrics: IntrusionDetectionMetrics,
    /// Siguiente ID de amenaza
    pub next_threat_id: u64,
}

impl IntrusionDetectionEngine {
    pub fn new() -> Self {
        Self {
            rules: BTreeMap::new(),
            threats: Vec::new(),
            state: IntrusionDetectionState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            metrics: IntrusionDetectionMetrics::default(),
            next_threat_id: 1,
        }
    }

    /// Inicializar el motor en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != IntrusionDetectionState::Uninitialized {
            return Err(format!("Motor ya inicializado, estado actual: {:?}", self.state));
        }

        let node_id = graph_kernel.create_node(
            NodeType::File,
            String::from("intrusion_detection_engine"),
        );
        self.graph_node_id = Some(node_id);

        self.state = IntrusionDetectionState::Initialized;
        Ok(())
    }

    /// Agregar regla de detección
    pub fn add_rule(&mut self, rule: DetectionRuleConfig) -> Result<(), String> {
        self.rules.insert(rule.rule_id, rule);
        Ok(())
    }

    /// Iniciar monitoreo
    pub fn start_monitoring(&mut self) -> Result<(), String> {
        if self.state != IntrusionDetectionState::Initialized {
            return Err(format!("Motor no está en estado Initialized, estado actual: {:?}", self.state));
        }

        self.state = IntrusionDetectionState::Monitoring;
        Ok(())
    }

    /// Analizar eventos de seguridad
    pub fn analyze_events(&mut self, events: Vec<String>) -> Result<Vec<Threat>, String> {
        if self.state != IntrusionDetectionState::Monitoring {
            return Err(format!("Motor no está en estado Monitoring, estado actual: {:?}", self.state));
        }

        self.state = IntrusionDetectionState::Analyzing;

        let mut detected_threats = Vec::new();

        // En un sistema real, esto analizaría eventos contra las reglas
        // Por ahora, simulamos la detección
        for event in events {
            for rule in self.rules.values().filter(|r| r.enabled) {
                if event.contains(&rule.pattern) {
                    let threat = Threat {
                        threat_id: self.next_threat_id,
                        threat_type: rule.threat_type,
                        severity: rule.min_severity,
                        source: String::from("unknown"),
                        target: String::from("system"),
                        description: format!("Threat detected by rule: {}", rule.name),
                        timestamp: 0,
                        responded: false,
                    };
                    detected_threats.push(threat.clone());
                    self.threats.push(threat);
                    self.metrics.total_threats += 1;
                    self.next_threat_id += 1;
                }
            }
        }

        self.state = IntrusionDetectionState::Monitoring;
        Ok(detected_threats)
    }

    /// Responder a amenaza
    pub fn respond_to_threat(&mut self, threat_id: u64, response_type: ResponseType) -> Result<ResponseResult, String> {
        self.state = IntrusionDetectionState::Responding;

        // En un sistema real, esto ejecutaría la respuesta
        let result = ResponseResult {
            threat_id,
            response_type,
            successful: true,
            response_time_ms: 500,
            error_message: None,
        };

        if result.successful {
            if let Some(threat) = self.threats.iter_mut().find(|t| t.threat_id == threat_id) {
                threat.responded = true;
            }
            self.metrics.successful_responses += 1;
        } else {
            self.metrics.failed_responses += 1;
        }

        self.metrics.total_response_time_ms += result.response_time_ms;
        self.state = IntrusionDetectionState::Monitoring;
        Ok(result)
    }

    /// Actualizar métricas de monitoreo
    pub fn update_metrics(&mut self, monitoring_time_ms: u64) {
        self.metrics.total_monitoring_time_ms += monitoring_time_ms;
    }

    /// Verificar si está monitoreando
    pub fn is_monitoring(&self) -> bool {
        self.state == IntrusionDetectionState::Monitoring
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &IntrusionDetectionState {
        &self.state
    }
}

/// Integración Intrusion Detection para CRONOS W-OS (AEGIS Layer)
pub struct CronosIntrusionDetectionIntegration {
    /// Motor de detección
    pub engine: IntrusionDetectionEngine,
    /// Estado del módulo
    pub state: IntrusionDetectionState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del módulo
    pub capability_id: Option<CapabilityId>,
}

impl CronosIntrusionDetectionIntegration {
    pub fn new() -> Self {
        Self {
            engine: IntrusionDetectionEngine::new(),
            state: IntrusionDetectionState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = IntrusionDetectionState::Initialized;
    }

    /// Inicializar el motor
    pub fn initialize_engine(&mut self) -> Result<(), String> {
        if self.state == IntrusionDetectionState::Uninitialized {
            return Err(String::from("Intrusion Detection no inicializado. Llamar a set_graph_kernel primero."));
        }

        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                self.engine.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        // Agregar reglas predeterminadas
        let rule1 = DetectionRuleConfig::new(1, String::from("Unauthorized Access"), ThreatType::UnauthorizedAccess, String::from("unauthorized"));
        let rule2 = DetectionRuleConfig::new(2, String::from("Privilege Escalation"), ThreatType::PrivilegeEscalation, String::from("escalation"));
        let rule3 = DetectionRuleConfig::new(3, String::from("Code Injection"), ThreatType::CodeInjection, String::from("injection"));

        self.engine.add_rule(rule1)?;
        self.engine.add_rule(rule2)?;
        self.engine.add_rule(rule3)?;

        self.state = IntrusionDetectionState::Initialized;
        Ok(())
    }

    /// Agregar regla de detección
    pub fn add_rule(&mut self, rule: DetectionRuleConfig) -> Result<(), String> {
        self.engine.add_rule(rule)
    }

    /// Iniciar monitoreo
    pub fn start_monitoring(&mut self) -> Result<(), String> {
        self.engine.start_monitoring()
    }

    /// Analizar eventos
    pub fn analyze_events(&mut self, events: Vec<String>) -> Result<Vec<Threat>, String> {
        self.engine.analyze_events(events)
    }

    /// Responder a amenaza
    pub fn respond_to_threat(&mut self, threat_id: u64, response_type: ResponseType) -> Result<ResponseResult, String> {
        self.engine.respond_to_threat(threat_id, response_type)
    }

    /// Ejecutar ciclo completo de detección y respuesta
    pub fn execute_detection_cycle(&mut self, events: Vec<String>) -> Result<(), String> {
        let threats = self.analyze_events(events)?;
        
        for threat in threats {
            if !threat.responded {
                let response_type = if threat.severity == ThreatSeverity::Critical {
                    ResponseType::TerminateProcess
                } else if threat.severity == ThreatSeverity::High {
                    ResponseType::BlockCapability
                } else {
                    ResponseType::LogOnly
                };
                
                self.respond_to_threat(threat.threat_id, response_type)?;
            }
        }
        
        Ok(())
    }

    /// Actualizar métricas
    pub fn update_metrics(&mut self, monitoring_time_ms: u64) {
        self.engine.update_metrics(monitoring_time_ms);
    }

    /// Obtener métricas
    pub fn get_metrics(&self) -> &IntrusionDetectionMetrics {
        &self.engine.metrics
    }

    /// Obtener amenazas
    pub fn get_threats(&self) -> &[Threat] {
        &self.engine.threats
    }

    /// Verificar si está monitoreando
    pub fn is_monitoring(&self) -> bool {
        self.engine.is_monitoring()
    }

    /// Obtener el estado del módulo
    pub fn state(&self) -> &IntrusionDetectionState {
        &self.state
    }
}

impl Default for CronosIntrusionDetectionIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración Intrusion Detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrusionDetectionError {
    EngineNotInitialized,
    EngineAlreadyMonitoring,
    EngineNotMonitoring,
    InvalidRule,
    ResponseFailed,
    AnalysisFailed,
}

impl fmt::Display for IntrusionDetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntrusionDetectionError::EngineNotInitialized => write!(f, "Engine not initialized"),
            IntrusionDetectionError::EngineAlreadyMonitoring => write!(f, "Engine is already monitoring"),
            IntrusionDetectionError::EngineNotMonitoring => write!(f, "Engine is not monitoring"),
            IntrusionDetectionError::InvalidRule => write!(f, "Invalid rule"),
            IntrusionDetectionError::ResponseFailed => write!(f, "Response failed"),
            IntrusionDetectionError::AnalysisFailed => write!(f, "Analysis failed"),
        }
    }
}
