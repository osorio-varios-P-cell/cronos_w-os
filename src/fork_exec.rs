//! Fork/Exec Module
//! 
//! This module implements fork and exec operations for process management.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Resultado de fork
#[derive(Debug, Clone)]
pub struct ForkResult {
    /// PID del proceso hijo (en el padre) o 0 (en el hijo)
    pub pid: u32,
    /// Si es el proceso padre
    pub is_parent: bool,
    /// Si es el proceso hijo
    pub is_child: bool,
}

impl ForkResult {
    /// Crear resultado desde el padre
    pub fn parent(child_pid: u32) -> Self {
        Self {
            pid: child_pid,
            is_parent: true,
            is_child: false,
        }
    }

    /// Crear resultado desde el hijo
    pub fn child() -> Self {
        Self {
            pid: 0,
            is_parent: false,
            is_child: true,
        }
    }
}

/// Argumentos de exec
#[derive(Debug, Clone)]
pub struct ExecArgs {
    /// Nombre del programa
    pub program: String,
    /// Argumentos del programa
    pub args: Vec<String>,
    /// Variables de entorno
    pub env: Vec<String>,
}

impl ExecArgs {
    /// Crear nuevos argumentos de exec
    pub fn new(program: String) -> Self {
        Self {
            program,
            args: Vec::new(),
            env: Vec::new(),
        }
    }

    /// Agregar argumento
    pub fn add_arg(&mut self, arg: String) {
        self.args.push(arg);
    }

    /// Agregar variable de entorno
    pub fn add_env(&mut self, env: String) {
        self.env.push(env);
    }
}

/// Estado de espera
#[derive(Debug, Clone, Copy)]
pub enum WaitStatus {
    /// Proceso terminado normalmente
    Exited(i32),
    /// Proceso terminado por señal
    Signaled(i32),
    /// Proceso detenido
    Stopped(i32),
    /// Proceso continuado
    Continued,
    /// Sin cambios (WNOHANG)
    NoChange,
}

/// Gestor de fork/exec
pub struct ForkExecManager {
    /// Procesos en espera de ser reaped
    pub zombie_processes: Vec<u32>,
}

impl ForkExecManager {
    /// Crear nuevo gestor
    pub fn new() -> Self {
        Self {
            zombie_processes: Vec::new(),
        }
    }

    /// Fork: crear un proceso hijo
    pub fn fork(&mut self, parent_pid: u32) -> Result<ForkResult, String> {
        // En un sistema real, esto:
        // 1. Crearía un nuevo PCB para el hijo
        // 2. Clonaría el espacio de direcciones del padre
        // 3. Copiaría los descriptores de archivo
        // 4. Configuraría el estado del hijo
        // 5. Retornaría el resultado apropiado

        // Para este ejemplo, simulamos el fork
        let child_pid = parent_pid + 1000; // PID simulado
        
        Ok(ForkResult::parent(child_pid))
    }

    /// Exec: reemplazar el proceso actual
    pub fn exec(&mut self, args: ExecArgs) -> Result<(), String> {
        // En un sistema real, esto:
        // 1. Cargaría el nuevo programa desde el filesystem
        // 2. Limpiaría el espacio de direcciones actual
        // 3. Configuraría un nuevo espacio de direcciones
        // 4. Cargaría el código y datos del programa
        // 5. Configuraría el stack y heap
        // 6. Iniciaría la ejecución del nuevo programa

        // Para este ejemplo, solo validamos los argumentos
        if args.program.is_empty() {
            return Err(String::from("Program name cannot be empty"));
        }

        Ok(())
    }

    /// Wait: esperar a que un proceso hijo termine
    pub fn wait(&mut self, pid: Option<u32>) -> Result<WaitStatus, String> {
        // En un sistema real, esto:
        // 1. Buscaría un proceso hijo zombie
        // 2. Si pid es Some, buscaría ese proceso específico
        // 3. Si pid es None, buscaría cualquier proceso hijo
        // 4. Retornaría el estado de salida del proceso
        // 5. Limpiaría el PCB del proceso hijo

        // Para este ejemplo, simulamos la espera
        if let Some(target_pid) = pid {
            if self.zombie_processes.contains(&target_pid) {
                self.zombie_processes.retain(|&p| p != target_pid);
                return Ok(WaitStatus::Exited(0));
            }
        } else if !self.zombie_processes.is_empty() {
            let zombie_pid = self.zombie_processes.remove(0);
            let _ = zombie_pid;
            return Ok(WaitStatus::Exited(0));
        }

        Err(String::from("No child process to wait for"))
    }

    /// Waitpid: esperar a un proceso específico
    pub fn waitpid(&mut self, pid: u32, options: u32) -> Result<WaitStatus, String> {
        // En un sistema real, esto implementaría waitpid con opciones
        // como WNOHANG, WUNTRACED, etc.

        // Para este ejemplo, ignoramos las opciones
        self.wait(Some(pid))
    }

    /// Exit: terminar el proceso actual
    pub fn exit(&mut self, exit_code: i32) {
        // En un sistema real, esto:
        // 1. Cerraría todos los descriptores de archivo
        // 2. Liberaría el espacio de direcciones
        // 3. Notificaría al proceso padre
        // 4. Cambiaría el estado a Zombie
        // 5. El scheduler no volvería a ejecutar este proceso

        // Para este ejemplo, solo registramos la salida
        let _ = exit_code;
    }

    /// Agregar proceso a la lista de zombies
    pub fn add_zombie(&mut self, pid: u32) {
        self.zombie_processes.push(pid);
    }

    /// Verificar si hay procesos zombies
    pub fn has_zombies(&self) -> bool {
        !self.zombie_processes.is_empty()
    }

    /// Obtener número de procesos zombies
    pub fn zombie_count(&self) -> usize {
        self.zombie_processes.len()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Fork/Exec Manager Status\n");
        report.push_str("========================\n\n");
        
        report.push_str(&format!("Zombie Processes: {}\n", self.zombie_count()));
        if !self.zombie_processes.is_empty() {
            report.push_str("Zombie PIDs: ");
            for (i, pid) in self.zombie_processes.iter().enumerate() {
                if i > 0 {
                    report.push_str(", ");
                }
                report.push_str(&format!("{}", pid));
            }
            report.push_str("\n");
        }
        
        report
    }
}

impl Default for ForkExecManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades para fork/exec
pub struct ForkExecUtils;

impl ForkExecUtils {
    /// Verificar si un PID es válido
    pub fn is_valid_pid(pid: u32) -> bool {
        pid > 0 && pid <= 32767 // Rango típico de PIDs en Unix
    }

    /// Verificar si un código de salida es válido
    pub fn is_valid_exit_code(code: i32) -> bool {
        code >= 0 && code <= 255
    }

    /// Crear argumentos de exec desde un array de strings
    pub fn create_exec_args(program: String, args: &[String]) -> ExecArgs {
        let mut exec_args = ExecArgs::new(program);
        for arg in args {
            exec_args.add_arg(arg.clone());
        }
        exec_args
    }

    /// Parsear variables de entorno
    pub fn parse_env(env_str: &str) -> Vec<String> {
        env_str.split('\0')
            .filter(|s| !s.is_empty())
            .map(|s| String::from(s))
            .collect()
    }
}
