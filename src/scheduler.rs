//! Scheduler con Preemptive Multitasking para CRONOS W-OS
//!
//! Este módulo implementa el scheduler de procesos con preemptive multitasking,
//! adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Cell, CapabilityId, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId};
use crate::address_space::AddressSpace;

/// Estado del proceso
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    /// Listo para ejecutar
    Ready,
    /// Ejecutándose
    Running,
    /// Bloqueado esperando I/O
    Blocked,
    /// Terminado
    Terminated,
    /// Zombie (esperando que padre recoja)
    Zombie,
}

/// Prioridad de proceso (Orden de declaración define prioridad: Realtime > High > Normal > Low > Idle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    Realtime = 0,
    High = 5,
    Normal = 10,
    Low = 20,
    Idle = 31,
}

impl ProcessPriority {
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

/// Contexto de ejecución del proceso
#[derive(Debug, Clone)]
pub struct ProcessContext {
    /// Registros generales
    pub registers: [u64; 16],
    /// Puntero de pila
    pub rsp: u64,
    /// Puntero de base
    pub rbp: u64,
    /// Puntero de instrucción
    pub rip: u64,
    /// Flags del procesador
    pub rflags: u64,
    /// CR3 (Page Directory Base)
    pub cr3: u64,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            registers: [0; 16],
            rsp: 0,
            rbp: 0,
            rip: 0,
            rflags: 0,
            cr3: 0,
        }
    }
}

/// Información de tiempo del proceso
#[derive(Debug, Clone)]
pub struct ProcessTime {
    /// Tiempo total de CPU en nanosegundos
    pub total_cpu_time_ns: u64,
    /// Tiempo de CPU en el último quantum
    pub last_quantum_ns: u64,
    /// Número de quanta ejecutados
    pub quanta_executed: u64,
    /// Tiempo de inicio del quantum actual (BUG #6 corregido)
    pub quantum_start_ns: u64,
    /// Virtual runtime para CFS (BUG #7 corregido)
    pub vruntime: u64,
}

impl Default for ProcessTime {
    fn default() -> Self {
        Self {
            total_cpu_time_ns: 0,
            last_quantum_ns: 0,
            quanta_executed: 0,
            quantum_start_ns: 0,
            vruntime: 0,
        }
    }
}

/// Proceso
pub struct Process {
    /// ID del proceso
    pub id: u64,
    /// ID del proceso padre
    pub parent_id: Option<u64>,
    /// Estado del proceso
    pub state: ProcessState,
    /// Prioridad del proceso
    pub priority: ProcessPriority,
    /// Contexto de ejecución
    pub context: ProcessContext,
    /// Información de tiempo
    pub time_info: ProcessTime,
    /// Capabilities del proceso
    pub capabilities: BTreeSet<CapabilityId>,
    /// Nombre del proceso
    pub name: String,
    /// Stack del proceso (para hilos del kernel)
    pub stack: Vec<u8>,
    /// Espacio de direcciones virtual (Aislamiento AEGIS)
    pub address_space: Option<AddressSpace>,
    /// ID del nodo en el grafo que representa este proceso
    pub graph_node_id: Option<NodeId>,
}

/// Punto de entrada inicial para nuevos procesos del kernel
extern "C" fn process_entry_stub() {
    crate::serial_println!("[KERNEL] Proceso iniciado satisfactoriamente.");
    loop {
        x86_64::instructions::hlt();
    }
}

impl Process {
    pub fn new(id: u64, name: String, priority: ProcessPriority) -> Self {
        let stack_size = 4096 * 4; // 16KB stack
        let mut stack = vec![0u8; stack_size];
        let mut context = ProcessContext::default();

        // Preparar el stack para el primer context_switch_asm
        // El layout físico debe ser (de arriba a abajo): [RIP] [RBP] [RBX] [R12] [R13] [R14] [R15]
        unsafe {
            let stack_ptr = stack.as_mut_ptr().add(stack_size) as *mut u64;

            // Colocamos el RIP (dirección de retorno para el 'ret' de context_switch_asm)
            stack_ptr.offset(-1).write(process_entry_stub as *const () as u64);

            // Colocamos los 6 registros callee-saved inicializados a 0
            stack_ptr.offset(-2).write(0); // RBP
            stack_ptr.offset(-3).write(0); // RBX
            stack_ptr.offset(-4).write(0); // R12
            stack_ptr.offset(-5).write(0); // R13
            stack_ptr.offset(-6).write(0); // R14
            stack_ptr.offset(-7).write(0); // R15

            context.rsp = stack_ptr.offset(-7) as u64;
        }

        Self {
            id,
            parent_id: None,
            state: ProcessState::Ready,
            priority,
            context,
            time_info: ProcessTime::default(),
            capabilities: BTreeSet::new(),
            name,
            stack,
            address_space: Some(AddressSpace::new(id as u32)),
            graph_node_id: None,
        }
    }

