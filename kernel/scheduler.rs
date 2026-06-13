//! Módulo de Scheduler de Procesos para CRONOS W-OS
//! Implementa scheduler de procesos completamente fair (CFS)

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// ID de proceso
pub type ProcessId = u64;

/// ID de hilo
pub type ThreadId = u64;

/// Estados de proceso
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    New,
    Ready,
    Running,
    Blocked,
    Terminated,
    Zombie,
}

/// Prioridad de proceso
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    Idle,
    Low,
    Normal,
    High,
    Realtime,
}

/// Proceso
#[derive(Debug, Clone)]
pub struct Process {
    pub id: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub priority: ProcessPriority,
    pub threads: Vec<Thread>,
    pub virtual_memory: u64,
    pub cpu_time: u64,
    pub nice_value: i32,
    pub vruntime: u64,
}

/// Hilo
#[derive(Debug, Clone)]
pub struct Thread {
    pub id: ThreadId,
    pub process_id: ProcessId,
    pub state: ThreadState,
    pub cpu_affinity: Vec<usize>,
    pub stack_pointer: u64,
    pub instruction_pointer: u64,
}

/// Estado de hilo
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    New,
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Políticas de scheduling
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulingPolicy {
    CFS, // Completely Fair Scheduler
    FIFO, // First-In-First-Out
    RR,  // Round Robin
    Realtime,
}

/// Scheduler de procesos
pub struct ProcessScheduler {
    processes: BTreeMap<ProcessId, Process>,
    ready_queue: Vec<ProcessId>,
    current_process: Option<ProcessId>,
    policy: SchedulingPolicy,
    min_granularity: u64,
    latency: u64,
    next_process_id: ProcessId,
    next_thread_id: ThreadId,
    cpu_cores: usize,
}

impl ProcessScheduler {
    /// Crea un nuevo scheduler de procesos
    pub fn new() -> Self {
        ProcessScheduler {
            processes: BTreeMap::new(),
            ready_queue: Vec::new(),
            current_process: None,
            policy: SchedulingPolicy::CFS,
            min_granularity: 1_000_000, // 1ms en nanosegundos
            latency: 20_000_000, // 20ms en nanosegundos
            next_process_id: 1,
            next_thread_id: 1,
            cpu_cores: 4,
        }
    }

    /// Inicializa el scheduler
    pub fn initialize(&mut self) {
        println!("⏱️ Inicializando Scheduler de Procesos...");
        println!("   - Política: {:?}", self.policy);
        println!("   - Cores CPU: {}", self.cpu_cores);
        println!("   - Latencia: {}ms", self.latency / 1_000_000);
        println!("   - Granularidad mínima: {}ms", self.min_granularity / 1_000_000);
        println!("✅ Scheduler de Procesos inicializado");
    }

    /// Crea un nuevo proceso
    pub fn create_process(&mut self, name: String, priority: ProcessPriority) -> ProcessId {
        let process_id = self.next_process_id;
        self.next_process_id += 1;

        let thread_id = self.next_thread_id;
        self.next_thread_id += 1;

        let thread = Thread {
            id: thread_id,
            process_id,
            state: ThreadState::New,
            cpu_affinity: vec![0], // Por defecto en core 0
            stack_pointer: 0,
            instruction_pointer: 0,
        };

        let process = Process {
            id: process_id,
            name,
            state: ProcessState::New,
            priority,
            threads: vec![thread],
            virtual_memory: 0,
            cpu_time: 0,
            nice_value: 0,
            vruntime: 0,
        };

        self.processes.insert(process_id, process);
        self.ready_queue.push(process_id);

        println!("📝 Proceso creado: ID={}, Priority={:?}", process_id, priority);
        process_id
    }

    /// Elimina un proceso
    pub fn kill_process(&mut self, process_id: ProcessId) -> Result<(), SchedulerError> {
        if let Some(mut process) = self.processes.remove(&process_id) {
            process.state = ProcessState::Terminated;
            self.ready_queue.retain(|&id| id != process_id);
            if self.current_process == Some(process_id) {
                self.current_process = None;
            }
            println!("💀 Proceso eliminado: ID={}", process_id);
            Ok(())
        } else {
            Err(SchedulerError::ProcessNotFound)
        }
    }

    /// Selecciona el siguiente proceso a ejecutar
    pub fn schedule(&mut self) -> Option<ProcessId> {
        match self.policy {
            SchedulingPolicy::CFS => self.schedule_cfs(),
            SchedulingPolicy::FIFO => self.schedule_fifo(),
            SchedulingPolicy::RR => self.schedule_rr(),
            SchedulingPolicy::Realtime => self.schedule_realtime(),
        }
    }

    /// Algoritmo CFS (Completely Fair Scheduler)
    fn schedule_cfs(&mut self) -> Option<ProcessId> {
        // Encontrar el proceso con menor vruntime
        let mut min_vruntime = u64::MAX;
        let mut selected_process = None;

        for &process_id in &self.ready_queue {
            if let Some(process) = self.processes.get(&process_id) {
                if process.state == ProcessState::Ready && process.vruntime < min_vruntime {
                    min_vruntime = process.vruntime;
                    selected_process = Some(process_id);
                }
            }
        }

        if let Some(process_id) = selected_process {
            self.current_process = Some(process_id);
            if let Some(process) = self.processes.get_mut(&process_id) {
                process.state = ProcessState::Running;
            }
        }

        selected_process
    }

