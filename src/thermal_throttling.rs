//! Thermal Throttling Module
//! 
//! This module implements thermal throttling and adaptive performance management
//! to protect hardware from overheating while maintaining optimal performance.

/// Perfil de rendimiento
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceProfile {
    /// Máximo rendimiento (sin throttling)
    Performance,
    /// Balanceado (throttling moderado)
    Balanced,
    /// Ahorro de energía (throttling agresivo)
    PowerSaving,
}

/// Estado de throttling térmico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottlingState {
    /// Sin throttling
    None,
    /// Throttling leve (10-25%)
    Light,
    /// Throttling moderado (25-50%)
    Moderate,
    /// Throttling severo (50-75%)
    Severe,
    /// Throttling crítico (75-100%)
    Critical,
}

/// Umbral de temperatura
#[derive(Debug, Clone, Copy)]
pub struct TemperatureThreshold {
    /// Temperatura para iniciar throttling leve
    pub light_threshold: i32,
    /// Temperatura para iniciar throttling moderado
    pub moderate_threshold: i32,
    /// Temperatura para iniciar throttling severo
    pub severe_threshold: i32,
    /// Temperatura crítica (apagado)
    pub critical_threshold: i32,
}

impl Default for TemperatureThreshold {
    fn default() -> Self {
        Self {
            light_threshold: 70,      // 70°C
            moderate_threshold: 80,  // 80°C
            severe_threshold: 90,    // 90°C
            critical_threshold: 100, // 100°C
        }
    }
}

/// Configuración de throttling
#[derive(Debug, Clone)]
pub struct ThrottlingConfig {
    /// Perfil de rendimiento actual
    pub profile: PerformanceProfile,
    /// Umbrales de temperatura
    pub thresholds: TemperatureThreshold,
    /// Habilitar throttling automático
    pub auto_throttle: bool,
    /// Factor de throttling (0.0 - 1.0)
    pub throttle_factor: f32,
}

impl Default for ThrottlingConfig {
    fn default() -> Self {
        Self {
            profile: PerformanceProfile::Balanced,
            thresholds: TemperatureThreshold::default(),
            auto_throttle: true,
            throttle_factor: 1.0,
        }
    }
}

/// Información de throttling
#[derive(Debug, Clone)]
pub struct ThrottlingInfo {
    /// Estado actual de throttling
    pub state: ThrottlingState,
    /// Temperatura actual
    pub temperature: i32,
    /// Factor de throttling actual (0.0 - 1.0)
    pub throttle_factor: f32,
    /// Frecuencia CPU actual en MHz
    pub cpu_frequency_mhz: u32,
    /// Frecuencia CPU nominal en MHz
    pub nominal_frequency_mhz: u32,
    /// Timestamp
    pub timestamp: u64,
}

impl ThrottlingInfo {
    /// Crear nueva información de throttling
    pub fn new(temperature: i32, state: ThrottlingState, throttle_factor: f32) -> Self {
        Self {
            state,
            temperature,
            throttle_factor,
            cpu_frequency_mhz: 3000, // Simulación: 3 GHz nominal
            nominal_frequency_mhz: 3000,
            timestamp: 0,
        }
    }

    /// Verificar si hay throttling activo
    pub fn is_throttling(&self) -> bool {
        self.state != ThrottlingState::None
    }

    /// Obtener el porcentaje de throttling
    pub fn throttle_percentage(&self) -> u8 {
        ((1.0 - self.throttle_factor) * 100.0) as u8
    }
}

/// Gestor de throttling térmico
pub struct ThermalThrottlingManager {
    /// Configuración de throttling
    config: ThrottlingConfig,
    /// Información histórica de throttling
    throttling_history: alloc::vec::Vec<ThrottlingInfo>,
    /// Contador de eventos de throttling
    throttling_events: u32,
}

impl ThermalThrottlingManager {
    /// Crear un nuevo gestor de throttling térmico
    pub fn new(config: ThrottlingConfig) -> Self {
        Self {
            config,
            throttling_history: alloc::vec::Vec::new(),
            throttling_events: 0,
        }
    }

