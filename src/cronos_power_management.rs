//! Power Management de CRONOS original adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el sistema de gestión de energía de CRONOS original,
//! adaptado al sistema de capabilities y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::format;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Estado de energía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerState {
    Active,
    Idle,
    Standby,
    Suspend,
    Hibernate,
    Shutdown,
}

/// Tipo de evento de energía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerEventType {
    PowerButtonPress,
    LowBattery,
    CriticalBattery,
    ThermalEvent,
    SleepTimer,
    UserRequest,
}

/// Evento de energía
#[derive(Debug, Clone)]
pub struct PowerEvent {
    pub event_id: u64,
    pub event_type: PowerEventType,
    pub timestamp: u64,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl PowerEvent {
    pub fn new(event_id: u64, event_type: PowerEventType, timestamp: u64) -> Self {
        Self {
            event_id,
            event_type,
            timestamp,
            graph_node_id: None,
        }
    }
}

/// Política de energía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerPolicy {
    Performance,
    Balanced,
    PowerSaver,
}

/// Información de batería
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub present: bool,
    pub charging: bool,
    pub level: u8, // 0-100
    pub voltage: u32, // mV
    pub current: i32, // mA
    pub capacity: u32, // mAh
    pub health: u8, // 0-100
}

impl BatteryInfo {
    pub fn new() -> Self {
        Self {
            present: false,
            charging: false,
            level: 0,
            voltage: 0,
            current: 0,
            capacity: 0,
            health: 100,
        }
    }

    pub fn is_low_battery(&self) -> bool {
        self.present && self.level < 20
    }

    pub fn is_critical_battery(&self) -> bool {
        self.present && self.level < 5
    }
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Información de temperatura
#[derive(Debug, Clone)]
pub struct ThermalInfo {
    pub cpu_temperature: i32, // Celsius
    pub gpu_temperature: i32, // Celsius
    pub fan_speed: u32, // RPM
}

impl ThermalInfo {
    pub fn new() -> Self {
        Self {
            cpu_temperature: 0,
            gpu_temperature: 0,
            fan_speed: 0,
        }
    }

    pub fn is_overheating(&self) -> bool {
        self.cpu_temperature > 80 || self.gpu_temperature > 80
    }

