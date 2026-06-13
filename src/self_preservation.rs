//! Self-Preservation System Module
//! 
//! This module implements a self-preservation system that monitors critical hardware
//! conditions and automatically initiates safe shutdown procedures to protect the system
//! from catastrophic failures.

extern crate alloc;

use alloc::string::String;
use alloc::format;

/// Tipo de evento crítico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalEventType {
    /// Temperatura crítica
    CriticalTemperature,
    /// Voltaje crítico
    CriticalVoltage,
    /// Falla de drive
    DriveFailure,
    /// Falla de fan
    FanFailure,
    /// Error de memoria
    MemoryError,
    /// Error de CPU
    CpuError,
    /// Error de GPU
    GpuError,
    /// Otro error crítico
    Other,
}

/// Severidad del evento
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSeverity {
    /// Advertencia (no requiere acción inmediata)
    Warning,
    /// Error (requiere atención)
    Error,
    /// Crítico (requiere acción inmediata)
    Critical,
    /// Catastrófico (requiere apagado inmediato)
    Catastrophic,
}

/// Acción de preservación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreservationAction {
    /// Ninguna acción
    None,
    /// Registrar evento
    LogOnly,
    /// Notificar al usuario
    Notify,
    /// Throttling
    Throttle,
    /// Apagado gracioso
    GracefulShutdown,
    /// Apagado de emergencia
    EmergencyShutdown,
}

/// Evento crítico
#[derive(Debug, Clone)]
pub struct CriticalEvent {
    /// Tipo de evento
    pub event_type: CriticalEventType,
    /// Severidad
    pub severity: EventSeverity,
    /// Mensaje descriptivo
    pub message: String,
    /// Acción tomada
    pub action_taken: PreservationAction,
    /// Timestamp
    pub timestamp: u64,
}

impl CriticalEvent {
    /// Crear un nuevo evento crítico
    pub fn new(event_type: CriticalEventType, severity: EventSeverity, message: String) -> Self {
        Self {
            event_type,
            severity,
            message,
            action_taken: PreservationAction::None,
            timestamp: 0,
        }
    }
}

/// Configuración del sistema de preservación
#[derive(Debug, Clone)]
pub struct PreservationConfig {
    /// Temperatura crítica máxima (Celsius)
    pub critical_temperature: i32,
    /// Umbral de voltaje bajo (porcentaje del nominal)
    pub low_voltage_threshold: f32,
    /// Umbral de voltaje alto (porcentaje del nominal)
    pub high_voltage_threshold: f32,
    /// Habilitar apagado automático
    pub auto_shutdown_enabled: bool,
    /// Tiempo de gracia antes del apagado (segundos)
    pub grace_period_seconds: u32,
    /// Habilitar notificación antes del apagado
    pub notify_before_shutdown: bool,
}

impl Default for PreservationConfig {
    fn default() -> Self {
        Self {
            critical_temperature: 100,
            low_voltage_threshold: 0.8,
            high_voltage_threshold: 1.2,
            auto_shutdown_enabled: true,
            grace_period_seconds: 30,
            notify_before_shutdown: true,
        }
    }
}

/// Estado del sistema de preservación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreservationState {
    /// Normal
    Normal,
    /// Advertencia
    Warning,
    /// Error
    Error,
    /// Crítico
    Critical,
    /// Apagado en progreso
    ShuttingDown,
    /// Apagado de emergencia
    EmergencyShutdown,
}

/// Sistema de preservación propio
pub struct SelfPreservationSystem {
    /// Configuración
    config: PreservationConfig,
    /// Estado actual
    state: PreservationState,
    /// Historial de eventos críticos
    event_history: alloc::vec::Vec<CriticalEvent>,
    /// Contador de eventos críticos
    critical_event_count: u32,
    /// Tiempo de inicio del apagado
    shutdown_start_time: Option<u64>,
}

impl SelfPreservationSystem {
    /// Crear un nuevo sistema de preservación
    pub fn new(config: PreservationConfig) -> Self {
        Self {
            config,
            state: PreservationState::Normal,
            event_history: alloc::vec::Vec::new(),
            critical_event_count: 0,
            shutdown_start_time: None,
        }
    }

    /// Verificar temperatura y tomar acción si es necesario
    pub fn check_temperature(&mut self, temperature: i32) -> PreservationAction {
        if temperature >= self.config.critical_temperature {
            self.handle_critical_event(CriticalEvent::new(
                CriticalEventType::CriticalTemperature,
                EventSeverity::Catastrophic,
                alloc::format!("Critical temperature: {}°C", temperature),
            ))
        } else if temperature >= self.config.critical_temperature - 10 {
            self.handle_critical_event(CriticalEvent::new(
                CriticalEventType::CriticalTemperature,
                EventSeverity::Critical,
                alloc::format!("High temperature: {}°C", temperature),
            ))
        } else {
            PreservationAction::None
        }
    }

