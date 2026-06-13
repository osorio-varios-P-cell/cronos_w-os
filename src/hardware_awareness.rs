//! Hardware Awareness Module
//! 
//! This module implements hardware awareness that maintains a comprehensive view
//! of the hardware state and provides notifications for significant changes.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de cambio de hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareChangeType {
    /// Cambio de temperatura
    TemperatureChange,
    /// Cambio de voltaje
    VoltageChange,
    /// Cambio de estado de fan
    FanStateChange,
    /// Cambio de salud de drive
    DriveHealthChange,
    /// Cambio de throttling
    ThrottlingChange,
    /// Evento crítico
    CriticalEvent,
    /// Cambio de estado general
    OverallHealthChange,
}

/// Evento de cambio de hardware
#[derive(Debug, Clone)]
pub struct HardwareChangeEvent {
    /// Tipo de cambio
    pub change_type: HardwareChangeType,
    /// Descripción del cambio
    pub description: String,
    /// Valor anterior
    pub previous_value: String,
    /// Valor nuevo
    pub new_value: String,
    /// Severidad del cambio
    pub severity: ChangeSeverity,
    /// Timestamp
    pub timestamp: u64,
}

/// Severidad del cambio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeSeverity {
    /// Informativo
    Info,
    /// Menor
    Minor,
    /// Mayor
    Major,
    /// Crítico
    Critical,
}

impl HardwareChangeEvent {
    /// Crear un nuevo evento de cambio
    pub fn new(
        change_type: HardwareChangeType,
        description: String,
        previous_value: String,
        new_value: String,
        severity: ChangeSeverity,
    ) -> Self {
        Self {
            change_type,
            description,
            previous_value,
            new_value,
            severity,
            timestamp: 0,
        }
    }

    /// Verificar si el cambio es crítico
    pub fn is_critical(&self) -> bool {
        self.severity == ChangeSeverity::Critical
    }
}

/// Estado del hardware
#[derive(Debug, Clone)]
pub struct HardwareState {
    /// Temperatura CPU
    pub cpu_temperature: i32,
    /// Temperatura GPU
    pub gpu_temperature: i32,
    /// Temperatura motherboard
    pub motherboard_temperature: i32,
    /// Voltaje CPU
    pub cpu_voltage: u32,
    /// Voltaje 12V
    pub voltage_12v: u32,
    /// Voltaje 5V
    pub voltage_5v: u32,
    /// Voltaje 3.3V
    pub voltage_3v3: u32,
    /// Velocidad fan CPU
    pub cpu_fan_speed: u32,
    /// Velocidad fan sistema
    pub system_fan_speed: u32,
    /// Estado de throttling
    pub throttling_active: bool,
    /// Factor de throttling
    pub throttle_factor: f32,
    /// Salud de drives
    pub drive_health: u8,
    /// Estado general de salud
    pub overall_health: u8,
    /// Timestamp
    pub timestamp: u64,
}

impl HardwareState {
    /// Crear un nuevo estado de hardware
    pub fn new() -> Self {
        Self {
            cpu_temperature: 0,
            gpu_temperature: 0,
            motherboard_temperature: 0,
            cpu_voltage: 0,
            voltage_12v: 0,
            voltage_5v: 0,
            voltage_3v3: 0,
            cpu_fan_speed: 0,
            system_fan_speed: 0,
            throttling_active: false,
            throttle_factor: 1.0,
            drive_health: 100,
            overall_health: 100,
            timestamp: 0,
        }
    }

    /// Comparar con otro estado y detectar cambios
    pub fn compare(&self, other: &HardwareState) -> Vec<HardwareChangeEvent> {
        let mut changes = Vec::new();

        // Comparar temperatura CPU
        if (self.cpu_temperature - other.cpu_temperature).abs() > 5 {
            changes.push(HardwareChangeEvent::new(
                HardwareChangeType::TemperatureChange,
                String::from("CPU temperature changed"),
                format!("{}", self.cpu_temperature),
                format!("{}", other.cpu_temperature),
                if (self.cpu_temperature - other.cpu_temperature).abs() > 10 {
                    ChangeSeverity::Major
                } else {
                    ChangeSeverity::Minor
                },
            ));
        }

        // Comparar temperatura GPU
        if (self.gpu_temperature - other.gpu_temperature).abs() > 5 {
            changes.push(HardwareChangeEvent::new(
                HardwareChangeType::TemperatureChange,
                String::from("GPU temperature changed"),
                format!("{}", self.gpu_temperature),
                format!("{}", other.gpu_temperature),
                if (self.gpu_temperature - other.gpu_temperature).abs() > 10 {
                    ChangeSeverity::Major
                } else {
                    ChangeSeverity::Minor
                },
            ));
        }

        // Comparar voltaje CPU
        if (self.cpu_voltage as i32 - other.cpu_voltage as i32).abs() > 100 {
            changes.push(HardwareChangeEvent::new(
                HardwareChangeType::VoltageChange,
                String::from("CPU voltage changed"),
                format!("{}", self.cpu_voltage),
                format!("{}", other.cpu_voltage),
                if (self.cpu_voltage as i32 - other.cpu_voltage as i32).abs() > 200 {
                    ChangeSeverity::Critical
                } else {
                    ChangeSeverity::Major
                },
            ));
        }

        // Comparar throttling
        if self.throttling_active != other.throttling_active {
            changes.push(HardwareChangeEvent::new(
                HardwareChangeType::ThrottlingChange,
                String::from("Throttling state changed"),
                format!("{}", self.throttling_active),
                format!("{}", other.throttling_active),
                if other.throttling_active {
                    ChangeSeverity::Major
                } else {
                    ChangeSeverity::Minor
                },
            ));
        }

        // Comparar salud de drives
        if (self.drive_health as i32 - other.drive_health as i32).abs() > 5 {
            changes.push(HardwareChangeEvent::new(
                HardwareChangeType::DriveHealthChange,
                String::from("Drive health changed"),
                format!("{}", self.drive_health),
                format!("{}", other.drive_health),
                if other.drive_health < 50 {
                    ChangeSeverity::Critical
                } else {
                    ChangeSeverity::Major
                },
            ));
        }

        // Comparar salud general
        if (self.overall_health as i32 - other.overall_health as i32).abs() > 10 {
            changes.push(HardwareChangeEvent::new(
                HardwareChangeType::OverallHealthChange,
                String::from("Overall health changed"),
                format!("{}", self.overall_health),
                format!("{}", other.overall_health),
                if other.overall_health < 50 {
                    ChangeSeverity::Critical
                } else {
                    ChangeSeverity::Major
                },
            ));
        }

        changes
    }
}