    /// Algoritmo FIFO
    fn schedule_fifo(&mut self) -> Option<ProcessId> {
        if let Some(process_id) = self.ready_queue.first() {
            let process_id = *process_id;
            self.current_process = Some(process_id);
            if let Some(process) = self.processes.get_mut(&process_id) {
                process.state = ProcessState::Running;
            }
            Some(process_id)
        } else {
            None
        }
    }

    /// Algoritmo Round Robin
    fn schedule_rr(&mut self) -> Option<ProcessId> {
        if let Some(process_id) = self.ready_queue.first() {
            let process_id = *process_id;
            self.ready_queue.rotate_left(1);
            self.current_process = Some(process_id);
            if let Some(process) = self.processes.get_mut(&process_id) {
                process.state = ProcessState::Running;
            }
            Some(process_id)
        } else {
            None
        }
    }

    /// Algoritmo Realtime
    fn schedule_realtime(&mut self) -> Option<ProcessId> {
        // Priorizar procesos de tiempo real
        for &process_id in &self.ready_queue {
            if let Some(process) = self.processes.get(&process_id) {
                if process.priority == ProcessPriority::Realtime {
                    self.current_process = Some(process_id);
                    if let Some(p) = self.processes.get_mut(&process_id) {
                        p.state = ProcessState::Running;
                    }
                    return Some(process_id);
                }
            }
        }
        self.schedule_cfs()
    }

    /// Actualiza el tiempo de CPU de un proceso
    pub fn update_cpu_time(&mut self, process_id: ProcessId, delta: u64) {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.cpu_time += delta;
            process.vruntime += delta;
        }
    }

    /// Bloquea un proceso
    pub fn block_process(&mut self, process_id: ProcessId) -> Result<(), SchedulerError> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.state = ProcessState::Blocked;
            self.ready_queue.retain(|&id| id != process_id);
            if self.current_process == Some(process_id) {
                self.current_process = None;
            }
            Ok(())
        } else {
            Err(SchedulerError::ProcessNotFound)
        }
    }

    /// Desbloquea un proceso
    pub fn unblock_process(&mut self, process_id: ProcessId) -> Result<(), SchedulerError> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.state = ProcessState::Ready;
            self.ready_queue.push(process_id);
            Ok(())
        } else {
            Err(SchedulerError::ProcessNotFound)
        }
    }

    /// Establece la prioridad de un proceso
    pub fn set_priority(&mut self, process_id: ProcessId, priority: ProcessPriority) -> Result<(), SchedulerError> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.priority = priority;
            Ok(())
        } else {
            Err(SchedulerError::ProcessNotFound)
        }
    }

    /// Obtiene el proceso actual
    pub fn get_current_process(&self) -> Option<&Process> {
        if let Some(process_id) = self.current_process {
            self.processes.get(&process_id)
        } else {
            None
        }
    }

    /// Obtiene todos los procesos
    pub fn get_all_processes(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }

    /// Obtiene el número de procesos
    pub fn get_process_count(&self) -> usize {
        self.processes.len()
    }

    /// Establece la política de scheduling
    pub fn set_policy(&mut self, policy: SchedulingPolicy) {
        self.policy = policy;
        println!("🔄 Política de scheduling cambiada a: {:?}", policy);
    }

    /// Genera reporte del scheduler
    pub fn generate_report(&self) -> SchedulerReport {
        let total_processes = self.processes.len();
        let running_processes = self.processes.values().filter(|p| p.state == ProcessState::Running).count();
        let ready_processes = self.processes.values().filter(|p| p.state == ProcessState::Ready).count();
        let blocked_processes = self.processes.values().filter(|p| p.state == ProcessState::Blocked).count();

        SchedulerReport {
            total_processes,
            running_processes,
            ready_processes,
            blocked_processes,
            policy: self.policy.clone(),
            cpu_cores: self.cpu_cores,
        }
    }
}

/// Reporte del scheduler
#[derive(Debug, Clone)]
pub struct SchedulerReport {
    pub total_processes: usize,
    pub running_processes: usize,
    pub ready_processes: usize,
    pub blocked_processes: usize,
    pub policy: SchedulingPolicy,
    pub cpu_cores: usize,
}

/// Errores del scheduler
#[derive(Debug, Clone)]
pub enum SchedulerError {
    ProcessNotFound,
    ThreadNotFound,
    InvalidState,
    SchedulingFailed,
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessState::New => write!(f, "New"),
            ProcessState::Ready => write!(f, "Ready"),
            ProcessState::Running => write!(f, "Running"),
            ProcessState::Blocked => write!(f, "Blocked"),
            ProcessState::Terminated => write!(f, "Terminated"),
            ProcessState::Zombie => write!(f, "Zombie"),
        }
    }
}

impl fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessPriority::Idle => write!(f, "Idle"),
            ProcessPriority::Low => write!(f, "Low"),
            ProcessPriority::Normal => write!(f, "Normal"),
            ProcessPriority::High => write!(f, "High"),
            ProcessPriority::Realtime => write!(f, "Realtime"),
        }
    }
}
