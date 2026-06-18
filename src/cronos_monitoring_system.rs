//! Monitoring System de CRONOS original adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el sistema de monitoreo y logging de CRONOS original,
//! adaptado al sistema de capabilities y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::BTreeMap;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Nivel de log
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Entrada de log
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub log_id: u64,
    pub timestamp: u64,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl LogEntry {
    pub fn new(log_id: u64, level: LogLevel, component: String, message: String) -> Self {
        Self {
            log_id,
            timestamp: 0, // Simulación de timestamp actual
            level,
            component,
            message,
            graph_node_id: None,
        }
    }
}

/// Métrica del sistema
#[derive(Debug, Clone)]
pub struct Metric {
    pub metric_id: u64,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: u64,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl Metric {
    pub fn new(metric_id: u64, name: String, value: f64, unit: String) -> Self {
        Self {
            metric_id,
            name,
            value,
            unit,
            timestamp: 0,
            graph_node_id: None,
        }
    }
}

/// Contador
#[derive(Debug, Clone)]
pub struct Counter {
    pub counter_id: u64,
    pub name: String,
    pub value: u64,
    pub description: String,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl Counter {
    pub fn new(counter_id: u64, name: String, description: String) -> Self {
        Self {
            counter_id,
            name,
            value: 0,
            description,
            graph_node_id: None,
        }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn increment_by(&mut self, amount: u64) {
        self.value += amount;
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}

/// Gauge (medidor)
#[derive(Debug, Clone)]
pub struct Gauge {
    pub gauge_id: u64,
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub description: String,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl Gauge {
    pub fn new(gauge_id: u64, name: String, min: f64, max: f64, description: String) -> Self {
        Self {
            gauge_id,
            name,
            value: min,
            min,
            max,
            description,
            graph_node_id: None,
        }
    }

    pub fn set(&mut self, value: f64) {
        self.value = value.max(self.min).min(self.max);
    }

    pub fn increment(&mut self) {
        self.set(self.value + 1.0);
    }

    pub fn decrement(&mut self) {
        self.set(self.value - 1.0);
    }
}

/// Histograma
#[derive(Debug, Clone)]
pub struct Histogram {
    pub histogram_id: u64,
    pub name: String,
    pub values: Vec<f64>,
    pub buckets: Vec<f64>,
    pub description: String,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl Histogram {
    pub fn new(histogram_id: u64, name: String, buckets: Vec<f64>, description: String) -> Self {
        Self {
            histogram_id,
            name,
            values: Vec::new(),
            buckets,
            description,
            graph_node_id: None,
        }
    }

    pub fn observe(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn sum(&self) -> f64 {
        self.values.iter().sum()
    }

    pub fn avg(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum() / self.values.len() as f64
        }
    }

    pub fn min(&self) -> f64 {
        self.values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    pub fn max(&self) -> f64 {
        self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
}

/// Logger
#[derive(Debug, Clone)]
pub struct Logger {
    pub logger_id: u64,
    pub logs: Vec<LogEntry>,
    pub max_logs: usize,
    pub component: String,
    pub next_log_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl Logger {
    pub fn new(logger_id: u64, component: String, max_logs: usize) -> Self {
        Self {
            logger_id,
            logs: Vec::new(),
            max_logs,
            component,
            next_log_id: 1,
            graph_kernel: None,
        }
    }

    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    pub fn log(&mut self, level: LogLevel, message: String) {
        let log_id = self.next_log_id;
        self.next_log_id += 1;

        let mut entry = LogEntry::new(log_id, level, self.component.clone(), message);

        // Registrar el log como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("log_entry_{}", log_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            entry.graph_node_id = node_id;
        }

        self.logs.push(entry);
        
        // Mantener solo los últimos max_logs logs
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
    }

    pub fn debug(&mut self, message: String) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&mut self, message: String) {
        self.log(LogLevel::Info, message);
    }

    pub fn warning(&mut self, message: String) {
        self.log(LogLevel::Warning, message);
    }

    pub fn error(&mut self, message: String) {
        self.log(LogLevel::Error, message);
    }

    pub fn critical(&mut self, message: String) {
        self.log(LogLevel::Critical, message);
    }

    pub fn get_logs(&self) -> &[LogEntry] {
        &self.logs
    }

    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.logs.iter().filter(|l| l.level == level).collect()
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }

    pub fn log_count(&self) -> usize {
        self.logs.len()
    }
}

/// Sistema de monitoreo
#[derive(Debug, Clone)]
pub struct CronosMonitoringSystem {
    pub loggers: BTreeMap<u64, Logger>,
    pub counters: BTreeMap<u64, Counter>,
    pub gauges: BTreeMap<u64, Gauge>,
    pub histograms: BTreeMap<u64, Histogram>,
    pub metrics_history: Vec<Metric>,
    pub max_metrics: usize,
    pub next_logger_id: u64,
    pub next_counter_id: u64,
    pub next_gauge_id: u64,
    pub next_histogram_id: u64,
    pub next_metric_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosMonitoringSystem {
    pub fn new(max_metrics: usize) -> Self {
        Self {
            loggers: BTreeMap::new(),
            counters: BTreeMap::new(),
            gauges: BTreeMap::new(),
            histograms: BTreeMap::new(),
            metrics_history: Vec::new(),
            max_metrics,
            next_logger_id: 1,
            next_counter_id: 1,
            next_gauge_id: 1,
            next_histogram_id: 1,
            next_metric_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel.clone()));
        for logger in self.loggers.values_mut() {
            logger.set_graph_kernel(graph_kernel.clone());
        }
    }

    /// Crear logger
    pub fn create_logger(&mut self, component: String, max_logs: usize) -> u64 {
        let logger_id = self.next_logger_id;
        self.next_logger_id += 1;

        let mut logger = Logger::new(logger_id, component, max_logs);

        // No asignar graph kernel aquí, se asigna en set_graph_kernel si es necesario

        self.loggers.insert(logger_id, logger);
        logger_id
    }

    /// Obtener logger
    pub fn get_logger(&self, logger_id: u64) -> Option<&Logger> {
        self.loggers.get(&logger_id)
    }

    /// Obtener logger mut
    pub fn get_logger_mut(&mut self, logger_id: u64) -> Option<&mut Logger> {
        self.loggers.get_mut(&logger_id)
    }

    /// Crear contador
    pub fn create_counter(&mut self, name: String, description: String) -> u64 {
        let counter_id = self.next_counter_id;
        self.next_counter_id += 1;

        let mut counter = Counter::new(counter_id, name, description);

        // Registrar el contador como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("counter_{}", counter_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            counter.graph_node_id = node_id;
        }

        self.counters.insert(counter_id, counter);
        counter_id
    }

    /// Obtener contador
    pub fn get_counter(&self, counter_id: u64) -> Option<&Counter> {
        self.counters.get(&counter_id)
    }

    /// Obtener contador mut
    pub fn get_counter_mut(&mut self, counter_id: u64) -> Option<&mut Counter> {
        self.counters.get_mut(&counter_id)
    }

    /// Incrementar contador
    pub fn increment_counter(&mut self, counter_id: u64) -> Result<(), String> {
        if let Some(counter) = self.counters.get_mut(&counter_id) {
            counter.increment();
            Ok(())
        } else {
            Err(format!("Counter {} not found", counter_id))
        }
    }

    /// Crear gauge
    pub fn create_gauge(&mut self, name: String, min: f64, max: f64, description: String) -> u64 {
        let gauge_id = self.next_gauge_id;
        self.next_gauge_id += 1;

        let mut gauge = Gauge::new(gauge_id, name, min, max, description);

        // Registrar el gauge como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("gauge_{}", gauge_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            gauge.graph_node_id = node_id;
        }

        self.gauges.insert(gauge_id, gauge);
        gauge_id
    }

    /// Obtener gauge
    pub fn get_gauge(&self, gauge_id: u64) -> Option<&Gauge> {
        self.gauges.get(&gauge_id)
    }

    /// Obtener gauge mut
    pub fn get_gauge_mut(&mut self, gauge_id: u64) -> Option<&mut Gauge> {
        self.gauges.get_mut(&gauge_id)
    }

    /// Establecer gauge
    pub fn set_gauge(&mut self, gauge_id: u64, value: f64) -> Result<(), String> {
        if let Some(gauge) = self.gauges.get_mut(&gauge_id) {
            gauge.set(value);
            Ok(())
        } else {
            Err(format!("Gauge {} not found", gauge_id))
        }
    }

    /// Crear histograma
    pub fn create_histogram(&mut self, name: String, buckets: Vec<f64>, description: String) -> u64 {
        let histogram_id = self.next_histogram_id;
        self.next_histogram_id += 1;

        let mut histogram = Histogram::new(histogram_id, name, buckets, description);

        // Registrar el histograma como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("histogram_{}", histogram_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            histogram.graph_node_id = node_id;
        }

        self.histograms.insert(histogram_id, histogram);
        histogram_id
    }

    /// Obtener histograma
    pub fn get_histogram(&self, histogram_id: u64) -> Option<&Histogram> {
        self.histograms.get(&histogram_id)
    }

    /// Obtener histograma mut
    pub fn get_histogram_mut(&mut self, histogram_id: u64) -> Option<&mut Histogram> {
        self.histograms.get_mut(&histogram_id)
    }

    /// Observar histograma
    pub fn observe_histogram(&mut self, histogram_id: u64, value: f64) -> Result<(), String> {
        if let Some(histogram) = self.histograms.get_mut(&histogram_id) {
            histogram.observe(value);
            Ok(())
        } else {
            Err(format!("Histogram {} not found", histogram_id))
        }
    }

    /// Registrar métrica
    pub fn record_metric(&mut self, name: String, value: f64, unit: String) {
        let metric_id = self.next_metric_id;
        self.next_metric_id += 1;

        let mut metric = Metric::new(metric_id, name, value, unit);

        // Registrar la métrica como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("metric_{}", metric_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            metric.graph_node_id = node_id;
        }

        self.metrics_history.push(metric);
        
        // Mantener solo los últimos max_metrics métricas
        if self.metrics_history.len() > self.max_metrics {
            self.metrics_history.remove(0);
        }
    }

    /// Obtener historial de métricas
    pub fn get_metrics_history(&self) -> &[Metric] {
        &self.metrics_history
    }

    /// Obtener métricas por nombre
    pub fn get_metrics_by_name(&self, name: &str) -> Vec<&Metric> {
        self.metrics_history.iter().filter(|m| m.name == name).collect()
    }

    /// Obtener todos los logs
    pub fn get_all_logs(&self) -> Vec<&LogEntry> {
        let mut all_logs = Vec::new();
        for logger in self.loggers.values() {
            all_logs.extend(logger.get_logs());
        }
        all_logs
    }

    /// Obtener logs por nivel
    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        let mut logs = Vec::new();
        for logger in self.loggers.values() {
            logs.extend(logger.get_logs_by_level(level));
        }
        logs
    }

