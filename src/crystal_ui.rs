//! Crystal UI - Interfaz Gráfica Adaptada para CRONOS W-OS
//!
//! Este módulo implementa componentes de UI adaptados a la arquitectura
//! de exokernel con grafos y sistema de capabilities

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::desktop::{DesktopEnvironment, Taskbar, StartMenu};

/// Tipo de aplicación
#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationType {
    /// Navegador web
    WebBrowser,
    /// Gestor de archivos
    FileManager,
    /// Terminal
    Terminal,
    /// Configuración
    Settings,
    /// IA Colmena
    ColmenaChat,
    /// Aplicación Virtualizada (Modo Fluido)
    VirtualApp(String),
}

/// Estado de carga
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    /// No cargando
    NotLoading,
    /// Cargando
    Loading,
    /// Cargado
    Loaded,
    /// Error
    Error(String),
}

/// Tipo de elemento de archivo
#[derive(Debug, Clone, PartialEq)]
pub enum FileItemType {
    /// Directorio
    Directory,
    /// Archivo
    File,
    /// Enlace
    Symlink,
}

/// Tipo de vista de archivos
#[derive(Debug, Clone, PartialEq)]
pub enum FileViewType {
    /// Iconos
    Icons,
    /// Lista
    List,
    /// Detalles
    Details,
}

/// Elemento de archivo
#[derive(Debug, Clone)]
pub struct FileItem {
    /// Nombre
    pub name: String,
    /// Tipo
    pub item_type: FileItemType,
    /// Tamaño
    pub size: u64,
    /// Fecha de modificación
    pub modified_time: String,
    /// Permisos
    pub permissions: String,
}

impl FileItem {
    pub fn new(name: String, item_type: FileItemType, size: u64) -> Self {
        Self {
            name,
            item_type,
            size,
            modified_time: String::from("2024-01-01"),
            permissions: String::from("rw-r--r--"),
        }
    }
}

/// Contenido del navegador web
#[derive(Debug, Clone)]
pub struct WebBrowserContent {
    /// URL actual
    pub current_url: String,
    /// Historial
    pub history: Vec<String>,
    /// Estado de carga
    pub loading_state: LoadingState,
}

impl WebBrowserContent {
    pub fn new() -> Self {
        Self {
            current_url: String::from("https://www.google.com"),
            history: Vec::new(),
            loading_state: LoadingState::Loaded,
        }
    }

    pub fn navigate_to(&mut self, url: &str) {
        self.history.push(self.current_url.clone());
        self.current_url = String::from(url);
        self.loading_state = LoadingState::Loading;
        // En un sistema real, aquí se cargaría la página
        self.loading_state = LoadingState::Loaded;
    }
}

/// Contenido del gestor de archivos
#[derive(Debug, Clone)]
pub struct FileManagerContent {
    /// Directorio actual
    pub current_directory: String,
    /// Archivos y directorios
    pub items: Vec<FileItem>,
    /// Vista actual
    pub view_type: FileViewType,
    /// Elemento seleccionado
    pub selected_item: Option<u32>,
}

impl FileManagerContent {
    pub fn new() -> Self {
        Self {
            current_directory: String::from("/"),
            items: vec![
                FileItem::new(String::from("Documentos"), FileItemType::Directory, 4096),
                FileItem::new(String::from("Descargas"), FileItemType::Directory, 4096),
                FileItem::new(String::from("Imágenes"), FileItemType::Directory, 4096),
                FileItem::new(String::from("README.txt"), FileItemType::File, 1024),
                FileItem::new(String::from("config.json"), FileItemType::File, 512),
            ],
            view_type: FileViewType::Details,
            selected_item: None,
        }
    }

    pub fn navigate_to(&mut self, path: &str) {
        if path == ".." {
            self.current_directory = String::from("/");
        } else {
            self.current_directory = format!("{}/{}", self.current_directory, path);
        }
    }
}

/// Contenido del terminal
#[derive(Debug, Clone)]
pub struct TerminalContent {
    /// Líneas de salida
    pub lines: Vec<String>,
    /// Línea actual de entrada
    pub current_line: String,
    /// Cursor
    pub cursor_position: u16,
    /// Historial de comandos
    pub command_history: Vec<String>,
}

impl TerminalContent {
    pub fn new() -> Self {
        Self {
            lines: vec![
                String::from("🚀 CRONOS W-OS Terminal v2.0.0"),
                String::from("📍 Usuario: cronos"),
                String::from("🔧 Sistema: Exokernel con Grafos"),
                String::from("💻 "),
            ],
            current_line: String::new(),
            cursor_position: 0,
            command_history: Vec::new(),
        }
    }

