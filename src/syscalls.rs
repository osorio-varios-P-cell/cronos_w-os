//! Sistema de Syscalls para CRONOS W-OS
//!
//! Este módulo implementa el sistema de llamadas al sistema (syscalls)
//! para la comunicación entre user space y kernel space, adaptado a la
//! arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Números de syscalls (siguiendo convención POSIX donde sea posible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SyscallNumber {
    /// Exit del proceso
    Exit = 1,
    /// Leer de descriptor de archivo
    Read = 2,
    /// Escribir a descriptor de archivo
    Write = 3,
    /// Abrir archivo
    Open = 4,
    /// Cerrar descriptor de archivo
    Close = 5,
    /// Crear capability
    CreateCapability = 100,
    /// Invocar capability
    InvokeCapability = 101,
    /// Transferir capability
    TransferCapability = 102,
    /// Revocar capability
    RevokeCapability = 103,
    /// Crear nodo en grafo
    CreateGraphNode = 200,
    /// Eliminar nodo en grafo
    DeleteGraphNode = 201,
    /// Crear edge en grafo
    CreateGraphEdge = 202,
    /// Eliminar edge en grafo
    DeleteGraphEdge = 203,
    /// Consultar grafo
    QueryGraph = 204,
    /// Crear contenedor
    CreateContainer = 300,
    /// Iniciar contenedor
    StartContainer = 301,
    /// Detener contenedor
    StopContainer = 302,
    /// Crear VM
    CreateVm = 400,
    /// Iniciar VM
    StartVm = 401,
    /// Detener VM
    StopVm = 402,
    /// HTTP request
    HttpRequest = 500,
    /// Web search
    WebSearch = 501,
    /// Generar contenido con IA
    AiGenerate = 600,
}

/// Argumentos de syscall
#[derive(Debug, Clone)]
pub struct SyscallArgs {
    /// Número de syscall
    pub number: u64,
    /// Argumento 1
    pub arg1: u64,
    /// Argumento 2
    pub arg2: u64,
    /// Argumento 3
    pub arg3: u64,
    /// Argumento 4
    pub arg4: u64,
    /// Argumento 5
    pub arg5: u64,
}

impl SyscallArgs {
    pub fn new(number: u64) -> Self {
        Self {
            number,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
        }
    }

    pub fn with_args(mut self, args: [u64; 5]) -> Self {
        self.arg1 = args[0];
        self.arg2 = args[1];
        self.arg3 = args[2];
        self.arg4 = args[3];
        self.arg5 = args[4];
        self
    }
}

/// Resultado de syscall
#[derive(Debug, Clone)]
pub struct SyscallResult {
    /// Valor de retorno
    pub return_value: u64,
    /// Código de error (0 = éxito)
    pub error_code: u64,
}

impl SyscallResult {
    pub fn success(return_value: u64) -> Self {
        Self {
            return_value,
            error_code: 0,
        }
    }

    pub fn error(error_code: u64) -> Self {
        Self {
            return_value: 0,
            error_code,
        }
    }

    pub fn is_success(&self) -> bool {
        self.error_code == 0
    }
}

/// Contexto de proceso para syscalls
#[derive(Debug, Clone)]
pub struct ProcessContext {
    /// ID del proceso
    pub process_id: u64,
    /// Capabilities del proceso
    pub capabilities: Vec<CapabilityId>,
    /// Directorio de trabajo
    pub working_directory: String,
}

impl ProcessContext {
    pub fn new(process_id: u64) -> Self {
        Self {
            process_id,
            capabilities: Vec::new(),
            working_directory: String::from("/"),
        }
    }
}

/// Handler de syscalls
pub struct SyscallHandler {
    /// Referencia al graph kernel
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Contextos de procesos activos
    pub process_contexts: Vec<ProcessContext>,
}