    /// Verificar voltaje y tomar acción si es necesario
    pub fn check_voltage(&mut self, voltage_mv: u32, nominal_mv: u32) -> PreservationAction {
        let voltage_ratio = voltage_mv as f32 / nominal_mv as f32;
        
        if voltage_ratio < self.config.low_voltage_threshold || voltage_ratio > self.config.high_voltage_threshold {
            self.handle_critical_event(CriticalEvent::new(
                CriticalEventType::CriticalVoltage,
                EventSeverity::Catastrophic,
                alloc::format!("Critical voltage: {}mV (nominal: {}mV)", voltage_mv, nominal_mv),
            ))
        } else if voltage_ratio < self.config.low_voltage_threshold + 0.1 || voltage_ratio > self.config.high_voltage_threshold - 0.1 {
            self.handle_critical_event(CriticalEvent::new(
                CriticalEventType::CriticalVoltage,
                EventSeverity::Warning,
                alloc::format!("Voltage out of range: {}mV (nominal: {}mV)", voltage_mv, nominal_mv),
            ))
        } else {
            PreservationAction::None
        }
    }

    /// Reportar falla de drive
    pub fn report_drive_failure(&mut self, drive_name: String) -> PreservationAction {
        self.handle_critical_event(CriticalEvent::new(
            CriticalEventType::DriveFailure,
            EventSeverity::Critical,
            alloc::format!("Drive failure: {}", drive_name),
        ))
    }

    /// Reportar falla de fan
    pub fn report_fan_failure(&mut self, fan_location: String) -> PreservationAction {
        self.handle_critical_event(CriticalEvent::new(
            CriticalEventType::FanFailure,
            EventSeverity::Critical,
            alloc::format!("Fan failure: {}", fan_location),
        ))
    }

    /// Manejar un evento crítico
    fn handle_critical_event(&mut self, event: CriticalEvent) -> PreservationAction {
        let action = self.determine_action(&event);
        
        // Actualizar estado
        self.state = match action {
            PreservationAction::EmergencyShutdown => PreservationState::EmergencyShutdown,
            PreservationAction::GracefulShutdown => PreservationState::ShuttingDown,
            PreservationAction::Throttle => PreservationState::Critical,
            PreservationAction::Notify => PreservationState::Warning,
            PreservationAction::LogOnly => PreservationState::Error,
            PreservationAction::None => self.state,
        };
        
        // Contar eventos críticos
        if event.severity == EventSeverity::Critical || event.severity == EventSeverity::Catastrophic {
            self.critical_event_count += 1;
        }
        
        // Guardar evento
        self.event_history.push(event);
        
        // Mantener solo los últimos 1000 eventos
        if self.event_history.len() > 1000 {
            self.event_history.drain(0..self.event_history.len() - 1000);
        }
        
        action
    }

    /// Determinar la acción apropiada para un evento
    fn determine_action(&self, event: &CriticalEvent) -> PreservationAction {
        if !self.config.auto_shutdown_enabled {
            return PreservationAction::LogOnly;
        }
        
        match event.severity {
            EventSeverity::Catastrophic => PreservationAction::EmergencyShutdown,
            EventSeverity::Critical => PreservationAction::GracefulShutdown,
            EventSeverity::Error => PreservationAction::Throttle,
            EventSeverity::Warning => PreservationAction::Notify,
        }
    }

    /// Iniciar apagado gracioso
    pub fn initiate_graceful_shutdown(&mut self) {
        self.shutdown_start_time = Some(0); // En un sistema real, timestamp actual
        self.state = PreservationState::ShuttingDown;
        
        // En un sistema real, aquí se:
        // 1. Notificar al usuario
        // 2. Guardar datos críticos
        // 3. Cerrar procesos
        // 4. Desmontar filesystems
        // 5. Apagar el sistema
    }

    /// Iniciar apagado de emergencia
    pub fn initiate_emergency_shutdown(&mut self) {
        self.shutdown_start_time = Some(0); // En un sistema real, timestamp actual
        self.state = PreservationState::EmergencyShutdown;
        
        // En un sistema real, aquí se:
        // 1. Guardar datos críticos si es posible
        // 2. Apagar el sistema inmediatamente
    }

    /// Obtener el estado actual
    pub fn state(&self) -> PreservationState {
        self.state
    }

    /// Verificar si el sistema está en apagado
    pub fn is_shutting_down(&self) -> bool {
        self.state == PreservationState::ShuttingDown || self.state == PreservationState::EmergencyShutdown
    }

    /// Obtener el número de eventos críticos
    pub fn critical_event_count(&self) -> u32 {
        self.critical_event_count
    }

    /// Obtener eventos por tipo
    pub fn get_events_by_type(&self, event_type: CriticalEventType) -> alloc::vec::Vec<&CriticalEvent> {
        self.event_history.iter()
            .filter(|event| event.event_type == event_type)
            .collect()
    }

    /// Obtener eventos por severidad
    pub fn get_events_by_severity(&self, severity: EventSeverity) -> alloc::vec::Vec<&CriticalEvent> {
        self.event_history.iter()
            .filter(|event| event.severity == severity)
            .collect()
    }

    /// Verificar si hay eventos catastróficos recientes
    pub fn has_recent_catastrophic_events(&self) -> bool {
        self.event_history.iter()
            .any(|event| event.severity == EventSeverity::Catastrophic)
    }

    /// Obtener la configuración
    pub fn config(&self) -> &PreservationConfig {
        &self.config
    }

    /// Modificar la configuración
    pub fn set_config(&mut self, config: PreservationConfig) {
        self.config = config;
    }
}

impl Default for SelfPreservationSystem {
    fn default() -> Self {
        Self::new(PreservationConfig::default())
    }
}
