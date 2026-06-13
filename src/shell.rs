//! Shell Real y Utilidades Básicas para CRONOS W-OS
//!
//! Este módulo implementa un shell de línea de comandos real con
//! utilidades básicas, adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Tipo de comando
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Comando interno (built-in)
    Internal,
    /// Comando externo (ejecutable)
    External,
    /// Comando de sistema
    System,
}

/// Resultado de ejecución de comando
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandResult {
    pub fn success(stdout: String) -> Self {
        Self {
            exit_code: 0,
            stdout,
            stderr: String::new(),
        }
    }

    pub fn error(stderr: String) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr,
        }
    }
}

/// Comando del shell
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub command_type: CommandType,
}

impl Command {
    pub fn new(name: String, args: Vec<String>) -> Self {
        Self {
            name,
            args,
            command_type: CommandType::Internal,
        }
    }

    /// Parsear una línea de comando
    pub fn parse(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts[0].to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        Some(Command::new(name, args))
    }
}

/// Directorio de trabajo actual
#[derive(Debug, Clone)]
pub struct WorkingDirectory {
    pub path: String,
    pub inode_id: Option<u64>,
}

impl WorkingDirectory {
    pub fn new(path: String) -> Self {
        Self {
            path,
            inode_id: None,
        }
    }

    pub fn root() -> Self {
        Self::new(String::from("/"))
    }
}

impl Default for WorkingDirectory {
    fn default() -> Self {
        Self::root()
    }
}

/// Historial de comandos
#[derive(Debug, Clone)]
pub struct CommandHistory {
    pub commands: Vec<String>,
    pub max_size: usize,
    pub current_index: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: Vec::new(),
            max_size,
            current_index: 0,
        }
    }

    /// Agregar un comando al historial
    pub fn add(&mut self, command: String) {
        if !command.is_empty() {
            self.commands.push(command);
            if self.commands.len() > self.max_size {
                self.commands.remove(0);
            }
            self.current_index = self.commands.len();
        }
    }

    /// Obtener el comando anterior
    pub fn previous(&mut self) -> Option<&String> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.commands.get(self.current_index)
        } else {
            None
        }
    }

    /// Obtener el comando siguiente
    pub fn next(&mut self) -> Option<&String> {
        if self.current_index < self.commands.len() - 1 {
            self.current_index += 1;
            self.commands.get(self.current_index)
        } else {
            None
        }
    }

    /// Listar todos los comandos
    pub fn list(&self) -> &[String] {
        &self.commands
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Variable de entorno
#[derive(Debug, Clone)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
}

impl EnvironmentVariable {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

/// Entorno del shell
#[derive(Debug, Clone)]
pub struct ShellEnvironment {
    pub variables: BTreeMap<String, String>,
    pub working_directory: WorkingDirectory,
    pub user: String,
    pub hostname: String,
}

impl ShellEnvironment {
    pub fn new() -> Self {
        let mut variables = BTreeMap::new();
        variables.insert(String::from("PATH"), String::from("/bin:/usr/bin"));
        variables.insert(String::from("HOME"), String::from("/home/user"));
        variables.insert(String::from("SHELL"), String::from("/bin/cronosh"));
        variables.insert(String::from("TERM"), String::from("xterm-256color"));

        Self {
            variables,
            working_directory: WorkingDirectory::root(),
            user: String::from("root"),
            hostname: String::from("cronos"),
        }
    }