    /// Establecer el perfil de rendimiento
    pub fn set_performance_profile(&mut self, profile: PerformanceProfile) {
        self.config.profile = profile;
        
        // Ajustar umbrales según el perfil
        match profile {
            PerformanceProfile::Performance => {
                // Umbrales más altos para máximo rendimiento
                self.config.thresholds.light_threshold = 80;
                self.config.thresholds.moderate_threshold = 85;
                self.config.thresholds.severe_threshold = 90;
                self.config.thresholds.critical_threshold = 100;
            }
            PerformanceProfile::Balanced => {
                // Umbrales balanceados
                self.config.thresholds.light_threshold = 70;
                self.config.thresholds.moderate_threshold = 80;
                self.config.thresholds.severe_threshold = 90;
                self.config.thresholds.critical_threshold = 100;
            }
            PerformanceProfile::PowerSaving => {
                // Umbrales más bajos para ahorro de energía
                self.config.thresholds.light_threshold = 60;
                self.config.thresholds.moderate_threshold = 70;
                self.config.thresholds.severe_threshold = 80;
                self.config.thresholds.critical_threshold = 90;
            }
        }
    }

    /// Calcular el estado de throttling basado en la temperatura
    pub fn calculate_throttling_state(&self, temperature: i32) -> ThrottlingState {
        if temperature >= self.config.thresholds.critical_threshold {
            ThrottlingState::Critical
        } else if temperature >= self.config.thresholds.severe_threshold {
            ThrottlingState::Severe
        } else if temperature >= self.config.thresholds.moderate_threshold {
            ThrottlingState::Moderate
        } else if temperature >= self.config.thresholds.light_threshold {
            ThrottlingState::Light
        } else {
            ThrottlingState::None
        }
    }

    /// Calcular el factor de throttling basado en el estado
    pub fn calculate_throttle_factor(&self, state: ThrottlingState) -> f32 {
        match state {
            ThrottlingState::None => 1.0,
            ThrottlingState::Light => 0.85,    // 15% throttling
            ThrottlingState::Moderate => 0.65, // 35% throttling
            ThrottlingState::Severe => 0.45,   // 55% throttling
            ThrottlingState::Critical => 0.25, // 75% throttling
        }
    }

    /// Procesar temperatura y aplicar throttling si es necesario
    pub fn process_temperature(&mut self, temperature: i32) -> ThrottlingInfo {
        if !self.config.auto_throttle {
            return ThrottlingInfo::new(temperature, ThrottlingState::None, 1.0);
        }

        let state = self.calculate_throttling_state(temperature);
        let throttle_factor = self.calculate_throttle_factor(state);
        
        // Actualizar configuración
        self.config.throttle_factor = throttle_factor;
        
        // Contar eventos de throttling
        if state != ThrottlingState::None {
            self.throttling_events += 1;
        }
        
        // Crear información de throttling
        let mut info = ThrottlingInfo::new(temperature, state, throttle_factor);
        
        // Ajustar frecuencia CPU según el factor de throttling
        info.cpu_frequency_mhz = (info.nominal_frequency_mhz as f32 * throttle_factor) as u32;
        
        // Guardar historial
        self.throttling_history.push(info.clone());
        
        // Mantener solo las últimas 1000 lecturas
        if self.throttling_history.len() > 1000 {
            self.throttling_history.drain(0..self.throttling_history.len() - 1000);
        }
        
        info
    }

    /// Obtener el estado actual de throttling
    pub fn current_state(&self) -> ThrottlingState {
        if let Some(last) = self.throttling_history.last() {
            last.state
        } else {
            ThrottlingState::None
        }
    }

    /// Obtener el factor de throttling actual
    pub fn current_throttle_factor(&self) -> f32 {
        self.config.throttle_factor
    }

    /// Verificar si el sistema está en throttling crítico
    pub fn is_critical(&self) -> bool {
        self.current_state() == ThrottlingState::Critical
    }

    /// Obtener el número de eventos de throttling
    pub fn throttling_event_count(&self) -> u32 {
        self.throttling_events
    }

    /// Obtener el tiempo total en throttling (simulado)
    pub fn total_throttling_time(&self) -> u64 {
        // En un sistema real, aquí se calcularía el tiempo total en throttling
        self.throttling_history.len() as u64 * 100 // 100ms por lectura
    }

