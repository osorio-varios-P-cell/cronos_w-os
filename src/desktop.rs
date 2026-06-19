//! Desktop Environment para CRONOS W-OS
//!
//! Este módulo implementa el entorno de escritorio básico
//! incluyendo la barra de tareas y el menú de inicio
//! Adaptado para trabajar con el sistema de capabilities

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};

/// Ítem de la barra de tareas
#[derive(Debug, Clone)]
pub struct TaskbarItem {
    pub window_id: u32,
    pub title: String,
    pub icon: Option<u32>, // ID del icono
    pub active: bool,
}

impl TaskbarItem {
    pub fn new(window_id: u32, title: &str) -> Self {
        Self {
            window_id,
            title: String::from(title),
            icon: None,
            active: false,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Barra de tareas
pub struct Taskbar {
    items: Vec<TaskbarItem>,
    height: u32,
    visible: bool,
    auto_hide: bool,
}

impl Taskbar {
    pub fn new(_screen_width: u32, _screen_height: u32) -> Self {
        let height = 40;
        
        Self {
            items: Vec::new(),
            height,
            visible: true,
            auto_hide: false,
        }
    }

    pub fn add_item(&mut self, window_id: u32, title: &str) {
        let item = TaskbarItem::new(window_id, title);
        self.items.push(item);
    }

    pub fn remove_item(&mut self, window_id: u32) -> bool {
        if let Some(pos) = self.items.iter().position(|item| item.window_id == window_id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_item(&self, window_id: u32) -> Option<&TaskbarItem> {
        self.items.iter().find(|item| item.window_id == window_id)
    }

    pub fn get_item_mut(&mut self, window_id: u32) -> Option<&mut TaskbarItem> {
        self.items.iter_mut().find(|item| item.window_id == window_id)
    }

    pub fn set_item_active(&mut self, window_id: u32, active: bool) -> bool {
        if let Some(item) = self.get_item_mut(window_id) {
            item.set_active(active);
            true
        } else {
            false
        }
    }

    pub fn items(&self) -> &[TaskbarItem] {
        &self.items
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_auto_hide(&self) -> bool {
        self.auto_hide
    }

    pub fn set_auto_hide(&mut self, auto_hide: bool) {
        self.auto_hide = auto_hide;
    }
}

impl Default for Taskbar {
    fn default() -> Self {
        Self::new(1024, 768)
    }
}

/// Menú de inicio
pub struct StartMenu {
    visible: bool,
    items: Vec<StartMenuItem>,
}

/// Ítem del menú de inicio
#[derive(Debug, Clone)]
pub struct StartMenuItem {
    pub id: u32,
    pub title: String,
    pub icon: Option<u32>,
    pub action: StartMenuAction,
}

/// Acción del menú de inicio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartMenuAction {
    LaunchApp(u32),
    OpenSettings,
    Shutdown,
    Restart,
    Logout,
    Separator,
}

impl StartMenu {
    pub fn new() -> Self {
        Self {
            visible: false,
            items: Vec::new(),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn add_item(&mut self, item: StartMenuItem) {
        self.items.push(item);
    }

    pub fn remove_item(&mut self, id: u32) -> bool {
        if let Some(pos) = self.items.iter().position(|item| item.id == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn items(&self) -> &[StartMenuItem] {
        &self.items
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl Default for StartMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// FASE 16: Tipo de Fondo de Escritorio
#[derive(Debug, Clone, PartialEq)]
pub enum WallpaperType {
    SolidColor(u32),
    Image(String),
    Video(String),
    AIGenerated(String),
}

/// Widget del Dashboard
#[derive(Debug, Clone)]
pub struct DashboardWidget {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub widget_type: WidgetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    SystemStats,
    Weather,
    AInsights,
    Calendar,
    Network,
}

/// Dashboard inteligente inspirado en macOS/Windows 11
pub struct SovereignDashboard {
    pub visible: bool,
    pub widgets: Vec<DashboardWidget>,
    pub blur_radius: u32,
}

impl SovereignDashboard {
    pub fn new() -> Self {
        Self {
            visible: false,
            widgets: Vec::new(),
            blur_radius: 20,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Entorno de escritorio
pub struct DesktopEnvironment {
    taskbar: Taskbar,
    start_menu: StartMenu,
    pub dashboard: SovereignDashboard,
    screen_width: u32,
    screen_height: u32,
    /// FASE 16: Controles de hardware (Brillo, Volumen)
    pub system_volume: u8,
    pub screen_brightness: u8,
    /// FASE 16: Gestión estética del fondo
    pub wallpaper: WallpaperType,
    /// FASE 35: Tema dinámico
    pub accent_color: u32,
}

impl DesktopEnvironment {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let taskbar = Taskbar::new(screen_width, screen_height);
        let start_menu = StartMenu::new();
        let dashboard = SovereignDashboard::new();
        
        Self {
            taskbar,
            start_menu,
            dashboard,
            screen_width,
            screen_height,
            system_volume: 75,
            screen_brightness: 80,
            wallpaper: WallpaperType::SolidColor(0xFF1E1E2E), // Gris Soberano por defecto
            accent_color: 0xFF3B82F6, // Azul Crystal por defecto
        }
    }

    /// Establecer un nuevo fondo de escritorio
    pub fn set_wallpaper(&mut self, wallpaper: WallpaperType) {
        self.wallpaper = wallpaper;
        // En un sistema real, esto redibujaría la capa 0 del compositor
    }

    pub fn taskbar(&self) -> &Taskbar {
        &self.taskbar
    }

    pub fn taskbar_mut(&mut self) -> &mut Taskbar {
        &mut self.taskbar
    }

    pub fn start_menu(&self) -> &StartMenu {
        &self.start_menu
    }

    pub fn start_menu_mut(&mut self) -> &mut StartMenu {
        &mut self.start_menu
    }

    pub fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }
}

impl Default for DesktopEnvironment {
    fn default() -> Self {
        Self::new(1024, 768)
    }
}

/// Eventos del escritorio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEvent {
    None,
    TaskbarItemClicked(u32),
    StartMenuToggled,
    StartMenuClosed,
    StartMenuItemClicked(u32, StartMenuAction),
}

/// Errores del entorno de escritorio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopError {
    ItemNotFound,
    InvalidPosition,
    InvalidSize,
}

impl fmt::Display for DesktopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesktopError::ItemNotFound => write!(f, "Item not found"),
            DesktopError::InvalidPosition => write!(f, "Invalid position"),
            DesktopError::InvalidSize => write!(f, "Invalid size"),
        }
    }
}
