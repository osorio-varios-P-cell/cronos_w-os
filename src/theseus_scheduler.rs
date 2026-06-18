//! Técnicas de Theseus para Scheduler (Intrusive Data Structures)
//!
//! Este módulo incorpora las técnicas de intrusive data structures de Theseus OS
//! para optimizar el scheduler de CRONOS W-OS, adaptadas a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Nodo de lista enlazada intrusiva (inspirado en Theseus)
#[derive(Debug, Clone)]
pub struct IntrusiveListNode {
    pub prev: Option<*mut IntrusiveListNode>,
    pub next: Option<*mut IntrusiveListNode>,
}

impl IntrusiveListNode {
    pub fn new() -> Self {
        Self {
            prev: None,
            next: None,
        }
    }

    /// Verificar si el nodo está en una lista
    pub fn is_linked(&self) -> bool {
        self.prev.is_some() || self.next.is_some()
    }
}

impl Default for IntrusiveListNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Lista enlazada intrusiva (inspirado en Theseus)
#[derive(Debug, Clone)]
pub struct IntrusiveList<T> {
    pub head: Option<*mut IntrusiveListNode>,
    pub tail: Option<*mut IntrusiveListNode>,
    pub count: usize,
    pub _phantom: core::marker::PhantomData<T>,
}

impl<T> IntrusiveList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            count: 0,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Agregar al frente de la lista
    pub fn push_front(&mut self, node: *mut IntrusiveListNode) {
        unsafe {
            (*node).next = self.head;
            (*node).prev = None;

            if let Some(head) = self.head {
                (*head).prev = Some(node);
            } else {
                self.tail = Some(node);
            }

            self.head = Some(node);
            self.count += 1;
        }
    }

    /// Agregar al final de la lista
    pub fn push_back(&mut self, node: *mut IntrusiveListNode) {
        unsafe {
            (*node).prev = self.tail;
            (*node).next = None;

            if let Some(tail) = self.tail {
                (*tail).next = Some(node);
            } else {
                self.head = Some(node);
            }

            self.tail = Some(node);
            self.count += 1;
        }
    }

    /// Remover de la lista
    pub fn remove(&mut self, node: *mut IntrusiveListNode) {
        unsafe {
            let prev = (*node).prev;
            let next = (*node).next;

            if let Some(prev) = prev {
                (*prev).next = next;
            } else {
                self.head = next;
            }

            if let Some(next) = next {
                (*next).prev = prev;
            } else {
                self.tail = prev;
            }

            (*node).prev = None;
            (*node).next = None;
            self.count -= 1;
        }
    }

    /// Obtener el primer elemento
    pub fn front(&self) -> Option<*mut IntrusiveListNode> {
        self.head
    }

    /// Obtener el último elemento
    pub fn back(&self) -> Option<*mut IntrusiveListNode> {
        self.tail
    }

    /// Verificar si está vacía
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Obtener el número de elementos
    pub fn len(&self) -> usize {
        self.count
    }
}

impl<T> Default for IntrusiveList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Proceso con nodo intrusivo (inspirado en Theseus)
#[derive(Debug, Clone)]
pub struct IntrusiveProcess {
    pub process_id: u64,
    pub priority: u8,
    pub state: ProcessState,
    pub node: IntrusiveListNode,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

/// Estado del proceso
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    /// Listo para ejecutar
    Ready,
    /// Ejecutando
    Running,
    /// Bloqueado
    Blocked,
    /// Terminado
    Terminated,
}

impl IntrusiveProcess {
    pub fn new(process_id: u64, priority: u8) -> Self {
        Self {
            process_id,
            priority,
            state: ProcessState::Ready,
            node: IntrusiveListNode::new(),
            graph_node_id: None,
        }
    }

    /// Obtener el nodo intrusivo
    pub fn node(&mut self) -> *mut IntrusiveListNode {
        &mut self.node
    }
}

/// Cola de prioridad intrusiva (inspirado en Theseus)
#[derive(Debug, Clone)]
pub struct IntrusivePriorityQueue {
    pub queues: [IntrusiveList<IntrusiveProcess>; 32], // 32 niveles de prioridad
    pub bitmap: u32, // Bitmap para encontrar la cola no vacía de mayor prioridad
}

impl IntrusivePriorityQueue {
    pub fn new() -> Self {
        Self {
            queues: [
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
                IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(), IntrusiveList::new(),
            ],
            bitmap: 0,
        }
    }

    /// Agregar un proceso a la cola de prioridad
    pub fn enqueue(&mut self, process: *mut IntrusiveProcess) {
        unsafe {
            let priority = (*process).priority as usize;
            if priority < 32 {
                self.queues[priority].push_back((*process).node());
                self.bitmap |= 1 << priority;
            }
        }
    }

