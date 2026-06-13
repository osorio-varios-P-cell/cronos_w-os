//! Process Control Block Module
//! 
//! This module implements the Process Control Block (PCB) for process management.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Estado del proceso
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Proceso nuevo
    New,
    /// Listo para ejecutar
    Ready,
    /// Ejecutándose
    Running,
    /// Bloqueado (esperando I/O)
    Blocked,
    /// Terminado
    Terminated,
    /// Zombie (esperando que el padre lea el estado de salida)
    Zombie,
}

/// Prioridad del proceso
#[derive(Debug, Clone, Copy)]
pub struct ProcessPriority {
    /// Valor de prioridad (0-255, menor es mayor prioridad)
    pub value: u8,
}

impl ProcessPriority {
    /// Crear nueva prioridad
    pub fn new(value: u8) -> Self {
        Self { value }
    }

    /// Prioridad alta
    pub fn high() -> Self {
        Self::new(0)
    }

    /// Prioridad normal
    pub fn normal() -> Self {
        Self::new(128)
    }

    /// Prioridad baja
    pub fn low() -> Self {
        Self::new(255)
    }
}

impl Default for ProcessPriority {
    fn default() -> Self {
        Self::normal()
    }
}

/// Descriptor de archivo
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    /// Número de descriptor
    pub fd: u32,
    /// Flags del descriptor
    pub flags: u32,
    /// Offset actual
    pub offset: u64,
}

/// Tabla de descriptores de archivo
#[derive(Debug, Clone)]
pub struct FileDescriptorTable {
    /// Descriptores de archivo
    pub descriptors: Vec<Option<FileDescriptor>>,
    /// Siguiente descriptor disponible
    pub next_fd: u32,
}

impl FileDescriptorTable {
    /// Crear nueva tabla de descriptores
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            next_fd: 0,
        }
    }

    /// Agregar descriptor
    pub fn add_descriptor(&mut self, flags: u32) -> u32 {
        let fd = self.next_fd;
        self.descriptors.push(Some(FileDescriptor {
            fd,
            flags,
            offset: 0,
        }));
        self.next_fd += 1;
        fd
    }

    /// Remover descriptor
    pub fn remove_descriptor(&mut self, fd: u32) -> Result<(), String> {
        if fd as usize >= self.descriptors.len() {
            return Err(String::from("Invalid file descriptor"));
        }
        self.descriptors[fd as usize] = None;
        Ok(())
    }

    /// Obtener descriptor
    pub fn get_descriptor(&self, fd: u32) -> Option<&FileDescriptor> {
        self.descriptors.get(fd as usize)?.as_ref()
    }
}

impl Default for FileDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Información de memoria del proceso
#[derive(Debug, Clone)]
pub struct ProcessMemory {
    /// Dirección base del código
    pub code_base: u64,
    /// Tamaño del código
    pub code_size: u64,
    /// Dirección base de datos
    pub data_base: u64,
    /// Tamaño de datos
    pub data_size: u64,
    /// Dirección base del heap
    pub heap_base: u64,
    /// Tamaño del heap
    pub heap_size: u64,
    /// Dirección base del stack
    pub stack_base: u64,
    /// Tamaño del stack
    pub stack_size: u64,
}

impl ProcessMemory {
    /// Crear nueva información de memoria
    pub fn new() -> Self {
        Self {
            code_base: 0,
            code_size: 0,
            data_base: 0,
            data_size: 0,
            heap_base: 0,
            heap_size: 0,
            stack_base: 0,
            stack_size: 0,
        }
    }

    /// Calcular tamaño total de memoria
    pub fn total_size(&self) -> u64 {
        self.code_size + self.data_size + self.heap_size + self.stack_size
    }
}

impl Default for ProcessMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Información de scheduling
#[derive(Debug, Clone)]
pub struct SchedulingInfo {
    /// Prioridad del proceso
    pub priority: ProcessPriority,
    /// Tiempo de CPU usado
    pub cpu_time: u64,
    /// Tiempo de ejecución restante
    pub time_slice: u64,
    /// Número de veces que ha sido scheduled
    pub schedule_count: u64,
    /// Último tiempo de ejecución
    pub last_run_time: u64,
}

