//! Hardware Health Monitoring Module
//! 
//! This module implements continuous hardware health monitoring that aggregates data
//! from all sensors and provides a unified view of system health status.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Estado de salud general del hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareHealthStatus {
    /// Excelente
    Excellent,
    /// Bueno
    Good,
    /// Aceptable
    Fair,
    /// Pobre
    Poor,
    /// Crítico
    Critical,
}

/// Métrica de salud del hardware
#[derive(Debug, Clone)]
pub struct HealthMetric {
    /// Nombre de la métrica
    pub name: String,
    /// Valor actual
    pub value: f32,
    /// Valor nominal
    pub nominal: f32,
    /// Unidad
    pub unit: String,
    /// Estado de la métrica
    pub status: HardwareHealthStatus,
    /// Timestamp
    pub timestamp: u64,
}

impl HealthMetric {
    /// Crear una nueva métrica de salud
    pub fn new(name: String, value: f32, nominal: f32, unit: String) -> Self {
        let status = Self::calculate_status(value, nominal);
        Self {
            name,
            value,
            nominal,
            unit,
            status,
            timestamp: 0,
        }
    }

    /// Calcular el estado basado en el valor y nominal
    fn calculate_status(value: f32, nominal: f32) -> HardwareHealthStatus {
        let ratio = value / nominal;
        
        if ratio >= 0.95 && ratio <= 1.05 {
            HardwareHealthStatus::Excellent
        } else if ratio >= 0.85 && ratio <= 1.15 {
            HardwareHealthStatus::Good
        } else if ratio >= 0.70 && ratio <= 1.30 {
            HardwareHealthStatus::Fair
        } else if ratio >= 0.50 && ratio <= 1.50 {
            HardwareHealthStatus::Poor
        } else {
            HardwareHealthStatus::Critical
        }
    }

    /// Verificar si la métrica es crítica
    pub fn is_critical(&self) -> bool {
        self.status == HardwareHealthStatus::Critical
    }
}

/// Información de salud del hardware
#[derive(Debug, Clone)]
pub struct HardwareHealthInfo {
    /// Estado general
    pub overall_status: HardwareHealthStatus,
    /// Porcentaje de salud (0-100)
    pub health_percentage: u8,
    /// Métricas de salud
    pub metrics: Vec<HealthMetric>,
    /// Alertas activas
    pub alerts: Vec<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl HardwareHealthInfo {
    /// Crear nueva información de salud
    pub fn new() -> Self {
        Self {
            overall_status: HardwareHealthStatus::Excellent,
            health_percentage: 100,
            metrics: Vec::new(),
            alerts: Vec::new(),
            timestamp: 0,
        }
    }

    /// Calcular el estado general basado en las métricas
    pub fn calculate_overall_status(&mut self) {
        if self.metrics.is_empty() {
            return;
        }

        let critical_count = self.metrics.iter().filter(|m| m.is_critical()).count();
        let poor_count = self.metrics.iter().filter(|m| m.status == HardwareHealthStatus::Poor).count();
        let fair_count = self.metrics.iter().filter(|m| m.status == HardwareHealthStatus::Fair).count();
        let good_count = self.metrics.iter().filter(|m| m.status == HardwareHealthStatus::Good).count();
        let excellent_count = self.metrics.iter().filter(|m| m.status == HardwareHealthStatus::Excellent).count();

        let total = self.metrics.len();

        if critical_count > 0 {
            self.overall_status = HardwareHealthStatus::Critical;
            self.health_percentage = 25;
        } else if poor_count > total / 3 {
            self.overall_status = HardwareHealthStatus::Poor;
            self.health_percentage = 50;
        } else if fair_count > total / 2 {
            self.overall_status = HardwareHealthStatus::Fair;
            self.health_percentage = 65;
        } else if good_count > total / 2 {
            self.overall_status = HardwareHealthStatus::Good;
            self.health_percentage = 85;
        } else {
            self.overall_status = HardwareHealthStatus::Excellent;
            self.health_percentage = 100;
        }
    }

    /// Agregar una métrica
    pub fn add_metric(&mut self, metric: HealthMetric) {
        self.metrics.push(metric);
    }

    /// Agregar una alerta
    pub fn add_alert(&mut self, alert: String) {
        self.alerts.push(alert);
    }

    /// Verificar si hay alertas críticas
    pub fn has_critical_alerts(&self) -> bool {
        self.metrics.iter().any(|m| m.is_critical()) || !self.alerts.is_empty()
    }
}

impl Default for HardwareHealthInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestor de monitoreo de salud del hardware
pub struct HardwareHealthMonitor {
    /// Historial de salud
    health_history: Vec<HardwareHealthInfo>,
    /// Intervalo de monitoreo en milisegundos
    monitoring_interval_ms: u32,
    /// Habilitar monitoreo continuo
    continuous_monitoring_enabled: bool,
    /// Último tiempo de monitoreo
    last_monitor_time: u64,
}

impl HardwareHealthMonitor {
    /// Crear un nuevo gestor de monitoreo
    pub fn new(monitoring_interval_ms: u32) -> Self {
        Self {
            health_history: Vec::new(),
            monitoring_interval_ms,
            continuous_monitoring_enabled: true,
            last_monitor_time: 0,
        }
    }

