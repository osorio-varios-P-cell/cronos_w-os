//! POSIX-like Shell Module
//! 
//! This module implements a POSIX-like shell for user space.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de comando
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Comando builtin
    Builtin,
    /// Comando externo
    External,
    /// Comando de tubería
    Pipeline,
    /// Comando de fondo
    Background,
}

/// Comando
#[derive(Debug, Clone)]
pub struct Command {
    /// Nombre del comando
    pub name: String,
    /// Argumentos
    pub args: Vec<String>,
    /// Tipo de comando
    pub command_type: CommandType,
    /// Entrada redirigida
    pub input_redirect: Option<String>,
    /// Salida redirigida
    pub output_redirect: Option<String>,
    /// Error redirigido
    pub error_redirect: Option<String>,
}

impl Command {
    /// Crear nuevo comando
    pub fn new(name: String, args: Vec<String>) -> Self {
        Self {
            name,
            args,
            command_type: CommandType::External,
            input_redirect: None,
            output_redirect: None,
            error_redirect: None,
        }
    }

    /// Crear comando builtin
    pub fn builtin(name: String, args: Vec<String>) -> Self {
        Self {
            name,
            args,
            command_type: CommandType::Builtin,
            input_redirect: None,
            output_redirect: None,
            error_redirect: None,
        }
    }
}

/// Estado del shell
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellState {
    /// Ejecutando
    Running,
    /// Saliendo
    Exiting,
    /// Error
    Error,
}

/// Historial de comandos
#[derive(Debug, Clone)]
pub struct CommandHistory {
    /// Comandos
    pub commands: Vec<String>,
    /// Índice actual
    pub current_index: usize,
    /// Tamaño máximo
    pub max_size: usize,
}