impl SchedulingInfo {
    /// Crear nueva información de scheduling
    pub fn new(priority: ProcessPriority) -> Self {
        Self {
            priority,
            cpu_time: 0,
            time_slice: 10_000_000, // 10ms
            schedule_count: 0,
            last_run_time: 0,
        }
    }

    /// Actualizar tiempo de CPU
    pub fn add_cpu_time(&mut self, delta: u64) {
        self.cpu_time += delta;
    }

    /// Resetear time slice
    pub fn reset_time_slice(&mut self) {
        self.time_slice = 10_000_000;
    }

    /// Decrementar time slice
    pub fn decrement_time_slice(&mut self, delta: u64) {
        if self.time_slice > delta {
            self.time_slice -= delta;
        } else {
            self.time_slice = 0;
        }
    }
}

impl Default for SchedulingInfo {
    fn default() -> Self {
        Self::new(ProcessPriority::default())
    }
}

/// Process Control Block
#[derive(Debug, Clone)]
pub struct ProcessControlBlock {
    /// ID del proceso (PID)
    pub pid: u32,
    /// ID del proceso padre (PPID)
    pub ppid: u32,
    /// Estado del proceso
    pub state: ProcessState,
    /// Nombre del proceso
    pub name: String,
    /// Prioridad
    pub priority: ProcessPriority,
    /// Información de memoria
    pub memory: ProcessMemory,
    /// Tabla de descriptores de archivo
    pub fd_table: FileDescriptorTable,
    /// Información de scheduling
    pub scheduling: SchedulingInfo,
    /// Código de salida (si está terminado)
    pub exit_code: Option<i32>,
    /// Tiempo de creación
    pub creation_time: u64,
    /// Tiempo de inicio
    pub start_time: u64,
    /// Tiempo de terminación
    pub termination_time: Option<u64>,
}

impl ProcessControlBlock {
    /// Crear nuevo PCB
    pub fn new(pid: u32, ppid: u32, name: String) -> Self {
        Self {
            pid,
            ppid,
            state: ProcessState::New,
            name,
            priority: ProcessPriority::default(),
            memory: ProcessMemory::default(),
            fd_table: FileDescriptorTable::default(),
            scheduling: SchedulingInfo::default(),
            exit_code: None,
            creation_time: 0,
            start_time: 0,
            termination_time: None,
        }
    }

    /// Establecer estado
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    /// Obtener estado
    pub fn get_state(&self) -> ProcessState {
        self.state
    }

    /// Establecer prioridad
    pub fn set_priority(&mut self, priority: ProcessPriority) {
        self.priority = priority;
        self.scheduling.priority = priority;
    }

    /// Obtener prioridad
    pub fn get_priority(&self) -> ProcessPriority {
        self.priority
    }

    /// Iniciar proceso
    pub fn start(&mut self) {
        self.state = ProcessState::Ready;
        self.start_time = 0; // En un sistema real, esto sería el tiempo actual
    }

    /// Terminar proceso
    pub fn terminate(&mut self, exit_code: i32) {
        self.state = ProcessState::Terminated;
        self.exit_code = Some(exit_code);
        self.termination_time = Some(0); // En un sistema real, esto sería el tiempo actual
    }

    /// Bloquear proceso
    pub fn block(&mut self) {
        self.state = ProcessState::Blocked;
    }

    /// Desbloquear proceso
    pub fn unblock(&mut self) {
        self.state = ProcessState::Ready;
    }

    /// Verificar si está vivo
    pub fn is_alive(&self) -> bool {
        matches!(self.state, ProcessState::New | ProcessState::Ready | ProcessState::Running | ProcessState::Blocked)
    }

    /// Verificar si está terminado
    pub fn is_terminated(&self) -> bool {
        matches!(self.state, ProcessState::Terminated | ProcessState::Zombie)
    }

    /// Obtener tiempo de ejecución
    pub fn get_execution_time(&self) -> u64 {
        self.scheduling.cpu_time
    }

    /// Agregar descriptor de archivo
    pub fn add_file_descriptor(&mut self, flags: u32) -> u32 {
        self.fd_table.add_descriptor(flags)
    }

    /// Remover descriptor de archivo
    pub fn remove_file_descriptor(&mut self, fd: u32) -> Result<(), String> {
        self.fd_table.remove_descriptor(fd)
    }

