//! Tool System Module
//! 
//! This module implements a tool system for AI agents based on Microsoft Agent Framework.
//! Based on Microsoft AI Agents for Beginners course - Lesson 4: Tool Use.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Tipo de herramienta
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    /// Herramienta de sistema
    System,
    /// Herramienta de archivo
    File,
    /// Herramienta de red
    Network,
    /// Herramienta de base de datos
    Database,
    /// Herramienta de API
    Api,
    /// Herramienta personalizada
    Custom,
}

/// Parámetro de herramienta
#[derive(Debug, Clone)]
pub struct ToolParameter {
    /// Nombre del parámetro
    pub name: String,
    /// Tipo de dato
    pub data_type: String,
    /// Requerido
    pub required: bool,
    /// Descripción
    pub description: String,
}

impl ToolParameter {
    /// Crear nuevo parámetro
    pub fn new(name: String, data_type: String, required: bool, description: String) -> Self {
        Self {
            name,
            data_type,
            required,
            description,
        }
    }
}

/// Herramienta del sistema
#[derive(Debug, Clone)]
pub struct Tool {
    /// Nombre de la herramienta
    pub name: String,
    /// Tipo de herramienta
    pub tool_type: ToolType,
    /// Descripción
    pub description: String,
    /// Parámetros
    pub parameters: Vec<ToolParameter>,
    /// Habilitada
    pub enabled: bool,
    /// Versión
    pub version: String,
}

impl Tool {
    /// Crear nueva herramienta
    pub fn new(name: String, tool_type: ToolType, description: String, version: String) -> Self {
        Self {
            name,
            tool_type,
            description,
            parameters: Vec::new(),
            enabled: true,
            version,
        }
    }

    /// Agregar parámetro
    pub fn add_parameter(&mut self, parameter: ToolParameter) {
        self.parameters.push(parameter);
    }

    /// Validar parámetros
    pub fn validate_parameters(&self, args: &[String]) -> Result<(), String> {
        let required_params: Vec<&ToolParameter> = self.parameters.iter()
            .filter(|p| p.required)
            .collect();
        
        if required_params.len() > args.len() {
            return Err(format!("Missing required parameters. Expected: {}, Got: {}", 
                required_params.len(), args.len()));
        }
        
        Ok(())
    }

    /// Ejecutar herramienta
    pub fn execute(&self, args: &[String]) -> Result<String, String> {
        if !self.enabled {
            return Err(String::from("Tool is disabled"));
        }
        
        self.validate_parameters(args)?;
        
        // En un sistema real, esto ejecutaría la herramienta
        Ok(format!("Executed tool: {} with args: {:?}", self.name, args))
    }
}

/// Resultado de ejecución de herramienta
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    /// Nombre de la herramienta
    pub tool_name: String,
    /// Éxito
    pub success: bool,
    /// Resultado
    pub result: Option<String>,
    /// Error
    pub error: Option<String>,
    /// Tiempo de ejecución (ms)
    pub execution_time_ms: u64,
}

impl ToolExecutionResult {
    /// Crear resultado exitoso
    pub fn success(tool_name: String, result: String, execution_time_ms: u64) -> Self {
        Self {
            tool_name,
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms,
        }
    }

    /// Crear resultado fallido
    pub fn failure(tool_name: String, error: String, execution_time_ms: u64) -> Self {
        Self {
            tool_name,
            success: false,
            result: None,
            error: Some(error),
            execution_time_ms,
        }
    }
}

/// Sistema de herramientas
pub struct ToolSystem {
    /// Herramientas disponibles
    pub tools: Vec<Tool>,
    /// Historial de ejecuciones
    pub execution_history: Vec<ToolExecutionResult>,
}

