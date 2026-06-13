//! Performance Profiling Module
//! 
//! This module implements performance profiling tools for the kernel,
//! including timing, function call profiling, and performance metrics.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

/// Contador de rendimiento
#[derive(Debug, Clone)]
pub struct PerformanceCounter {
    /// Nombre del contador
    pub name: String,
    /// Valor del contador
    pub value: u64,
    /// Unidad
    pub unit: String,
}

impl PerformanceCounter {
    /// Crear nuevo contador
    pub fn new(name: String, value: u64, unit: String) -> Self {
        Self { name, value, unit }
    }

    /// Incrementar contador
    pub fn increment(&mut self) {
        self.value += 1;
    }

    /// Agregar valor
    pub fn add(&mut self, value: u64) {
        self.value += value;
    }

    /// Resetear contador
    pub fn reset(&mut self) {
        self.value = 0;
    }
}

/// Métrica de función
#[derive(Debug, Clone)]
pub struct FunctionMetric {
    /// Nombre de la función
    pub function_name: String,
    /// Número de llamadas
    pub call_count: u64,
    /// Tiempo total en nanosegundos
    pub total_time_ns: u64,
    /// Tiempo mínimo en nanosegundos
    pub min_time_ns: u64,
    /// Tiempo máximo en nanosegundos
    pub max_time_ns: u64,
}

impl FunctionMetric {
    /// Crear nueva métrica
    pub fn new(function_name: String) -> Self {
        Self {
            function_name,
            call_count: 0,
            total_time_ns: 0,
            min_time_ns: u64::MAX,
            max_time_ns: 0,
        }
    }

    /// Registrar llamada
    pub fn record_call(&mut self, duration_ns: u64) {
        self.call_count += 1;
        self.total_time_ns += duration_ns;
        
        if duration_ns < self.min_time_ns {
            self.min_time_ns = duration_ns;
        }
        
        if duration_ns > self.max_time_ns {
            self.max_time_ns = duration_ns;
        }
    }

    /// Obtener tiempo promedio
    pub fn average_time_ns(&self) -> u64 {
        if self.call_count == 0 {
            0
        } else {
            self.total_time_ns / self.call_count
        }
    }

    /// Formatear reporte
    pub fn format_report(&self) -> String {
        format!(
            "{}: {} calls, total: {}ns, avg: {}ns, min: {}ns, max: {}ns",
            self.function_name,
            self.call_count,
            self.total_time_ns,
            self.average_time_ns(),
            if self.min_time_ns == u64::MAX { 0 } else { self.min_time_ns },
            self.max_time_ns
        )
    }
}

/// Scope de profiling
#[derive(Debug, Clone)]
pub struct ProfilingScope {
    /// Nombre del scope
    pub name: String,
    /// Tiempo de inicio
    pub start_time_ns: u64,
    /// Activo
    pub active: bool,
}

impl ProfilingScope {
    /// Crear nuevo scope
    pub fn new(name: String) -> Self {
        Self {
            name,
            start_time_ns: 0, // En un sistema real, esto sería el tiempo actual
            active: false,
        }
    }

    /// Iniciar scope
    pub fn start(&mut self) {
        self.start_time_ns = 0; // En un sistema real, esto sería el tiempo actual
        self.active = true;
    }

    /// Detener scope y retornar duración
    pub fn stop(&mut self) -> u64 {
        self.active = false;
        let end_time_ns: u64 = 0; // En un sistema real, esto sería el tiempo actual
        end_time_ns.saturating_sub(self.start_time_ns)
    }
}

/// Gestor de profiling
pub struct Profiler {
    /// Contadores de rendimiento
    counters: Vec<PerformanceCounter>,
    /// Métricas de funciones
    function_metrics: BTreeMap<String, FunctionMetric>,
    /// Scopes activos
    active_scopes: Vec<ProfilingScope>,
    /// Habilitado
    enabled: bool,
}

impl Profiler {
    /// Crear nuevo profiler
    pub fn new() -> Self {
        Self {
            counters: Vec::new(),
            function_metrics: BTreeMap::new(),
            active_scopes: Vec::new(),
            enabled: true,
        }
    }

    /// Habilitar profiling
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Deshabilitar profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Verificar si está habilitado
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Agregar contador
    pub fn add_counter(&mut self, name: String, value: u64, unit: String) {
        self.counters.push(PerformanceCounter::new(name, value, unit));
    }