    /// Obtener descriptor de archivo
    pub fn get_file_descriptor(&self, fd: u32) -> Option<&FileDescriptor> {
        self.fd_table.get_descriptor(fd)
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Process Control Block Status\n");
        report.push_str("============================\n\n");
        
        report.push_str(&format!("PID: {}\n", self.pid));
        report.push_str(&format!("PPID: {}\n", self.ppid));
        report.push_str(&format!("Name: {}\n", self.name));
        report.push_str(&format!("State: {:?}\n", self.state));
        report.push_str(&format!("Priority: {}\n", self.priority.value));
        
        report.push_str("\nMemory:\n");
        report.push_str(&format!("  Code: 0x{:X} - 0x{:X} ({} bytes)\n",
            self.memory.code_base, self.memory.code_base + self.memory.code_size, self.memory.code_size));
        report.push_str(&format!("  Data: 0x{:X} - 0x{:X} ({} bytes)\n",
            self.memory.data_base, self.memory.data_base + self.memory.data_size, self.memory.data_size));
        report.push_str(&format!("  Heap: 0x{:X} - 0x{:X} ({} bytes)\n",
            self.memory.heap_base, self.memory.heap_base + self.memory.heap_size, self.memory.heap_size));
        report.push_str(&format!("  Stack: 0x{:X} - 0x{:X} ({} bytes)\n",
            self.memory.stack_base, self.memory.stack_base + self.memory.stack_size, self.memory.stack_size));
        report.push_str(&format!("  Total: {} bytes\n", self.memory.total_size()));
        
        report.push_str("\nScheduling:\n");
        report.push_str(&format!("  CPU Time: {} ns\n", self.scheduling.cpu_time));
        report.push_str(&format!("  Time Slice: {} ns\n", self.scheduling.time_slice));
        report.push_str(&format!("  Schedule Count: {}\n", self.scheduling.schedule_count));
        
        report.push_str(&format!("\nFile Descriptors: {}\n", self.fd_table.descriptors.len()));
        
        if let Some(exit_code) = self.exit_code {
            report.push_str(&format!("\nExit Code: {}\n", exit_code));
        }
        
        report
    }
}

impl Default for ProcessControlBlock {
    fn default() -> Self {
        Self::new(0, 0, String::from("idle"))
    }
}

/// Gestor de procesos
pub struct ProcessManager {
    /// Lista de procesos
    pub processes: Vec<ProcessControlBlock>,
    /// Siguiente PID disponible
    pub next_pid: u32,
}

impl ProcessManager {
    /// Crear nuevo gestor de procesos
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            next_pid: 1,
        }
    }

    /// Crear nuevo proceso
    pub fn create_process(&mut self, name: String, ppid: u32) -> u32 {
        let pid = self.next_pid;
        let pcb = ProcessControlBlock::new(pid, ppid, name);
        self.processes.push(pcb);
        self.next_pid += 1;
        pid
    }

    /// Obtener proceso por PID
    pub fn get_process(&self, pid: u32) -> Option<&ProcessControlBlock> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Obtener proceso mutable por PID
    pub fn get_process_mut(&mut self, pid: u32) -> Option<&mut ProcessControlBlock> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }

    /// Terminar proceso
    pub fn terminate_process(&mut self, pid: u32, exit_code: i32) -> Result<(), String> {
        let process = self.get_process_mut(pid)
            .ok_or_else(|| String::from("Process not found"))?;
        process.terminate(exit_code);
        Ok(())
    }

    /// Obtener número de procesos
    pub fn get_process_count(&self) -> usize {
        self.processes.len()
    }

    /// Obtener procesos listos
    pub fn get_ready_processes(&self) -> Vec<&ProcessControlBlock> {
        self.processes.iter()
            .filter(|p| p.state == ProcessState::Ready)
            .collect()
    }

    /// Obtener procesos vivos
    pub fn get_alive_processes(&self) -> Vec<&ProcessControlBlock> {
        self.processes.iter()
            .filter(|p| p.is_alive())
            .collect()
    }

    /// Limpiar procesos zombies
    pub fn reap_zombies(&mut self) -> Vec<ProcessControlBlock> {
        let zombies: Vec<_> = self.processes.iter()
            .filter(|p| p.state == ProcessState::Zombie)
            .cloned()
            .collect();
        
        self.processes.retain(|p| p.state != ProcessState::Zombie);
        
        zombies
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