    /// Verificar si el proceso está listo para ejecutar
    pub fn is_ready(&self) -> bool {
        self.state == ProcessState::Ready
    }

    /// Verificar si el proceso está ejecutándose
    pub fn is_running(&self) -> bool {
        self.state == ProcessState::Running
    }

    /// Verificar si el proceso está bloqueado
    pub fn is_blocked(&self) -> bool {
        self.state == ProcessState::Blocked
    }

    /// Verificar si el proceso está terminado
    pub fn is_terminated(&self) -> bool {
        self.state == ProcessState::Terminated || self.state == ProcessState::Zombie
    }
}

/// Política de scheduling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulingPolicy {
    /// Round Robin
    RoundRobin,
    /// Priority-based
    Priority,
    /// Completely Fair Scheduler (CFS)
    CFS,
    /// Real-time
    RealTime,
}

/// Configuración del scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Política de scheduling
    pub policy: SchedulingPolicy,
    /// Quantum de tiempo en milisegundos
    pub quantum_ms: u32,
    /// Número máximo de procesos
    pub max_processes: usize,
    /// Habilitar preemptive scheduling
    pub enable_preemption: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            policy: SchedulingPolicy::RoundRobin,
            quantum_ms: 10,
            max_processes: 1024,
            enable_preemption: true,
        }
    }
}

/// Scheduler de procesos
pub struct Scheduler {
    /// Procesos en el sistema
    pub processes: BTreeMap<u64, Process>,
    /// Cola de procesos listos
    pub ready_queue: Vec<u64>,
    /// Proceso actualmente ejecutándose
    pub current_process: Option<u64>,
    /// Configuración del scheduler
    pub config: SchedulerConfig,
    /// Próximo ID de proceso
    pub next_process_id: u64,
    /// Referencia al graph kernel
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// FASE 3.1: Proceso/Ventana con foco para Boosting adaptativo
    pub focused_process_id: Option<u64>,
    /// Estadísticas del scheduler
    pub stats: SchedulerStats,
}