    /// Incrementar contador por nombre
    pub fn increment_counter(&mut self, name: &str) {
        if let Some(counter) = self.counters.iter_mut().find(|c| c.name == name) {
            counter.increment();
        }
    }

    /// Agregar valor a contador por nombre
    pub fn add_to_counter(&mut self, name: &str, value: u64) {
        if let Some(counter) = self.counters.iter_mut().find(|c| c.name == name) {
            counter.add(value);
        }
    }

    /// Iniciar profiling de función
    pub fn start_function(&mut self, function_name: String) -> ProfilingScope {
        if !self.enabled {
            return ProfilingScope::new(function_name.clone());
        }

        let mut scope = ProfilingScope::new(function_name.clone());
        scope.start();
        self.active_scopes.push(scope.clone());
        scope
    }

    /// Detener profiling de función
    pub fn end_function(&mut self, scope: ProfilingScope) {
        if !self.enabled {
            return;
        }

        let function_name = scope.name.clone();
        let duration_ns = {
            let mut scope = scope;
            scope.stop()
        };

        // Remover de scopes activos
        self.active_scopes.retain(|s| s.name != function_name);

        // Registrar métrica
        let metric = self.function_metrics
            .entry(function_name.clone())
            .or_insert_with(|| FunctionMetric::new(function_name));
        metric.record_call(duration_ns);
    }

    /// Obtener métrica de función
    pub fn get_function_metric(&self, function_name: &str) -> Option<&FunctionMetric> {
        self.function_metrics.get(function_name)
    }

    /// Obtener todas las métricas
    pub fn get_all_metrics(&self) -> Vec<&FunctionMetric> {
        self.function_metrics.values().collect()
    }

    /// Obtener contadores
    pub fn get_counters(&self) -> &Vec<PerformanceCounter> {
        &self.counters
    }

    /// Resetear todas las métricas
    pub fn reset_all(&mut self) {
        for metric in self.function_metrics.values_mut() {
            metric.call_count = 0;
            metric.total_time_ns = 0;
            metric.min_time_ns = u64::MAX;
            metric.max_time_ns = 0;
        }
        
        for counter in &mut self.counters {
            counter.reset();
        }
    }

    /// Generar reporte de performance
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Performance Profiling Report\n");
        report.push_str("============================\n\n");

        // Reporte de funciones
        report.push_str("Function Metrics:\n");
        let mut metrics: Vec<_> = self.function_metrics.values().collect();
        metrics.sort_by(|a, b| b.total_time_ns.cmp(&a.total_time_ns)); // Ordenar por tiempo total
        
        for metric in metrics {
            report.push_str(&format!("  {}\n", metric.format_report()));
        }
        report.push('\n');

        // Reporte de contadores
        report.push_str("Performance Counters:\n");
        for counter in &self.counters {
            report.push_str(&format!("  {}: {} {}\n", counter.name, counter.value, counter.unit));
        }
        report.push('\n');

        // Estadísticas generales
        report.push_str("Summary:\n");
        let total_calls: u64 = self.function_metrics.values().map(|m| m.call_count).sum();
        let total_time: u64 = self.function_metrics.values().map(|m| m.total_time_ns).sum();
        
        report.push_str(&format!("  Total function calls: {}\n", total_calls));
        report.push_str(&format!("  Total execution time: {}ns\n", total_time));
        report.push_str(&format!("  Functions profiled: {}\n", self.function_metrics.len()));
        report.push_str(&format!("  Counters: {}\n", self.counters.len()));

        report
    }

    /// Identificar hotspots (funciones que más tiempo consumen)
    pub fn identify_hotspots(&self, top_n: usize) -> Vec<&FunctionMetric> {
        let mut metrics: Vec<_> = self.function_metrics.values().collect();
        metrics.sort_by(|a, b| b.total_time_ns.cmp(&a.total_time_ns));
        metrics.into_iter().take(top_n).collect()
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de memoria
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Memoria total en bytes
    pub total_memory: u64,
    /// Memoria usada en bytes
    pub used_memory: u64,
    /// Memoria libre en bytes
    pub free_memory: u64,
    /// Número de allocations
    pub allocation_count: u64,
    /// Número de deallocations
    pub deallocation_count: u64,
}

impl MemoryStats {
    /// Crear nuevas estadísticas
    pub fn new() -> Self {
        Self {
            total_memory: 0,
            used_memory: 0,
            free_memory: 0,
            allocation_count: 0,
            deallocation_count: 0,
        }
    }

    /// Actualizar estadísticas
    pub fn update(&mut self, total: u64, used: u64) {
        self.total_memory = total;
        self.used_memory = used;
        self.free_memory = total.saturating_sub(used);
    }