    /// Remover un proceso de la cola de prioridad
    pub fn dequeue(&mut self, process: *mut IntrusiveProcess) {
        unsafe {
            let priority = (*process).priority as usize;
            if priority < 32 {
                self.queues[priority].remove((*process).node());
                if self.queues[priority].is_empty() {
                    self.bitmap &= !(1 << priority);
                }
            }
        }
    }

    /// Obtener el proceso de mayor prioridad (O(1) usando bitmap)
    pub fn peek(&self) -> Option<*mut IntrusiveProcess> {
        if self.bitmap == 0 {
            return None;
        }

        // Encontrar el bit más alto establecido (mayor prioridad)
        let highest_priority = 31 - (self.bitmap.leading_zeros() as usize);
        
        unsafe {
            if let Some(node) = self.queues[highest_priority].front() {
                // Convertir el nodo de vuelta al proceso
                // En un sistema real, esto se haría con offset_of
                Some(node as *mut IntrusiveProcess)
            } else {
                None
            }
        }
    }

    /// Verificar si está vacía
    pub fn is_empty(&self) -> bool {
        self.bitmap == 0
    }

    /// Obtener el número total de procesos
    pub fn len(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum()
    }
}

impl Default for IntrusivePriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheduler optimizado con técnicas de Theseus
#[derive(Debug, Clone)]
pub struct TheseusScheduler {
    pub ready_queue: IntrusivePriorityQueue,
    pub current_process: Option<*mut IntrusiveProcess>,
    pub processes: BTreeMap<u64, IntrusiveProcess>,
    pub next_process_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
    pub preemptive_enabled: bool,
    pub time_slice_ms: u32,
}

impl TheseusScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: IntrusivePriorityQueue::new(),
            current_process: None,
            processes: BTreeMap::new(),
            next_process_id: 1,
            graph_kernel: None,
            preemptive_enabled: true,
            time_slice_ms: 10,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Crear un nuevo proceso
    pub fn create_process(&mut self, priority: u8) -> u64 {
        let process_id = self.next_process_id;
        self.next_process_id += 1;

        let mut process = IntrusiveProcess::new(process_id, priority);

        // Registrar el proceso como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::Process;
            let node_name = format!("theseus_process_{}", process_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            process.graph_node_id = node_id;
        }

        self.processes.insert(process_id, process);
        process_id
    }

    /// Agregar un proceso a la cola de listos
    pub fn add_to_ready(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.state = ProcessState::Ready;
            self.ready_queue.enqueue(process);
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Bloquear un proceso
    pub fn block_process(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.state = ProcessState::Blocked;
            self.ready_queue.dequeue(process);
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Despertar un proceso
    pub fn wake_process(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            if process.state == ProcessState::Blocked {
                process.state = ProcessState::Ready;
                self.ready_queue.enqueue(process);
            }
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Programar el siguiente proceso (O(1) con intrusive data structures)
    pub fn schedule(&mut self) -> Option<u64> {
        // Guardar el proceso actual si existe
        if let Some(current) = self.current_process {
            unsafe {
                if (*current).state == ProcessState::Running {
                    (*current).state = ProcessState::Ready;
                    self.ready_queue.enqueue(current);
                }
            }
        }

        // Obtener el siguiente proceso de mayor prioridad (O(1))
        if let Some(next) = self.ready_queue.peek() {
            self.current_process = Some(next);
            unsafe {
                (*next).state = ProcessState::Running;
                Some((*next).process_id)
            }
        } else {
            self.current_process = None;
            None
        }
    }

    /// Obtener el proceso actual
    pub fn current_process(&self) -> Option<u64> {
        self.current_process.map(|p| unsafe { (*p).process_id })
    }

    /// Obtener el número de procesos en la cola de listos
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }

    /// Obtener el número total de procesos
    pub fn total_process_count(&self) -> usize {
        self.processes.len()
    }

    /// Habilitar/deshabilitar preemptive scheduling
    pub fn set_preemptive(&mut self, enabled: bool) {
        self.preemptive_enabled = enabled;
    }

    /// Establecer el time slice
    pub fn set_time_slice(&mut self, time_slice_ms: u32) {
        self.time_slice_ms = time_slice_ms;
    }

    /// Terminar un proceso
    pub fn terminate_process(&mut self, process_id: u64) -> Result<(), String> {
        if let Some(mut process) = self.processes.remove(&process_id) {
            process.state = ProcessState::Terminated;
            self.ready_queue.dequeue(&mut process);
            if self.current_process.map(|p| unsafe { (*p).process_id }) == Some(process_id) {
                self.current_process = None;
            }
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }
}

impl Default for TheseusScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del scheduler de Theseus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheseusSchedulerError {
    ProcessNotFound,
    InvalidPriority,
    QueueFull,
}

impl fmt::Display for TheseusSchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TheseusSchedulerError::ProcessNotFound => write!(f, "Process not found"),
            TheseusSchedulerError::InvalidPriority => write!(f, "Invalid priority"),
            TheseusSchedulerError::QueueFull => write!(f, "Queue full"),
        }
    }
}