impl CommandHistory {
    /// Crear nuevo historial
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
            max_size,
        }
    }

    /// Agregar comando
    pub fn add(&mut self, command: String) {
        if self.commands.len() >= self.max_size {
            self.commands.remove(0);
        }
        self.commands.push(command);
        self.current_index = self.commands.len();
    }

    /// Obtener comando anterior
    pub fn previous(&mut self) -> Option<&String> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.commands.get(self.current_index)
        } else {
            None
        }
    }

    /// Obtener comando siguiente
    pub fn next(&mut self) -> Option<&String> {
        if self.current_index < self.commands.len() - 1 {
            self.current_index += 1;
            self.commands.get(self.current_index)
        } else {
            None
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Variables de entorno
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variables
    pub variables: Vec<(String, String)>,
}

impl Environment {
    /// Crear nuevo entorno
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    /// Establecer variable
    pub fn set(&mut self, name: String, value: String) {
        // Buscar si ya existe
        for var in &mut self.variables {
            if var.0 == name {
                var.1 = value;
                return;
            }
        }
        
        // Si no existe, agregar
        self.variables.push((name, value));
    }

    /// Obtener variable
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.iter().find(|v| v.0 == name).map(|v| &v.1)
    }

    /// Eliminar variable
    pub fn unset(&mut self, name: &str) {
        self.variables.retain(|v| v.0 != name);
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Shell POSIX
pub struct PosixShell {
    /// Estado
    pub state: ShellState,
    /// Directorio de trabajo
    pub working_directory: String,
    /// Usuario
    pub user: String,
    /// Hostname
    pub hostname: String,
    /// Historial
    pub history: CommandHistory,
    /// Entorno
    pub environment: Environment,
    /// Prompt
    pub prompt: String,
}

impl PosixShell {
    /// Crear nuevo shell
    pub fn new() -> Self {
        let mut shell = Self {
            state: ShellState::Running,
            working_directory: String::from("/"),
            user: String::from("root"),
            hostname: String::from("cronos"),
            history: CommandHistory::default(),
            environment: Environment::default(),
            prompt: String::from("\\u@\\h:\\w\\$ "),
        };
        
        // Establecer variables de entorno por defecto
        shell.environment.set(String::from("PATH"), String::from("/bin:/usr/bin"));
        shell.environment.set(String::from("HOME"), String::from("/root"));
        shell.environment.set(String::from("SHELL"), String::from("/bin/sh"));
        
        shell
    }

    /// Ejecutar comando
    pub fn execute(&mut self, command: Command) -> Result<i32, String> {
        // Agregar al historial
        let cmd_str = format!("{} {}", command.name, command.args.join(" "));
        self.history.add(cmd_str);

        match command.name.as_str() {
            "cd" => self.builtin_cd(&command.args),
            "pwd" => self.builtin_pwd(),
            "echo" => self.builtin_echo(&command.args),
            "export" => self.builtin_export(&command.args),
            "unset" => self.builtin_unset(&command.args),
            "env" => self.builtin_env(),
            "exit" => self.builtin_exit(&command.args),
            "history" => self.builtin_history(),
            "clear" => self.builtin_clear(),
            _ => self.execute_external(command),
        }
    }

    /// Builtin cd
    fn builtin_cd(&mut self, args: &[String]) -> Result<i32, String> {
        let path = if args.is_empty() {
            self.environment.get("HOME").map(|s| s.clone()).unwrap_or_else(|| String::from("/"))
        } else {
            args[0].clone()
        };
        
        // En un sistema real, esto cambiaría el directorio
        self.working_directory = path;
        Ok(0)
    }

    /// Builtin pwd
    fn builtin_pwd(&self) -> Result<i32, String> {
        // En un sistema real, esto imprimiría el directorio actual
        Ok(0)
    }

    /// Builtin echo
    fn builtin_echo(&self, args: &[String]) -> Result<i32, String> {
        // En un sistema real, esto imprimiría los argumentos
        let _ = args;
        Ok(0)
    }

    /// Builtin export
    fn builtin_export(&mut self, args: &[String]) -> Result<i32, String> {
        if args.is_empty() {
            return Ok(0);
        }
        
        for arg in args {
            if let Some(pos) = arg.find('=') {
                let name = String::from(&arg[..pos]);
                let value = String::from(&arg[pos + 1..]);
                self.environment.set(name, value);
            }
        }
        
        Ok(0)
    }

    /// Builtin unset
    fn builtin_unset(&mut self, args: &[String]) -> Result<i32, String> {
        for arg in args {
            self.environment.unset(arg);
        }
        Ok(0)
    }

    /// Builtin env
    fn builtin_env(&self) -> Result<i32, String> {
        // En un sistema real, esto imprimiría las variables de entorno
        Ok(0)
    }

    /// Builtin exit
    fn builtin_exit(&self, args: &[String]) -> Result<i32, String> {
        let code = if args.is_empty() {
            0
        } else {
            args[0].parse().unwrap_or(0)
        };
        
        // En un sistema real, esto terminaría el proceso
        // self.state = ShellState::Exiting;
        let _ = code;
        Ok(0)
    }

    /// Builtin history
    fn builtin_history(&self) -> Result<i32, String> {
        // En un sistema real, esto imprimiría el historial
        Ok(0)
    }

    /// Builtin clear
    fn builtin_clear(&self) -> Result<i32, String> {
        // En un sistema real, esto limpiaría la pantalla
        Ok(0)
    }

    /// Ejecutar comando externo
    fn execute_external(&self, command: Command) -> Result<i32, String> {
        // En un sistema real, esto ejecutaría un comando externo
        let _ = command;
        Err(String::from("Command not found"))
    }

    /// Parsear línea de comando
    pub fn parse_line(&self, line: &str) -> Result<Command, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err(String::from("Empty command"));
        }
        
        let name = String::from(parts[0]);
        let args: Vec<String> = parts[1..].iter().map(|s| String::from(*s)).collect();
        
        // Determinar si es builtin
        let command_type = match name.as_str() {
            "cd" | "pwd" | "echo" | "export" | "unset" | "env" | "exit" | "history" | "clear" => CommandType::Builtin,
            _ => CommandType::External,
        };
        
        let mut command = if command_type == CommandType::Builtin {
            Command::builtin(name, args)
        } else {
            Command::new(name, args)
        };
        
        command.command_type = command_type;
        
        Ok(command)
    }

    /// Generar prompt
    pub fn generate_prompt(&self) -> String {
        let mut prompt = self.prompt.clone();
        
        prompt = prompt.replace("\\u", &self.user);
        prompt = prompt.replace("\\h", &self.hostname);
        prompt = prompt.replace("\\w", &self.working_directory);
        
        if prompt.contains("$") {
            if self.user == "root" {
                prompt = prompt.replace("$", "#");
            }
        }
        
        prompt
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("POSIX Shell Status\n");
        report.push_str("=================\n\n");
        
        report.push_str(&format!("State: {:?}\n", self.state));
        report.push_str(&format!("User: {}\n", self.user));
        report.push_str(&format!("Hostname: {}\n", self.hostname));
        report.push_str(&format!("Working Directory: {}\n", self.working_directory));
        report.push_str(&format!("History Size: {}\n", self.history.commands.len()));
        report.push_str(&format!("Environment Variables: {}\n", self.environment.variables.len()));
        
        report
    }
}

impl Default for PosixShell {
    fn default() -> Self {
        Self::new()
    }
}