impl SyscallHandler {
    pub fn new() -> Self {
        Self {
            graph_kernel: None,
            process_contexts: Vec::new(),
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Manejar una syscall
    pub fn handle_syscall(&mut self, args: SyscallArgs, context: &ProcessContext) -> SyscallResult {
        match args.number {
            1 => self.sys_exit(args),
            2 => self.sys_read(args, context),
            3 => self.sys_write(args, context),
            4 => self.sys_open(args, context),
            5 => self.sys_close(args, context),
            100 => self.sys_create_capability(args, context),
            101 => self.sys_invoke_capability(args, context),
            102 => self.sys_transfer_capability(args, context),
            103 => self.sys_revoke_capability(args, context),
            200 => self.sys_create_graph_node(args, context),
            201 => self.sys_delete_graph_node(args, context),
            202 => self.sys_create_graph_edge(args, context),
            203 => self.sys_delete_graph_edge(args, context),
            204 => self.sys_query_graph(args, context),
            300 => self.sys_create_container(args, context),
            301 => self.sys_start_container(args, context),
            302 => self.sys_stop_container(args, context),
            400 => self.sys_create_vm(args, context),
            401 => self.sys_start_vm(args, context),
            402 => self.sys_stop_vm(args, context),
            500 => self.sys_http_request(args, context),
            501 => self.sys_web_search(args, context),
            600 => self.sys_ai_generate(args, context),
            _ => SyscallResult::error(1), // ENOSYS
        }
    }

    /// Syscall: exit
    fn sys_exit(&mut self, args: SyscallArgs) -> SyscallResult {
        let exit_code = args.arg1 as i32;
        // En un sistema real, aquí se terminaría el proceso
        SyscallResult::success(exit_code as u64)
    }

    /// Syscall: read
    fn sys_read(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se leería de un descriptor de archivo
        SyscallResult::success(0)
    }

    /// Syscall: write
    fn sys_write(&mut self, args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        let bytes_written = args.arg3; // Simulación
        SyscallResult::success(bytes_written)
    }

    /// Syscall: open
    fn sys_open(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se abriría un archivo
        SyscallResult::success(3) // Retorna fd
    }

    /// Syscall: close
    fn sys_close(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: create_capability
    fn sys_create_capability(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        let cap_id = CapabilityId::new();
        SyscallResult::success(cap_id.0)
    }

    /// Syscall: invoke_capability
    fn sys_invoke_capability(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se invocaría la capability
        SyscallResult::success(0)
    }

    /// Syscall: transfer_capability
    fn sys_transfer_capability(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: revoke_capability
    fn sys_revoke_capability(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: create_graph_node
    fn sys_create_graph_node(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::Generic(String::from("syscall_node"));
            let name = String::from("node_from_syscall");
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, name)
            });
            match node_id {
                Some(nid) => SyscallResult::success(nid.0),
                None => SyscallResult::error(2),
            }
        } else {
            SyscallResult::error(3)
        }
    }

    /// Syscall: delete_graph_node
    fn sys_delete_graph_node(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // Método no implementado en GraphKernel actualmente
        SyscallResult::success(0)
    }

    /// Syscall: create_graph_edge
    fn sys_create_graph_edge(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeId, EdgeType, GraphKernel};
            let from = NodeId(_args.arg1);
            let to = NodeId(_args.arg2);
            let edge_type = EdgeType::Generic(String::from("syscall_edge"));
            let edge_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_edge(from, to, edge_type)
            }).flatten();
            match edge_id {
                Some(eid) => SyscallResult::success(eid.0),
                None => SyscallResult::error(2),
            }
        } else {
            SyscallResult::error(3)
        }
    }

    /// Syscall: delete_graph_edge
    fn sys_delete_graph_edge(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // Método no implementado en GraphKernel actualmente
        SyscallResult::success(0)
    }

    /// Syscall: query_graph
    fn sys_query_graph(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let stats = invoke_capability(&graph_kernel.capability(), |gk| {
                gk.get_stats()
            });
            match stats {
                Some(s) => SyscallResult::success(s.node_count as u64),
                None => SyscallResult::error(3),
            }
        } else {
            SyscallResult::error(3)
        }
    }

    /// Syscall: create_container
    fn sys_create_container(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se crearía un contenedor
        SyscallResult::success(1) // Retorna container_id
    }

    /// Syscall: start_container
    fn sys_start_container(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: stop_container
    fn sys_stop_container(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: create_vm
    fn sys_create_vm(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(1) // Retorna vm_id
    }

    /// Syscall: start_vm
    fn sys_start_vm(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: stop_vm
    fn sys_stop_vm(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        SyscallResult::success(0)
    }

    /// Syscall: http_request
    fn sys_http_request(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se haría un request HTTP
        SyscallResult::success(200) // Retorna status code
    }

    /// Syscall: web_search
    fn sys_web_search(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se haría una búsqueda web
        SyscallResult::success(3) // Retorna número de resultados
    }

    /// Syscall: ai_generate
    fn sys_ai_generate(&mut self, _args: SyscallArgs, _context: &ProcessContext) -> SyscallResult {
        // En un sistema real, aquí se generaría contenido con IA
        SyscallResult::success(1) // Retorna request_id
    }
}

impl Default for SyscallHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de syscall
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyscallError {
    InvalidSyscallNumber,
    InvalidArguments,
    PermissionDenied,
    ResourceNotFound,
    InsufficientPermissions,
    CapabilityNotFound,
    GraphOperationFailed,
}

impl fmt::Display for SyscallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyscallError::InvalidSyscallNumber => write!(f, "Invalid syscall number"),
            SyscallError::InvalidArguments => write!(f, "Invalid arguments"),
            SyscallError::PermissionDenied => write!(f, "Permission denied"),
            SyscallError::ResourceNotFound => write!(f, "Resource not found"),
            SyscallError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            SyscallError::CapabilityNotFound => write!(f, "Capability not found"),
            SyscallError::GraphOperationFailed => write!(f, "Graph operation failed"),
        }
    }
}