    /// Registrar allocation
    pub fn record_allocation(&mut self, size: u64) {
        self.allocation_count += 1;
        self.used_memory += size;
    }

    /// Registrar deallocation
    pub fn record_deallocation(&mut self, size: u64) {
        self.deallocation_count += 1;
        self.used_memory = self.used_memory.saturating_sub(size);
    }

    /// Obtener porcentaje de uso
    pub fn usage_percentage(&self) -> f32 {
        if self.total_memory == 0 {
            0.0
        } else {
            (self.used_memory as f32 / self.total_memory as f32) * 100.0
        }
    }

    /// Generar reporte
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Memory Statistics\n");
        report.push_str("=================\n\n");
        
        report.push_str(&format!("Total memory: {} bytes\n", self.total_memory));
        report.push_str(&format!("Used memory: {} bytes ({:.1}%)\n", self.used_memory, self.usage_percentage()));
        report.push_str(&format!("Free memory: {} bytes\n", self.free_memory));
        report.push_str(&format!("Allocations: {}\n", self.allocation_count));
        report.push_str(&format!("Deallocations: {}\n", self.deallocation_count));
        
        report
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de CPU
#[derive(Debug, Clone)]
pub struct CpuStats {
    /// Tiempo de CPU en nanosegundos
    pub cpu_time_ns: u64,
    /// Tiempo idle en nanosegundos
    pub idle_time_ns: u64,
    /// Número de context switches
    pub context_switches: u64,
    /// Número de interrupts
    pub interrupts: u64,
}

impl CpuStats {
    /// Crear nuevas estadísticas
    pub fn new() -> Self {
        Self {
            cpu_time_ns: 0,
            idle_time_ns: 0,
            context_switches: 0,
            interrupts: 0,
        }
    }

    /// Registrar uso de CPU
    pub fn record_cpu_time(&mut self, duration_ns: u64) {
        self.cpu_time_ns += duration_ns;
    }

    /// Registrar tiempo idle
    pub fn record_idle_time(&mut self, duration_ns: u64) {
        self.idle_time_ns += duration_ns;
    }

    /// Registrar context switch
    pub fn record_context_switch(&mut self) {
        self.context_switches += 1;
    }

    /// Registrar interrupt
    pub fn record_interrupt(&mut self) {
        self.interrupts += 1;
    }

    /// Obtener porcentaje de uso de CPU
    pub fn cpu_usage_percentage(&self) -> f32 {
        let total_time = self.cpu_time_ns + self.idle_time_ns;
        if total_time == 0 {
            0.0
        } else {
            (self.cpu_time_ns as f32 / total_time as f32) * 100.0
        }
    }

    /// Generar reporte
    pub fn generate_report(&self) -> String {
        let mut report = String::from("CPU Statistics\n");
        report.push_str("==============\n\n");
        
        report.push_str(&format!("CPU time: {}ns\n", self.cpu_time_ns));
        report.push_str(&format!("Idle time: {}ns\n", self.idle_time_ns));
        report.push_str(&format!("CPU usage: {:.1}%\n", self.cpu_usage_percentage()));
        report.push_str(&format!("Context switches: {}\n", self.context_switches));
        report.push_str(&format!("Interrupts: {}\n", self.interrupts));
        
        report
    }
}

impl Default for CpuStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro para profiling de funciones
#[macro_export]
macro_rules! profile_function {
    ($profiler:expr, $func_name:expr, $block:block) => {
        let scope = $profiler.start_function($func_name.to_string());
        let result = $block;
        $profiler.end_function(scope);
        result
    };
}

/// Utilidades para profiling
pub struct ProfilingUtils;

impl ProfilingUtils {
    /// Verificar si el profiling está habilitado
    pub fn is_profiling_enabled() -> bool {
        // En un sistema real, esto verificaría una configuración
        true
    }

    /// Obtener tiempo actual en nanosegundos
    pub fn get_current_time_ns() -> u64 {
        // En un sistema real, esto usaría TSC o similar
        0
    }

    /// Calcular diferencia de tiempo
    pub fn time_diff_ns(start: u64, end: u64) -> u64 {
        end.saturating_sub(start)
    }

    /// Crear profiler global
    pub fn create_global_profiler() -> Profiler {
        Profiler::new()
    }

    /// Exportar datos de profiling
    pub fn export_profiling_data(profiler: &Profiler) -> String {
        profiler.generate_report()
    }
}