    /// Obtener el porcentaje de tiempo en throttling
    pub fn throttling_percentage(&self) -> f32 {
        if self.throttling_history.is_empty() {
            return 0.0;
        }
        
        let throttling_count = self.throttling_history.iter()
            .filter(|info| info.is_throttling())
            .count();
        
        (throttling_count as f32 / self.throttling_history.len() as f32) * 100.0
    }

    /// Obtener la temperatura promedio
    pub fn average_temperature(&self) -> Option<i32> {
        if self.throttling_history.is_empty() {
            return None;
        }
        
        let sum: i32 = self.throttling_history.iter()
            .map(|info| info.temperature)
            .sum();
        
        Some(sum / self.throttling_history.len() as i32)
    }

    /// Obtener la temperatura máxima
    pub fn max_temperature(&self) -> Option<i32> {
        self.throttling_history.iter()
            .map(|info| info.temperature)
            .max()
    }

    /// Obtener la temperatura mínima
    pub fn min_temperature(&self) -> Option<i32> {
        self.throttling_history.iter()
            .map(|info| info.temperature)
            .min()
    }

    /// Habilitar/deshabilitar throttling automático
    pub fn set_auto_throttle(&mut self, enabled: bool) {
        self.config.auto_throttle = enabled;
    }

    /// Obtener la configuración actual
    pub fn config(&self) -> &ThrottlingConfig {
        &self.config
    }
}

impl Default for ThermalThrottlingManager {
    fn default() -> Self {
        Self::new(ThrottlingConfig::default())
    }
}

/// Sistema de rendimiento adaptativo
pub struct AdaptivePerformanceSystem {
    /// Gestor de throttling térmico
    thermal_manager: ThermalThrottlingManager,
    /// Historial de rendimiento
    performance_history: alloc::vec::Vec<f32>,
}

impl AdaptivePerformanceSystem {
    /// Crear un nuevo sistema de rendimiento adaptativo
    pub fn new(thermal_manager: ThermalThrottlingManager) -> Self {
        Self {
            thermal_manager,
            performance_history: alloc::vec::Vec::new(),
        }
    }

    /// Procesar temperatura y ajustar rendimiento adaptativamente
    pub fn process_temperature(&mut self, temperature: i32) -> ThrottlingInfo {
        let info = self.thermal_manager.process_temperature(temperature);
        
        // Guardar historial de rendimiento
        self.performance_history.push(info.throttle_factor);
        
        // Mantener solo las últimas 1000 lecturas
        if self.performance_history.len() > 1000 {
            self.performance_history.drain(0..self.performance_history.len() - 1000);
        }
        
        info
    }

    /// Ajustar automáticamente el perfil de rendimiento basado en el uso
    pub fn auto_adjust_profile(&mut self) {
        if self.performance_history.len() < 100 {
            return;
        }
        
        // Calcular el promedio de throttling
        let avg_throttle: f32 = self.performance_history.iter().sum::<f32>() / self.performance_history.len() as f32;
        
        // Ajustar perfil según el uso
        if avg_throttle > 0.8 {
            // Poco throttling, usar perfil de rendimiento
            self.thermal_manager.set_performance_profile(PerformanceProfile::Performance);
        } else if avg_throttle > 0.5 {
            // Throttling moderado, usar perfil balanceado
            self.thermal_manager.set_performance_profile(PerformanceProfile::Balanced);
        } else {
            // Mucho throttling, usar perfil de ahorro de energía
            self.thermal_manager.set_performance_profile(PerformanceProfile::PowerSaving);
        }
    }

    /// Obtener el gestor de throttling térmico
    pub fn thermal_manager(&self) -> &ThermalThrottlingManager {
        &self.thermal_manager
    }

    /// Obtener el gestor de throttling térmico mutable
    pub fn thermal_manager_mut(&mut self) -> &mut ThermalThrottlingManager {
        &mut self.thermal_manager
    }

    /// Obtener el rendimiento promedio
    pub fn average_performance(&self) -> Option<f32> {
        if self.performance_history.is_empty() {
            return None;
        }
        
        let sum: f32 = self.performance_history.iter().sum();
        Some(sum / self.performance_history.len() as f32)
    }
}

impl Default for AdaptivePerformanceSystem {
    fn default() -> Self {
        Self::new(ThermalThrottlingManager::default())
    }
}