    /// Realizar un ciclo de monitoreo completo
    pub fn perform_health_check(&mut self) -> HardwareHealthInfo {
        let mut health_info = HardwareHealthInfo::new();

        // En un sistema real, aquí se:
        // 1. Leer sensores de temperatura
        // 2. Leer sensores de voltaje
        // 3. Leer datos SMART de drives
        // 4. Leer estado de fans
        // 5. Leer estado de throttling
        // 6. Agregar todas las métricas
        // 7. Calcular el estado general

        // Simulación: agregar métricas de ejemplo
        health_info.add_metric(HealthMetric::new(
            String::from("CPU Temperature"),
            45.0,
            50.0,
            String::from("°C"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("GPU Temperature"),
            55.0,
            60.0,
            String::from("°C"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("CPU Voltage"),
            1.2,
            1.2,
            String::from("V"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("12V Rail"),
            12.0,
            12.0,
            String::from("V"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("5V Rail"),
            5.0,
            5.0,
            String::from("V"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("3.3V Rail"),
            3.3,
            3.3,
            String::from("V"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("CPU Fan Speed"),
            2000.0,
            2000.0,
            String::from("RPM"),
        ));

        health_info.add_metric(HealthMetric::new(
            String::from("System Fan Speed"),
            1500.0,
            1500.0,
            String::from("RPM"),
        ));

        health_info.calculate_overall_status();

        // Guardar historial
        self.health_history.push(health_info.clone());

        // Mantener solo las últimas 1000 lecturas
        if self.health_history.len() > 1000 {
            self.health_history.drain(0..self.health_history.len() - 1000);
        }

        health_info
    }

    /// Verificar si es tiempo de monitorear
    pub fn should_monitor(&self, current_time: u64) -> bool {
        if !self.continuous_monitoring_enabled {
            return false;
        }

        current_time - self.last_monitor_time >= self.monitoring_interval_ms as u64
    }

    /// Actualizar el tiempo de monitoreo
    pub fn update_monitor_time(&mut self, current_time: u64) {
        self.last_monitor_time = current_time;
    }

    /// Obtener el estado de salud actual
    pub fn current_health_status(&self) -> HardwareHealthStatus {
        if let Some(last) = self.health_history.last() {
            last.overall_status
        } else {
            HardwareHealthStatus::Excellent
        }
    }

    /// Obtener el porcentaje de salud actual
    pub fn current_health_percentage(&self) -> u8 {
        if let Some(last) = self.health_history.last() {
            last.health_percentage
        } else {
            100
        }
    }

    /// Obtener el historial de salud
    pub fn health_history(&self) -> &Vec<HardwareHealthInfo> {
        &self.health_history
    }

    /// Obtener métricas por nombre
    pub fn get_metric_history(&self, metric_name: &str) -> Vec<&HealthMetric> {
        self.health_history.iter()
            .filter_map(|info| info.metrics.iter().find(|m| m.name == metric_name))
            .collect()
    }

    /// Calcular el promedio de una métrica
    pub fn get_metric_average(&self, metric_name: &str) -> Option<f32> {
        let metrics = self.get_metric_history(metric_name);
        if metrics.is_empty() {
            return None;
        }

        let sum: f32 = metrics.iter().map(|m| m.value).sum();
        Some(sum / metrics.len() as f32)
    }

    /// Verificar si hay degradación de salud
    pub fn has_health_degradation(&self) -> bool {
        if self.health_history.len() < 2 {
            return false;
        }

        let current = self.current_health_percentage();
        let previous = self.health_history[self.health_history.len() - 2].health_percentage;

        current < previous
    }

    /// Habilitar/deshabilitar monitoreo continuo
    pub fn set_continuous_monitoring(&mut self, enabled: bool) {
        self.continuous_monitoring_enabled = enabled;
    }

    /// Establecer el intervalo de monitoreo
    pub fn set_monitoring_interval(&mut self, interval_ms: u32) {
        self.monitoring_interval_ms = interval_ms;
    }
}

impl Default for HardwareHealthMonitor {
    fn default() -> Self {
        Self::new(1000) // 1 segundo por defecto
    }
}