impl Default for HardwareState {
    fn default() -> Self {
        Self::new()
    }
}

/// Sistema de conciencia del hardware
pub struct HardwareAwarenessSystem {
    /// Estado actual del hardware
    current_state: HardwareState,
    /// Estado anterior del hardware
    previous_state: HardwareState,
    /// Historial de cambios
    change_history: Vec<HardwareChangeEvent>,
    /// Habilitar detección de cambios
    change_detection_enabled: bool,
}

impl HardwareAwarenessSystem {
    /// Crear un nuevo sistema de conciencia del hardware
    pub fn new() -> Self {
        Self {
            current_state: HardwareState::new(),
            previous_state: HardwareState::new(),
            change_history: Vec::new(),
            change_detection_enabled: true,
        }
    }

    /// Actualizar el estado del hardware
    pub fn update_state(&mut self, new_state: HardwareState) -> Vec<HardwareChangeEvent> {
        if self.change_detection_enabled {
            let changes = self.current_state.compare(&new_state);
            
            // Guardar cambios en el historial
            for change in &changes {
                self.change_history.push(change.clone());
            }
            
            // Mantener solo los últimos 1000 cambios
            if self.change_history.len() > 1000 {
                self.change_history.drain(0..self.change_history.len() - 1000);
            }
            
            // Actualizar estados
            self.previous_state = self.current_state.clone();
            self.current_state = new_state;
            
            changes
        } else {
            self.current_state = new_state;
            Vec::new()
        }
    }

    /// Obtener el estado actual
    pub fn current_state(&self) -> &HardwareState {
        &self.current_state
    }

    /// Obtener el estado anterior
    pub fn previous_state(&self) -> &HardwareState {
        &self.previous_state
    }

    /// Obtener el historial de cambios
    pub fn change_history(&self) -> &Vec<HardwareChangeEvent> {
        &self.change_history
    }

    /// Obtener cambios por tipo
    pub fn get_changes_by_type(&self, change_type: HardwareChangeType) -> Vec<&HardwareChangeEvent> {
        self.change_history.iter()
            .filter(|change| change.change_type == change_type)
            .collect()
    }

    /// Obtener cambios críticos
    pub fn get_critical_changes(&self) -> Vec<&HardwareChangeEvent> {
        self.change_history.iter()
            .filter(|change| change.is_critical())
            .collect()
    }

    /// Verificar si hay cambios críticos recientes
    pub fn has_recent_critical_changes(&self) -> bool {
        self.change_history.iter()
            .any(|change| change.is_critical())
    }

    /// Obtener el número de cambios por tipo
    pub fn get_change_count_by_type(&self, change_type: HardwareChangeType) -> usize {
        self.change_history.iter()
            .filter(|change| change.change_type == change_type)
            .count()
    }

    /// Habilitar/deshabilitar detección de cambios
    pub fn set_change_detection(&mut self, enabled: bool) {
        self.change_detection_enabled = enabled;
    }

    /// Obtener un resumen del estado actual
    pub fn get_state_summary(&self) -> String {
        alloc::format!(
            "CPU: {}°C, GPU: {}°C, Throttling: {}, Health: {}%",
            self.current_state.cpu_temperature,
            self.current_state.gpu_temperature,
            if self.current_state.throttling_active { "Active" } else { "Inactive" },
            self.current_state.overall_health
        )
    }

    /// Verificar si el hardware está en estado crítico
    pub fn is_critical_state(&self) -> bool {
        self.current_state.cpu_temperature > 90
            || self.current_state.gpu_temperature > 90
            || self.current_state.throttling_active
            || self.current_state.overall_health < 50
    }
}

impl Default for HardwareAwarenessSystem {
    fn default() -> Self {
        Self::new()
    }
}