/// Estadísticas del scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// Total de procesos creados
    pub total_processes_created: u64,
    /// Total de procesos terminados
    pub total_processes_terminated: u64,
    /// Total de context switches
    pub total_context_switches: u64,
    /// Total de quanta ejecutados
    pub total_quanta_executed: u64,
    /// Tiempo total de CPU usado
    pub total_cpu_time_ns: u64,
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self {
            total_processes_created: 0,
            total_processes_terminated: 0,
            total_context_switches: 0,
            total_quanta_executed: 0,
            total_cpu_time_ns: 0,
        }
    }
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            processes: BTreeMap::new(),
            ready_queue: Vec::new(),
            current_process: None,
            config,
            next_process_id: 1,
            graph_kernel: None,
            focused_process_id: None,
            stats: SchedulerStats::default(),
        }
    }

    /// FASE 3.1: Boosting adaptativo para aplicaciones en primer plano
    pub fn set_foreground_process(&mut self, process_id: u64) {
        self.focused_process_id = Some(process_id);
        if let Some(process) = self.get_process_mut(process_id) {
            // Boosting temporal: Si es Normal, subir a High. Si es Low, subir a Normal.
            process.priority = match process.priority {
                ProcessPriority::Normal => ProcessPriority::High,
                ProcessPriority::Low => ProcessPriority::Normal,
                p => p, // Mantener Realtime o High
            };
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Crear un nuevo proceso
    pub fn create_process(&mut self, name: String, priority: ProcessPriority) -> Result<u64, String> {
        if self.processes.len() >= self.config.max_processes {
            return Err(String::from("Maximum number of processes reached"));
        }

        let process_id = self.next_process_id;
        self.next_process_id += 1;

        let mut process = Process::new(process_id, name, priority);

        // Registrar el proceso como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::NodeType;
            let node_type = NodeType::Process;
            let node_name = format!("process_{}", process_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            process.graph_node_id = node_id;
        }

        self.processes.insert(process_id, process);
        self.ready_queue.push(process_id);
        self.stats.total_processes_created += 1;

        Ok(process_id)
    }

    /// Obtener un proceso por ID
    pub fn get_process(&self, process_id: u64) -> Option<&Process> {
        self.processes.get(&process_id)
    }

    /// Obtener un proceso mutable por ID
    pub fn get_process_mut(&mut self, process_id: u64) -> Option<&mut Process> {
        self.processes.get_mut(&process_id)
    }

    /// Seleccionar el siguiente proceso a ejecutar
    pub fn schedule(&mut self) -> Option<u64> {
        match self.config.policy {
            SchedulingPolicy::RoundRobin => self.schedule_round_robin(),
            SchedulingPolicy::Priority => self.schedule_priority(),
            SchedulingPolicy::CFS => self.schedule_cfs(),
            SchedulingPolicy::RealTime => self.schedule_realtime(),
        }
    }

    /// Scheduling Round Robin
    fn schedule_round_robin(&mut self) -> Option<u64> {
        if self.ready_queue.is_empty() {
            return None;
        }

        // Mover el proceso actual al final de la cola si está listo
        if let Some(current_id) = self.current_process {
            if let Some(process) = self.get_process(current_id) {
                if process.is_ready() {
                    self.ready_queue.push(current_id);
                }
            }
        }

        // Seleccionar el siguiente proceso
        let next_id = self.ready_queue.remove(0);
        self.current_process = Some(next_id);

        // BUG #6 corregido: calcular quantum_start_ns antes de tomar préstamo mutable
        let quantum_ns = self.config.quantum_ms as u64 * 1_000_000;
        let quantum_start_ns = self.stats.total_quanta_executed * quantum_ns;

        if let Some(process) = self.get_process_mut(next_id) {
            process.state = ProcessState::Running;
            process.time_info.quantum_start_ns = quantum_start_ns;
        }

        self.stats.total_context_switches += 1;
        Some(next_id)
    }

    /// Scheduling por prioridad
    fn schedule_priority(&mut self) -> Option<u64> {
        if self.ready_queue.is_empty() {
            return None;
        }

        // Extraer prioridades antes de ordenar
        let mut process_priorities: Vec<(u64, ProcessPriority)> = self.ready_queue.iter()
            .map(|&id| {
                let priority = self.get_process(id).map(|p| p.priority).unwrap_or(ProcessPriority::Idle);
                (id, priority)
            })
            .collect();

        // Ordenar por prioridad (menor valor es mayor prioridad)
        process_priorities.sort_by(|a, b| a.1.cmp(&b.1));

        // Reconstruir la ready_queue ordenada
        self.ready_queue = process_priorities.iter().map(|(id, _)| *id).collect();

        let next_id = self.ready_queue.remove(0);
        self.current_process = Some(next_id);

        // BUG #6 corregido: calcular quantum_start_ns antes de tomar préstamo mutable
        let quantum_ns = self.config.quantum_ms as u64 * 1_000_000;
        let quantum_start_ns = self.stats.total_quanta_executed * quantum_ns;

        if let Some(process) = self.get_process_mut(next_id) {
            process.state = ProcessState::Running;
            process.time_info.quantum_start_ns = quantum_start_ns;
        }

        self.stats.total_context_switches += 1;
        Some(next_id)
    }

    /// Scheduling CFS (Completely Fair Scheduler)
    fn schedule_cfs(&mut self) -> Option<u64> {
        // BUG #7 corregido: implementar CFS real con vruntime
        if self.ready_queue.is_empty() {
            return None;
        }

        // Encontrar el proceso con menor vruntime en la ready_queue
        let mut min_vruntime = u64::MAX;
        let mut selected_id = None;

        for &id in &self.ready_queue {
            if let Some(process) = self.get_process(id) {
                if process.time_info.vruntime < min_vruntime {
                    min_vruntime = process.time_info.vruntime;
                    selected_id = Some(id);
                }
            }
        }

        if let Some(next_id) = selected_id {
            // Remover el proceso seleccionado de la cola
            self.ready_queue.retain(|&id| id != next_id);
            self.current_process = Some(next_id);

            // BUG #6 corregido: calcular quantum_start_ns antes de tomar préstamo mutable
            let quantum_ns = self.config.quantum_ms as u64 * 1_000_000;
            let quantum_start_ns = self.stats.total_quanta_executed * quantum_ns;

            if let Some(process) = self.get_process_mut(next_id) {
                process.state = ProcessState::Running;
                process.time_info.quantum_start_ns = quantum_start_ns;
            }

            self.stats.total_context_switches += 1;
            Some(next_id)
        } else {
            // Fallback a Round Robin si no se puede encontrar por vruntime
            self.schedule_round_robin()
        }
    }

    /// Scheduling Real-time
    fn schedule_realtime(&mut self) -> Option<u64> {
        // Simplificación: usar Priority como base
        self.schedule_priority()
    }

    /// Cambio de contexto real (físico)
    pub fn context_switch(&mut self, from: Option<u64>, to: Option<u64>) {
        if from == to {
            return;
        }

        self.stats.total_context_switches += 1;

        // 1. Actualizar estados en memoria
        if let Some(from_id) = from {
            if let Some(process) = self.get_process_mut(from_id) {
                process.state = ProcessState::Ready;
            }
        }

        if let Some(to_id) = to {
            if let Some(process) = self.get_process_mut(to_id) {
                process.state = ProcessState::Running;
                self.current_process = Some(to_id);
            }
        }

        // 2. Realizar el cambio físico si ambos existen
        if let (Some(from_id), Some(to_id)) = (from, to) {
            // Obtenemos los punteros de pila de forma segura
            let old_rsp_ptr = &mut self.processes.get_mut(&from_id).unwrap().context.rsp as *mut u64;
            let new_rsp = self.processes.get(&to_id).unwrap().context.rsp;

            unsafe {
                crate::context_switch_asm(old_rsp_ptr, new_rsp);
            }
        }
    }

    /// Tick del scheduler (llamado por el timer)
    pub fn tick(&mut self) {
        self.stats.total_quanta_executed += 1;

        let quantum_ns = self.config.quantum_ms as u64 * 1_000_000;
        let enable_preemption = self.config.enable_preemption;
        let current_time_ns = self.stats.total_quanta_executed * quantum_ns;
        let is_cfs = self.config.policy == SchedulingPolicy::CFS;

        // BUG #9 corregido: incrementar KERNEL_TICK global
        crate::graph_kernel::increment_kernel_tick();

        if let Some(current_id) = self.current_process {
            // BUG #6 corregido: calcular elapsed_ns antes de tomar préstamo mutable
            let elapsed_ns = if let Some(process) = self.get_process(current_id) {
                current_time_ns.saturating_sub(process.time_info.quantum_start_ns)
            } else {
                0
            };

            if let Some(process) = self.get_process_mut(current_id) {
                process.time_info.total_cpu_time_ns += quantum_ns;
                
                // BUG #7 corregido: actualizar vruntime para CFS
                if is_cfs {
                    // vruntime = delta_time * weight_factor
                    // weight_factor = 1024 / nice_weight (simplificado a 1.0 para todos)
                    let weight_factor = 1024u64;
                    let delta_vruntime = (elapsed_ns * weight_factor) / 1024;
                    process.time_info.vruntime += delta_vruntime;
                }
                
                // Preemptive scheduling si está habilitado
                if enable_preemption {
                    // Solo preemper si el proceso ha excedido su quantum
                    if elapsed_ns >= quantum_ns {
                        process.time_info.quanta_executed += 1;
                        process.time_info.last_quantum_ns = elapsed_ns;
                        process.time_info.quantum_start_ns = current_time_ns;
                        
                        let next = self.schedule();
                        if next != Some(current_id) {
                            self.context_switch(self.current_process, next);
                        }
                    }
                }
            }
        }
    }

    /// Terminar un proceso
    pub fn terminate_process(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(process) = self.get_process_mut(process_id) {
            process.state = ProcessState::Terminated;
            
            // Remover de la cola de listos
            self.ready_queue.retain(|&id| id != process_id);
            
            // Si era el proceso actual, seleccionar otro
            if self.current_process == Some(process_id) {
                self.current_process = None;
                let next = self.schedule();
                self.context_switch(Some(process_id), next);
            }

            self.stats.total_processes_terminated += 1;
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Bloquear un proceso
    pub fn block_process(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(process) = self.get_process_mut(process_id) {
            process.state = ProcessState::Blocked;
            self.ready_queue.retain(|&id| id != process_id);
            
            if self.current_process == Some(process_id) {
                self.current_process = None;
                let next = self.schedule();
                self.context_switch(Some(process_id), next);
            }
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Desbloquear un proceso
    pub fn unblock_process(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(process) = self.get_process_mut(process_id) {
            process.state = ProcessState::Ready;
            self.ready_queue.push(process_id);
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Obtener el proceso actual
    pub fn current_process(&self) -> Option<&Process> {
        self.current_process.and_then(|id| self.get_process(id))
    }

    /// Obtener el número de procesos
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    /// Obtener el número de procesos listos
    pub fn ready_process_count(&self) -> usize {
        self.ready_queue.len()
    }

    /// Obtener estadísticas
    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(SchedulerConfig::default())
    }
}

/// Errores del scheduler
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    ProcessNotFound,
    ProcessAlreadyExists,
    MaximumProcessesReached,
    InvalidPriority,
    ContextSwitchFailed,
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::ProcessNotFound => write!(f, "Process not found"),
            SchedulerError::ProcessAlreadyExists => write!(f, "Process already exists"),
            SchedulerError::MaximumProcessesReached => write!(f, "Maximum processes reached"),
            SchedulerError::InvalidPriority => write!(f, "Invalid priority"),
            SchedulerError::ContextSwitchFailed => write!(f, "Context switch failed"),
        }
    }
}