impl ToolSystem {
    /// Crear nuevo sistema de herramientas
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            execution_history: Vec::new(),
        }
    }

    /// Registrar herramienta
    pub fn register_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// Obtener herramienta por nombre
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.name == name)
    }

    /// Obtener herramienta mutable por nombre
    pub fn get_tool_mut(&mut self, name: &str) -> Option<&mut Tool> {
        self.tools.iter_mut().find(|t| t.name == name)
    }

    /// Habilitar herramienta
    pub fn enable_tool(&mut self, name: &str) -> Result<(), String> {
        let tool = self.get_tool_mut(name)
            .ok_or_else(|| String::from("Tool not found"))?;
        
        tool.enabled = true;
        Ok(())
    }

    /// Deshabilitar herramienta
    pub fn disable_tool(&mut self, name: &str) -> Result<(), String> {
        let tool = self.get_tool_mut(name)
            .ok_or_else(|| String::from("Tool not found"))?;
        
        tool.enabled = false;
        Ok(())
    }

    /// Ejecutar herramienta
    pub fn execute_tool(&mut self, name: &str, args: &[String]) -> Result<ToolExecutionResult, String> {
        let tool = self.get_tool(name)
            .ok_or_else(|| String::from("Tool not found"))?
            .clone();
        
        let start_time = 0; // En un sistema real, esto sería el tiempo actual
        
        match tool.execute(args) {
            Ok(result) => {
                let execution_time = 10; // Simulado
                let exec_result = ToolExecutionResult::success(String::from(name), result, execution_time);
                self.execution_history.push(exec_result.clone());
                Ok(exec_result)
            }
            Err(error) => {
                let execution_time = 10; // Simulado
                let exec_result = ToolExecutionResult::failure(String::from(name), error, execution_time);
                self.execution_history.push(exec_result.clone());
                Ok(exec_result)
            }
        }
    }

    /// Obtener herramientas por tipo
    pub fn get_tools_by_type(&self, tool_type: ToolType) -> Vec<&Tool> {
        self.tools.iter().filter(|t| t.tool_type == tool_type).collect()
    }

    /// Obtener historial de ejecuciones
    pub fn get_execution_history(&self) -> &[ToolExecutionResult] {
        &self.execution_history
    }

    /// Limpiar historial de ejecuciones
    pub fn clear_history(&mut self) {
        self.execution_history.clear();
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Tool System Status\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("Total Tools: {}\n", self.tools.len()));
        report.push_str(&format!("Enabled: {}\n", self.tools.iter().filter(|t| t.enabled).count()));
        report.push_str(&format!("Disabled: {}\n", self.tools.iter().filter(|t| !t.enabled).count()));
        report.push_str(&format!("Execution History: {}\n\n", self.execution_history.len()));
        
        for tool in &self.tools {
            report.push_str(&format!("Tool: {}\n", tool.name));
            report.push_str(&format!("  Type: {:?}\n", tool.tool_type));
            report.push_str(&format!("  Version: {}\n", tool.version));
            report.push_str(&format!("  Enabled: {}\n", tool.enabled));
            report.push_str(&format!("  Parameters: {}\n", tool.parameters.len()));
            report.push('\n');
        }
        
        report
    }
}

impl Default for ToolSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de herramientas
pub struct ToolUtils;

impl ToolUtils {
    /// Crear sistema de herramientas por defecto
    pub fn create_default_tool_system() -> ToolSystem {
        let mut system = ToolSystem::new();
        
        // Herramienta de lectura de archivos
        let mut file_read = Tool::new(
            String::from("file_read"),
            ToolType::File,
            String::from("Read file contents"),
            String::from("1.0.0"),
        );
        file_read.add_parameter(ToolParameter::new(
            String::from("path"),
            String::from("string"),
            true,
            String::from("File path to read"),
        ));
        system.register_tool(file_read);
        
        // Herramienta de escritura de archivos
        let mut file_write = Tool::new(
            String::from("file_write"),
            ToolType::File,
            String::from("Write content to file"),
            String::from("1.0.0"),
        );
        file_write.add_parameter(ToolParameter::new(
            String::from("path"),
            String::from("string"),
            true,
            String::from("File path to write"),
        ));
        file_write.add_parameter(ToolParameter::new(
            String::from("content"),
            String::from("string"),
            true,
            String::from("Content to write"),
        ));
        system.register_tool(file_write);
        
        // Herramienta de ejecución de comandos
        let mut system_command = Tool::new(
            String::from("system_command"),
            ToolType::System,
            String::from("Execute system command"),
            String::from("1.0.0"),
        );
        system_command.add_parameter(ToolParameter::new(
            String::from("command"),
            String::from("string"),
            true,
            String::from("Command to execute"),
        ));
        system.register_tool(system_command);
        
        // Herramienta de consulta de red
        let mut network_query = Tool::new(
            String::from("network_query"),
            ToolType::Network,
            String::from("Query network information"),
            String::from("1.0.0"),
        );
        network_query.add_parameter(ToolParameter::new(
            String::from("query"),
            String::from("string"),
            true,
            String::from("Network query to execute"),
        ));
        system.register_tool(network_query);
        
        // Herramienta de búsqueda en base de datos
        let mut database_search = Tool::new(
            String::from("database_search"),
            ToolType::Database,
            String::from("Search database"),
            String::from("1.0.0"),
        );
        database_search.add_parameter(ToolParameter::new(
            String::from("query"),
            String::from("string"),
            true,
            String::from("Search query"),
        ));
        system.register_tool(database_search);
        
        system
    }
}