    pub fn is_critical_temperature(&self) -> bool {
        self.cpu_temperature > 90 || self.gpu_temperature > 90
    }
}

impl Default for ThermalInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestor de energía
pub struct CronosPowerManager {
    pub current_state: PowerState,
    pub policy: PowerPolicy,
    pub battery_info: BatteryInfo,
    pub thermal_info: ThermalInfo,
    pub event_buffer: Vec<PowerEvent>,
    pub sleep_timeout: u32, // segundos
    pub screen_timeout: u32, // segundos
    pub last_activity: u64,
    pub next_event_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosPowerManager {
    pub fn new() -> Self {
        Self {
            current_state: PowerState::Active,
            policy: PowerPolicy::Balanced,
            battery_info: BatteryInfo::new(),
            thermal_info: ThermalInfo::new(),
            event_buffer: Vec::new(),
            sleep_timeout: 300, // 5 minutos
            screen_timeout: 60, // 1 minuto
            last_activity: 0,
            next_event_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    pub fn current_state(&self) -> PowerState {
        self.current_state.clone()
    }

    pub fn set_state(&mut self, state: PowerState) {
        self.current_state = state;
    }

    pub fn policy(&self) -> PowerPolicy {
        self.policy.clone()
    }

    pub fn set_policy(&mut self, policy: PowerPolicy) {
        self.policy = policy;
    }

    pub fn battery_info(&self) -> BatteryInfo {
        self.battery_info.clone()
    }

    pub fn set_battery_info(&mut self, info: BatteryInfo) {
        self.battery_info = info;
    }

    pub fn thermal_info(&self) -> ThermalInfo {
        self.thermal_info.clone()
    }

    pub fn set_thermal_info(&mut self, info: ThermalInfo) {
        self.thermal_info = info;
    }

    pub fn sleep_timeout(&self) -> u32 {
        self.sleep_timeout
    }

    pub fn set_sleep_timeout(&mut self, timeout: u32) {
        self.sleep_timeout = timeout;
    }

    pub fn screen_timeout(&self) -> u32 {
        self.screen_timeout
    }

    pub fn set_screen_timeout(&mut self, timeout: u32) {
        self.screen_timeout = timeout;
    }

    pub fn update_activity(&mut self, timestamp: u64) {
        self.last_activity = timestamp;
    }

    pub fn push_event(&mut self, event_type: PowerEventType, timestamp: u64) {
        let event_id = self.next_event_id;
        self.next_event_id += 1;

        let mut event = PowerEvent::new(event_id, event_type, timestamp);

        // Registrar el evento como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("power_event_{}", event_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            event.graph_node_id = node_id;
        }

        self.event_buffer.push(event);
    }

    pub fn pop_event(&mut self) -> Option<PowerEvent> {
        self.event_buffer.pop()
    }

    pub fn peek_event(&self) -> Option<PowerEvent> {
        self.event_buffer.last().cloned()
    }

    pub fn has_events(&self) -> bool {
        !self.event_buffer.is_empty()
    }

    pub fn clear_buffer(&mut self) {
        self.event_buffer.clear();
    }

    pub fn check_idle_timeout(&self, current_time: u64) -> bool {
        if self.last_activity == 0 {
            return false;
        }
        
        let elapsed = current_time - self.last_activity;
        elapsed >= self.sleep_timeout as u64
    }

    pub fn check_screen_timeout(&self, current_time: u64) -> bool {
        if self.last_activity == 0 {
            return false;
        }
        
        let elapsed = current_time - self.last_activity;
        elapsed >= self.screen_timeout as u64
    }

    pub fn handle_power_event(&mut self, event: PowerEvent) -> PowerAction {
        match event.event_type {
            PowerEventType::PowerButtonPress => {
                match self.current_state {
                    PowerState::Active => PowerAction::EnterSuspend,
                    PowerState::Suspend => PowerAction::WakeUp,
                    _ => PowerAction::None,
                }
            }
            PowerEventType::LowBattery => {
                if self.policy == PowerPolicy::PowerSaver {
                    PowerAction::EnterSuspend
                } else {
                    PowerAction::NotifyLowBattery
                }
            }
            PowerEventType::CriticalBattery => {
                PowerAction::EmergencyShutdown
            }
            PowerEventType::ThermalEvent => {
                if self.thermal_info.is_critical_temperature() {
                    PowerAction::EmergencyShutdown
                } else if self.thermal_info.is_overheating() {
                    PowerAction::ThrottleCpu
                } else {
                    PowerAction::None
                }
            }
            PowerEventType::SleepTimer => {
                PowerAction::EnterSuspend
            }
            PowerEventType::UserRequest => {
                PowerAction::EnterSuspend
            }
        }
    }

    pub fn request_suspend(&mut self) -> bool {
        if self.current_state == PowerState::Active {
            self.current_state = PowerState::Suspend;
            true
        } else {
            false
        }
    }

    pub fn request_hibernate(&mut self) -> bool {
        if self.current_state == PowerState::Active || self.current_state == PowerState::Suspend {
            self.current_state = PowerState::Hibernate;
            true
        } else {
            false
        }
    }

    pub fn request_shutdown(&mut self) -> bool {
        if self.current_state == PowerState::Active {
            self.current_state = PowerState::Shutdown;
            true
        } else {
            false
        }
    }

    pub fn request_wake(&mut self) -> bool {
        if self.current_state == PowerState::Suspend || self.current_state == PowerState::Hibernate {
            self.current_state = PowerState::Active;
            true
        } else {
            false
        }
    }

    pub fn is_active(&self) -> bool {
        self.current_state == PowerState::Active
    }

    pub fn is_suspended(&self) -> bool {
        self.current_state == PowerState::Suspend
    }

    pub fn is_hibernating(&self) -> bool {
        self.current_state == PowerState::Hibernate
    }

    pub fn is_shutting_down(&self) -> bool {
        self.current_state == PowerState::Shutdown
    }
}

impl Default for CronosPowerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Acción de energía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerAction {
    None,
    EnterSuspend,
    EnterHibernate,
    WakeUp,
    Shutdown,
    EmergencyShutdown,
    ThrottleCpu,
    NotifyLowBattery,
}

/// Errores del gestor de energía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerError {
    InvalidState,
    TransitionFailed,
    BatteryNotPresent,
    ThermalProtection,
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerError::InvalidState => write!(f, "Invalid power state"),
            PowerError::TransitionFailed => write!(f, "Power state transition failed"),
            PowerError::BatteryNotPresent => write!(f, "Battery not present"),
            PowerError::ThermalProtection => write!(f, "Thermal protection triggered"),
        }
    }
}