    /// Obtener estadísticas del sistema
    pub fn get_system_stats(&self) -> SystemStats {
        let total_logs = self.loggers.values().map(|l| l.log_count()).sum();
        let total_counters = self.counters.len();
        let total_gauges = self.gauges.len();
        let total_histograms = self.histograms.len();
        let total_metrics = self.metrics_history.len();

        SystemStats {
            total_logs,
            total_counters,
            total_gauges,
            total_histograms,
            total_metrics,
        }
    }

    /// Limpiar todos los logs
    pub fn clear_all_logs(&mut self) {
        for logger in self.loggers.values_mut() {
            logger.clear();
        }
    }

    /// Resetear todos los contadores
    pub fn reset_all_counters(&mut self) {
        for counter in self.counters.values_mut() {
            counter.reset();
        }
    }
}

impl Default for CronosMonitoringSystem {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Estadísticas del sistema
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_logs: usize,
    pub total_counters: usize,
    pub total_gauges: usize,
    pub total_histograms: usize,
    pub total_metrics: usize,
}

/// Errores del sistema de monitoreo
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitoringError {
    LoggerNotFound,
    CounterNotFound,
    GaugeNotFound,
    HistogramNotFound,
    InvalidMetric,
}

impl fmt::Display for MonitoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitoringError::LoggerNotFound => write!(f, "Logger not found"),
            MonitoringError::CounterNotFound => write!(f, "Counter not found"),
            MonitoringError::GaugeNotFound => write!(f, "Gauge not found"),
            MonitoringError::HistogramNotFound => write!(f, "Histogram not found"),
            MonitoringError::InvalidMetric => write!(f, "Invalid metric"),
        }
    }
}