    pub fn execute_command(&mut self, command: &str) -> String {
        self.command_history.push(String::from(command));
        
        let output = match command {
            "ls" => String::from("Documentos  Descargas  Imágenes  README.txt  config.json"),
            "pwd" => String::from("/"),
            "help" => String::from("Comandos disponibles: ls, pwd, help, sysinfo"),
            "sysinfo" => String::from("CRONOS W-OS v2.0.0 - Exokernel con Grafos"),
            _ => format!("Comando no reconocido: {}", command),
        };
        
        self.lines.push(format!("💻 $ {}", command));
        self.lines.push(output.clone());
        self.lines.push(String::from("💻 "));
        
        output
    }
}

/// Crystal UI - Interfaz gráfica extendida
pub struct CrystalUI {
    /// Entorno de escritorio base
    pub desktop: DesktopEnvironment,
    /// Aplicación activa
    pub active_application: Option<ApplicationType>,
    /// Contenido del navegador web
    pub web_browser: Option<WebBrowserContent>,
    /// Contenido del gestor de archivos
    pub file_manager: Option<FileManagerContent>,
    /// Contenido del terminal
    pub terminal: Option<TerminalContent>,
}

impl CrystalUI {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            desktop: DesktopEnvironment::new(screen_width, screen_height),
            active_application: None,
            web_browser: None,
            file_manager: None,
            terminal: None,
        }
    }

    /// Abre una aplicación
    pub fn open_application(&mut self, app_type: ApplicationType) {
        self.active_application = Some(app_type.clone());
        
        // Registrar aplicación en la barra de tareas
        let app_name = match app_type {
            ApplicationType::WebBrowser => "Navegador Web",
            ApplicationType::FileManager => "Gestor de Archivos",
            ApplicationType::Terminal => "Terminal",
            ApplicationType::Settings => "Configuración",
            ApplicationType::ColmenaChat => "Chat IA Colmena",
            ApplicationType::VirtualApp(ref name) => name,
        };
        self.desktop.taskbar_mut().add_item(0, app_name);

        match app_type {
            ApplicationType::WebBrowser => {
                self.web_browser = Some(WebBrowserContent::new());
            }
            ApplicationType::FileManager => {
                self.file_manager = Some(FileManagerContent::new());
            }
            ApplicationType::Terminal => {
                self.terminal = Some(TerminalContent::new());
            }
            ApplicationType::Settings => {
                // Configuración pendiente de implementación
            }
            ApplicationType::ColmenaChat => {
                // Chat con IA Colmena pendiente de implementación
            }
            ApplicationType::VirtualApp(_) => {
                // Aplicación virtualizada ya gestionada por el compositor
            }
        }
    }

    /// Cierra la aplicación activa
    pub fn close_application(&mut self) {
        self.active_application = None;
        self.web_browser = None;
        self.file_manager = None;
        self.terminal = None;
    }

    /// Navega a una URL en el navegador
    pub fn navigate_to(&mut self, url: &str) {
        if let Some(ref mut browser) = self.web_browser {
            browser.navigate_to(url);
        }
    }

    /// Ejecuta un comando en el terminal
    pub fn execute_command(&mut self, command: &str) -> String {
        if let Some(ref mut terminal) = self.terminal {
            terminal.execute_command(command)
        } else {
            String::from("Terminal no está abierto")
        }
    }

    /// Navega a un directorio en el gestor de archivos
    pub fn navigate_directory(&mut self, path: &str) {
        if let Some(ref mut fm) = self.file_manager {
            fm.navigate_to(path);
        }
    }

    /// Obtiene el escritorio
    pub fn desktop(&self) -> &DesktopEnvironment {
        &self.desktop
    }

    /// Obtiene el escritorio mutable
    pub fn desktop_mut(&mut self) -> &mut DesktopEnvironment {
        &mut self.desktop
    }
}

impl Default for CrystalUI {
    fn default() -> Self {
        Self::new(1024, 768)
    }
}

/// Errores de Crystal UI
#[derive(Debug, Clone, PartialEq)]
pub enum CrystalUIError {
    /// Aplicación no encontrada
    ApplicationNotFound,
    /// Operación no válida
    InvalidOperation,
    /// Recurso no disponible
    ResourceUnavailable,
}

impl fmt::Display for CrystalUIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrystalUIError::ApplicationNotFound => write!(f, "Application not found"),
            CrystalUIError::InvalidOperation => write!(f, "Invalid operation"),
            CrystalUIError::ResourceUnavailable => write!(f, "Resource unavailable"),
        }
    }
}