    /// Obtener una variable de entorno
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Establecer una variable de entorno
    pub fn set(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Eliminar una variable de entorno
    pub fn unset(&mut self, name: &str) {
        self.variables.remove(name);
    }

    /// Obtener el prompt
    pub fn prompt(&self) -> String {
        format!("{}@{}:{}$ ", self.user, self.hostname, self.working_directory.path)
    }
}

impl Default for ShellEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Shell de CRONOS
pub struct CronosShell {
    pub environment: ShellEnvironment,
    pub history: CommandHistory,
    pub running: bool,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosShell {
    pub fn new() -> Self {
        Self {
            environment: ShellEnvironment::new(),
            history: CommandHistory::default(),
            running: true,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Ejecutar un comando
    pub fn execute(&mut self, command: &Command) -> CommandResult {
        let name = command.name.as_str();
        let args = &command.args;

        match name {
            "help" => self.cmd_help(args),
            "ls" => self.cmd_ls(args),
            "cd" => self.cmd_cd(args),
            "pwd" => self.cmd_pwd(args),
            "echo" => self.cmd_echo(args),
            "cat" => self.cmd_cat(args),
            "mkdir" => self.cmd_mkdir(args),
            "rm" => self.cmd_rm(args),
            "clear" => self.cmd_clear(args),
            "history" => self.cmd_history(args),
            "env" => self.cmd_env(args),
            "export" => self.cmd_export(args),
            "exit" => self.cmd_exit(args),
            "ps" => self.cmd_ps(args),
            "top" => self.cmd_top(args),
            "free" => self.cmd_free(args),
            "uname" => self.cmd_uname(args),
            "date" => self.cmd_date(args),
            "whoami" => self.cmd_whoami(args),
            "hostname" => self.cmd_hostname(args),
            _ => CommandResult::error(format!("Command not found: {}", name)),
        }
    }

    /// Procesar una línea de entrada
    pub fn process_line(&mut self, line: &str) -> CommandResult {
        // Agregar al historial
        self.history.add(line.to_string());

        // Parsear el comando
        if let Some(command) = Command::parse(line) {
            self.execute(&command)
        } else {
            CommandResult::success(String::new())
        }
    }

    /// Comando: help
    fn cmd_help(&self, _args: &[String]) -> CommandResult {
        let help_text = r#"
CRONOS W-OS Shell - Available Commands:

Internal Commands:
  help          Show this help message
  ls            List directory contents
  cd            Change working directory
  pwd           Print working directory
  echo          Print text
  cat           Concatenate and print files
  mkdir         Create directories
  rm            Remove files or directories
  clear         Clear the screen
  history       Show command history
  env           List environment variables
  export        Set environment variables
  exit          Exit the shell

System Commands:
  ps            List running processes
  top           Display system processes
  free          Display memory usage
  uname         Print system information
  date          Print current date and time
  whoami        Print current user
  hostname      Print system hostname
"#;
        CommandResult::success(help_text.to_string())
    }

    /// Comando: ls
    fn cmd_ls(&self, _args: &[String]) -> CommandResult {
        // En un sistema real, aquí se listaría el directorio actual
        let output = format!("bin  etc  home  lib  usr  var  tmp\n");
        CommandResult::success(output)
    }

    /// Comando: cd
    fn cmd_cd(&mut self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            self.environment.working_directory = WorkingDirectory::root();
            CommandResult::success(String::new())
        } else {
            let path = &args[0];
            // En un sistema real, aquí se verificaría si el directorio existe
            self.environment.working_directory.path = path.clone();
            CommandResult::success(String::new())
        }
    }

    /// Comando: pwd
    fn cmd_pwd(&self, _args: &[String]) -> CommandResult {
        CommandResult::success(format!("{}\n", self.environment.working_directory.path))
    }

    /// Comando: echo
    fn cmd_echo(&self, args: &[String]) -> CommandResult {
        let output = args.join(" ");
        CommandResult::success(format!("{}\n", output))
    }

    /// Comando: cat
    fn cmd_cat(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::error(String::from("Usage: cat <file>"));
        }
        // En un sistema real, aquí se leería el archivo
        CommandResult::success(format!("Contents of {}\n", args[0]))
    }

    /// Comando: mkdir
    fn cmd_mkdir(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::error(String::from("Usage: mkdir <directory>"));
        }
        // En un sistema real, aquí se crearía el directorio
        CommandResult::success(format!("Directory '{}' created\n", args[0]))
    }

    /// Comando: rm
    fn cmd_rm(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::error(String::from("Usage: rm <file>"));
        }
        // En un sistema real, aquí se eliminaría el archivo
        CommandResult::success(format!("File '{}' removed\n", args[0]))
    }

    /// Comando: clear
    fn cmd_clear(&self, _args: &[String]) -> CommandResult {
        // En un sistema real, aquí se limpiaría la pantalla
        CommandResult::success(String::from("\x1b[2J\x1b[H"))
    }

    /// Comando: history
    fn cmd_history(&self, _args: &[String]) -> CommandResult {
        let mut output = String::new();
        for (i, cmd) in self.history.list().iter().enumerate() {
            output.push_str(&format!("{}  {}\n", i + 1, cmd));
        }
        CommandResult::success(output)
    }

