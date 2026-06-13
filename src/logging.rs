//! Logging System Module
//! 
//! This module implements a comprehensive logging system for the kernel,
//! supporting multiple log levels, output targets, and efficient logging.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Nivel de log
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace - información muy detallada
    Trace,
    /// Debug - información de debugging
    Debug,
    /// Info - información general
    Info,
    /// Warn - advertencias
    Warn,
    /// Error - errores
    Error,
}

impl LogLevel {
    /// Convertir a string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Convertir desde string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(LogLevel::Trace),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

/// Entrada de log
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Nivel de log
    pub level: LogLevel,
    /// Mensaje
    pub message: String,
    /// Módulo origen
    pub module: String,
    /// Archivo origen
    pub file: String,
    /// Línea origen
    pub line: u32,
    /// Timestamp (simulado)
    pub timestamp: u64,
}

impl LogEntry {
    /// Crear nueva entrada de log
    pub fn new(level: LogLevel, message: String, module: String, file: String, line: u32) -> Self {
        Self {
            level,
            message,
            module,
            file,
            line,
            timestamp: 0, // En un sistema real, esto sería el tiempo actual
        }
    }

    /// Formatear entrada de log
    pub fn format(&self) -> String {
        format!(
            "[{}] [{}] {}:{}: {}",
            self.level.as_str(),
            self.module,
            self.file,
            self.line,
            self.message
        )
    }

    /// Formatear entrada de log de forma compacta
    pub fn format_compact(&self) -> String {
        format!(
            "[{}] {}",
            self.level.as_str(),
            self.message
        )
    }
}

/// Destino de log
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTarget {
    /// Salida serial
    Serial,
    /// Buffer en memoria
    Memory,
    /// Ambos
    Both,
}

/// Configuración del logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Nivel mínimo de log
    pub min_level: LogLevel,
    /// Destino de log
    pub target: LogTarget,
    /// Si incluir timestamp
    pub include_timestamp: bool,
    /// Si incluir ubicación (archivo/línea)
    pub include_location: bool,
    /// Tamaño máximo del buffer en memoria
    pub max_buffer_size: usize,
}

impl LoggerConfig {
    /// Crear configuración por defecto
    pub fn default_config() -> Self {
        Self {
            min_level: LogLevel::Info,
            target: LogTarget::Serial,
            include_timestamp: true,
            include_location: true,
            max_buffer_size: 4096,
        }
    }

    /// Crear configuración para debugging
    pub fn debug_config() -> Self {
        Self {
            min_level: LogLevel::Debug,
            target: LogTarget::Both,
            include_timestamp: true,
            include_location: true,
            max_buffer_size: 8192,
        }
    }