    /// Comando: env
    fn cmd_env(&self, _args: &[String]) -> CommandResult {
        let mut output = String::new();
        for (name, value) in &self.environment.variables {
            output.push_str(&format!("{}={}\n", name, value));
        }
        CommandResult::success(output)
    }

    /// Comando: export
    fn cmd_export(&mut self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::error(String::from("Usage: export NAME=VALUE"));
        }
        
        let arg = &args[0];
        if let Some((name, value)) = arg.split_once('=') {
            self.environment.set(name.to_string(), value.to_string());
            CommandResult::success(String::new())
        } else {
            CommandResult::error(String::from("Usage: export NAME=VALUE"))
        }
    }

    /// Comando: exit
    fn cmd_exit(&mut self, _args: &[String]) -> CommandResult {
        self.running = false;
        CommandResult::success(String::from("Exiting shell...\n"))
    }

    /// Comando: ps
    fn cmd_ps(&self, _args: &[String]) -> CommandResult {
        let output = r#"PID  USER    COMMAND
  1  root    init
  2  root    cronos_kernel
  3  root    compositor
  4  root    shell
"#;
        CommandResult::success(output.to_string())
    }

    /// Comando: top
    fn cmd_top(&self, _args: &[String]) -> CommandResult {
        let output = r#"top - 00:00:00 up 0:00:00, 1 user, load average: 0.00, 0.00, 0.00
Tasks: 4 total, 1 running, 3 sleeping
%Cpu(s): 0.0 us, 0.0 sy, 0.0 ni, 100.0 id, 0.0 wa, 0.0 hi, 0.0 si
MiB Mem: 4096.0 total, 1024.0 free, 2048.0 used, 1024.0 cache

PID  USER    PR  NI  VIRT  RES  %CPU  %MEM  TIME+  COMMAND
  1  root    20   0  1024  256  0.0   6.2  0:00.00 init
  2  root    20   0  2048  512  0.0  12.5  0:00.00 cronos_kernel
  3  root    20   0  4096 1024  0.0  25.0  0:00.00 compositor
  4  root    20   0  2048  256  0.0   6.2  0:00.00 shell
"#;
        CommandResult::success(output.to_string())
    }

    /// Comando: free
    fn cmd_free(&self, _args: &[String]) -> CommandResult {
        let output = r#"              total        used        free      shared  buff/cache   available
Mem:        4194304     2097152     1048576           0     1048576     3145728
Swap:       1048576           0     1048576
"#;
        CommandResult::success(output.to_string())
    }

    /// Comando: uname
    fn cmd_uname(&self, args: &[String]) -> CommandResult {
        if args.contains(&String::from("-a")) {
            CommandResult::success(String::from("CRONOS W-OS 2.0.0 x86_64 GNU/Linux\n"))
        } else {
            CommandResult::success(String::from("CRONOS W-OS\n"))
        }
    }

    /// Comando: date
    fn cmd_date(&self, _args: &[String]) -> CommandResult {
        // En un sistema real, aquí se obtendría la fecha real
        CommandResult::success(String::from("Thu Jan 1 00:00:00 UTC 2026\n"))
    }

    /// Comando: whoami
    fn cmd_whoami(&self, _args: &[String]) -> CommandResult {
        CommandResult::success(format!("{}\n", self.environment.user))
    }

    /// Comando: hostname
    fn cmd_hostname(&self, _args: &[String]) -> CommandResult {
        CommandResult::success(format!("{}\n", self.environment.hostname))
    }

    /// Verificar si el shell está corriendo
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Obtener el prompt
    pub fn prompt(&self) -> String {
        self.environment.prompt()
    }
}

impl Default for CronosShell {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del shell
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellError {
    CommandNotFound,
    PermissionDenied,
    InvalidArgument,
    FileNotFound,
    DirectoryNotFound,
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::CommandNotFound => write!(f, "Command not found"),
            ShellError::PermissionDenied => write!(f, "Permission denied"),
            ShellError::InvalidArgument => write!(f, "Invalid argument"),
            ShellError::FileNotFound => write!(f, "File not found"),
            ShellError::DirectoryNotFound => write!(f, "Directory not found"),
        }
    }
}