    /// Crear configuración minimal
    pub fn minimal_config() -> Self {
        Self {
            min_level: LogLevel::Warn,
            target: LogTarget::Serial,
            include_timestamp: false,
            include_location: false,
            max_buffer_size: 1024,
        }
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Sistema de logging
pub struct Logger {
    /// Configuración
    config: LoggerConfig,
    /// Buffer en memoria
    memory_buffer: Vec<LogEntry>,
    /// Número de logs perdidos por buffer lleno
    lost_logs: usize,
    /// Número total de logs
    total_logs: usize,
}

impl Logger {
    /// Crear nuevo logger
    pub fn new(config: LoggerConfig) -> Self {
        Self {
            config,
            memory_buffer: Vec::new(),
            lost_logs: 0,
            total_logs: 0,
        }
    }

    /// Crear logger con configuración por defecto
    pub fn with_default_config() -> Self {
        Self::new(LoggerConfig::default_config())
    }

    /// Loggear mensaje
    pub fn log(&mut self, level: LogLevel, message: String, module: String, file: String, line: u32) {
        // Verificar nivel mínimo
        if level < self.config.min_level {
            return;
        }

        self.total_logs += 1;

        let entry = LogEntry::new(level, message, module, file, line);

        // Output a serial
        if self.config.target == LogTarget::Serial || self.config.target == LogTarget::Both {
            let formatted = if self.config.include_location {
                entry.format()
            } else {
                entry.format_compact()
            };
            self.write_to_serial(&formatted);
        }

        // Output a memoria
        if self.config.target == LogTarget::Memory || self.config.target == LogTarget::Both {
            if self.memory_buffer.len() >= self.config.max_buffer_size {
                // Buffer lleno, remover entrada más antigua
                self.memory_buffer.remove(0);
                self.lost_logs += 1;
            }
            self.memory_buffer.push(entry);
        }
    }

    /// Loggear a nivel trace
    pub fn trace(&mut self, message: String, module: String, file: String, line: u32) {
        self.log(LogLevel::Trace, message, module, file, line);
    }

    /// Loggear a nivel debug
    pub fn debug(&mut self, message: String, module: String, file: String, line: u32) {
        self.log(LogLevel::Debug, message, module, file, line);
    }

    /// Loggear a nivel info
    pub fn info(&mut self, message: String, module: String, file: String, line: u32) {
        self.log(LogLevel::Info, message, module, file, line);
    }

    /// Loggear a nivel warn
    pub fn warn(&mut self, message: String, module: String, file: String, line: u32) {
        self.log(LogLevel::Warn, message, module, file, line);
    }

    /// Loggear a nivel error
    pub fn error(&mut self, message: String, module: String, file: String, line: u32) {
        self.log(LogLevel::Error, message, module, file, line);
    }

    /// Escribir a serial
    fn write_to_serial(&self, message: &str) {
        // En un sistema real, esto escribiría al puerto serial
        // Para este ejemplo, no hacemos nada
    }

    /// Obtener buffer de memoria
    pub fn get_memory_buffer(&self) -> &Vec<LogEntry> {
        &self.memory_buffer
    }

    /// Limpiar buffer de memoria
    pub fn clear_memory_buffer(&mut self) {
        self.memory_buffer.clear();
        self.lost_logs = 0;
    }

    /// Obtener número de logs perdidos
    pub fn get_lost_logs(&self) -> usize {
        self.lost_logs
    }

    /// Obtener número total de logs
    pub fn get_total_logs(&self) -> usize {
        self.total_logs
    }

    /// Establecer nivel mínimo
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.config.min_level = level;
    }

    /// Establecer destino de log
    pub fn set_target(&mut self, target: LogTarget) {
        self.config.target = target;
    }

    /// Obtener estadísticas
    pub fn get_stats(&self) -> LogStats {
        let mut stats = LogStats::new();
        stats.total_logs = self.total_logs;
        stats.lost_logs = self.lost_logs;
        stats.buffer_size = self.memory_buffer.len();
        
        for entry in &self.memory_buffer {
            match entry.level {
                LogLevel::Trace => stats.trace_count += 1,
                LogLevel::Debug => stats.debug_count += 1,
                LogLevel::Info => stats.info_count += 1,
                LogLevel::Warn => stats.warn_count += 1,
                LogLevel::Error => stats.error_count += 1,
            }
        }
        
        stats
    }

    /// Exportar logs a string
    pub fn export_logs(&self) -> String {
        let mut output = String::from("Log Export\n");
        output.push_str("===========\n\n");
        
        for entry in &self.memory_buffer {
            output.push_str(&entry.format());
            output.push('\n');
        }
        
        output.push_str(&format!("\nTotal: {}, Lost: {}\n", self.total_logs, self.lost_logs));
        
        output
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::with_default_config()
    }
}

/// Estadísticas de logging
#[derive(Debug, Clone)]
pub struct LogStats {
    /// Número total de logs
    pub total_logs: usize,
    /// Logs perdidos
    pub lost_logs: usize,
    /// Tamaño del buffer
    pub buffer_size: usize,
    /// Count de trace
    pub trace_count: usize,
    /// Count de debug
    pub debug_count: usize,
    /// Count de info
    pub info_count: usize,
    /// Count de warn
    pub warn_count: usize,
    /// Count de error
    pub error_count: usize,
}

impl LogStats {
    /// Crear nuevas estadísticas
    pub fn new() -> Self {
        Self {
            total_logs: 0,
            lost_logs: 0,
            buffer_size: 0,
            trace_count: 0,
            debug_count: 0,
            info_count: 0,
            warn_count: 0,
            error_count: 0,
        }
    }

    /// Generar reporte
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Logging Statistics\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("Total logs: {}\n", self.total_logs));
        report.push_str(&format!("Lost logs: {}\n", self.lost_logs));
        report.push_str(&format!("Buffer size: {}\n\n", self.buffer_size));
        
        report.push_str("By level:\n");
        report.push_str(&format!("  TRACE: {}\n", self.trace_count));
        report.push_str(&format!("  DEBUG: {}\n", self.debug_count));
        report.push_str(&format!("  INFO: {}\n", self.info_count));
        report.push_str(&format!("  WARN: {}\n", self.warn_count));
        report.push_str(&format!("  ERROR: {}\n", self.error_count));
        
        report
    }
}

impl Default for LogStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro para logging
#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $($arg:tt)*) => {
        $logger.trace(
            alloc::format!($($arg)*),
            module_path!().to_string(),
            file!().to_string(),
            line!()
        );
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.debug(
            alloc::format!($($arg)*),
            module_path!().to_string(),
            file!().to_string(),
            line!()
        );
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(
            alloc::format!($($arg)*),
            module_path!().to_string(),
            file!().to_string(),
            line!()
        );
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warn(
            alloc::format!($($arg)*),
            module_path!().to_string(),
            file!().to_string(),
            line!()
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(
            alloc::format!($($arg)*),
            module_path!().to_string(),
            file!().to_string(),
            line!()
        );
    };
}

/// Utilidades para logging
pub struct LoggingUtils;

impl LoggingUtils {
    /// Verificar si el logging está habilitado
    pub fn is_logging_enabled() -> bool {
        // En un sistema real, esto verificaría una configuración
        true
    }

    /// Obtener configuración por defecto
    pub fn get_default_config() -> LoggerConfig {
        LoggerConfig::default_config()
    }

    /// Crear logger global
    pub fn create_global_logger() -> Logger {
        Logger::with_default_config()
    }

    /// Limpiar todos los logs
    pub fn clear_all_logs(logger: &mut Logger) {
        logger.clear_memory_buffer();
    }

    /// Exportar logs a archivo (simulado)
    pub fn export_to_file(logger: &Logger, path: &str) -> Result<(), String> {
        // En un sistema real, esto escribiría a un archivo
        let logs = logger.export_logs();
        Ok(())
    }
}
