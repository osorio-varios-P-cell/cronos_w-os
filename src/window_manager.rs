//! Window Manager para CRONOS W-OS
//!
//! Este módulo implementa el sistema de ventanas adaptado a la arquitectura
//! de exokernel con grafos y sistema de capabilities

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};

/// Estado de una ventana
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Hidden,
}

/// Tipo de ventana
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Normal,
    Dialog,
    Menu,
    Tooltip,
    Splash,
    /// FASE 16: HUD / Overlay de Hive AI (Pequeña ventana superpuesta)
    Overlay,
}

/// Atributos de ventana
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowAttributes {
    pub title_bar: bool,
    pub close_button: bool,
    pub minimize_button: bool,
    pub maximize_button: bool,
    pub resizable: bool,
    pub movable: bool,
    pub always_on_top: bool,
}

impl WindowAttributes {
    pub fn default_normal() -> Self {
        Self {
            title_bar: true,
            close_button: true,
            minimize_button: true,
            maximize_button: true,
            resizable: true,
            movable: true,
            always_on_top: false,
        }
    }

    pub fn default_dialog() -> Self {
        Self {
            title_bar: true,
            close_button: true,
            minimize_button: false,
            maximize_button: false,
            resizable: false,
            movable: false,
            always_on_top: true,
        }
    }

    pub fn default_menu() -> Self {
        Self {
            title_bar: false,
            close_button: false,
            minimize_button: false,
            maximize_button: false,
            resizable: false,
            movable: false,
            always_on_top: true,
        }
    }
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self::default_normal()
    }
}

/// Rectángulo simple para posiciones y tamaños
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }
}

/// Punto simple para coordenadas
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Ventana representada como nodo en el grafo de recursos
#[derive(Debug, Clone)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub rect: Rect,
    pub state: WindowState,
    pub window_type: WindowType,
    pub attributes: WindowAttributes,
    pub z_order: u32,
    pub visible: bool,
    /// Capability que representa esta ventana en el grafo
    pub capability: Option<CapabilityId>,
}

impl Window {
    pub fn new(id: u32, title: &str, rect: Rect, window_type: WindowType) -> Self {
        let attributes = match window_type {
            WindowType::Normal => WindowAttributes::default_normal(),
            WindowType::Dialog => WindowAttributes::default_dialog(),
            WindowType::Menu => WindowAttributes::default_menu(),
            WindowType::Tooltip => WindowAttributes::default_menu(),
            WindowType::Splash => WindowAttributes::default_menu(),
            WindowType::Overlay => WindowAttributes {
                title_bar: false,
                close_button: false,
                minimize_button: false,
                maximize_button: false,
                resizable: false,
                movable: true,
                always_on_top: true,
            },
        };

        Self {
            id,
            title: String::from(title),
            rect,
            state: WindowState::Normal,
            window_type,
            attributes,
            z_order: 0,
            visible: true,
            capability: None,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.rect.x = x;
        self.rect.y = y;
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.rect.width = width;
        self.rect.height = height;
    }

    pub fn position(&self) -> Point {
        Point::new(self.rect.x, self.rect.y)
    }

    pub fn size(&self) -> (u32, u32) {
        (self.rect.width, self.rect.height)
    }

    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_z_order(&mut self, z_order: u32) {
        self.z_order = z_order;
    }

    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains(point.x, point.y)
    }

    pub fn is_normal(&self) -> bool {
        self.state == WindowState::Normal
    }

    pub fn is_minimized(&self) -> bool {
        self.state == WindowState::Minimized
    }

    pub fn is_maximized(&self) -> bool {
        self.state == WindowState::Maximized
    }

    pub fn is_hidden(&self) -> bool {
        self.state == WindowState::Hidden
    }

    pub fn is_visible(&self) -> bool {
        self.visible && !self.is_hidden()
    }

    pub fn can_move(&self) -> bool {
        self.attributes.movable && self.is_normal()
    }

    pub fn can_resize(&self) -> bool {
        self.attributes.resizable && self.is_normal()
    }

    pub fn has_title_bar(&self) -> bool {
        self.attributes.title_bar
    }

    pub fn has_close_button(&self) -> bool {
        self.attributes.close_button
    }

    pub fn has_minimize_button(&self) -> bool {
        self.attributes.minimize_button
    }

    pub fn has_maximize_button(&self) -> bool {
        self.attributes.maximize_button
    }

    pub fn is_always_on_top(&self) -> bool {
        self.attributes.always_on_top
    }
}

/// Gestor de ventanas integrado con exokernel/grafos
pub struct WindowManager {
    windows: Vec<Window>,
    next_window_id: u32,
    focused_window_id: Option<u32>,
    screen_width: u32,
    screen_height: u32,
    /// Capability del graph kernel para registrar ventanas como nodos
    graph_kernel_capability: Option<CapabilityId>,
}

impl WindowManager {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            windows: Vec::new(),
            next_window_id: 1,
            focused_window_id: None,
            screen_width,
            screen_height,
            graph_kernel_capability: None,
        }
    }

    /// Crea una nueva ventana y la registra en el grafo de recursos
    pub fn create_window(&mut self, title: &str, x: i32, y: i32, width: u32, height: u32, window_type: WindowType) -> u32 {
        let rect = Rect::new(x, y, width, height);
        let mut window = Window::new(self.next_window_id, title, rect, window_type);
        window.z_order = self.windows.len() as u32;
        
        // En un sistema completo, aquí se registraría la ventana como nodo en el grafo
        // usando invoke_capability con el graph_kernel
        
        self.windows.push(window);
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        
        window_id
    }

    pub fn destroy_window(&mut self, window_id: u32) -> bool {
        if let Some(pos) = self.windows.iter().position(|w| w.id == window_id) {
            self.windows.remove(pos);
            
            // Actualizar z_order de las ventanas restantes
            for (i, window) in self.windows.iter_mut().enumerate() {
                window.z_order = i as u32;
            }
            
            // Si la ventana destruida estaba enfocada, limpiar el foco
            if self.focused_window_id == Some(window_id) {
                self.focused_window_id = None;
            }
            
            true
        } else {
            false
        }
    }

    pub fn get_window(&self, window_id: u32) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == window_id)
    }

    pub fn get_window_mut(&mut self, window_id: u32) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == window_id)
    }

    pub fn get_window_at_point(&self, point: Point) -> Option<&Window> {
        // Buscar ventanas visibles de mayor a menor z_order
        let mut windows: Vec<_> = self.windows.iter()
            .filter(|w| w.is_visible())
            .collect();
        
        windows.sort_by(|a, b| b.z_order.cmp(&a.z_order));
        
        windows.iter().find(|w| w.contains_point(point)).copied()
    }

    pub fn focus_window(&mut self, window_id: u32) -> bool {
        if self.get_window(window_id).is_some() {
            self.focused_window_id = Some(window_id);
            
            // Calcular max_z_order antes de hacer el borrow mutable
            let max_z_order = self.windows.iter().map(|w| w.z_order).max().unwrap_or(0);
            
            // Mover la ventana al frente (mayor z_order)
            if let Some(window) = self.get_window_mut(window_id) {
                window.z_order = max_z_order + 1;
            }
            
            true
        } else {
            false
        }
    }

    pub fn defocused_window(&mut self) {
        self.focused_window_id = None;
    }

    pub fn get_focused_window(&self) -> Option<&Window> {
        self.focused_window_id.and_then(|id| self.get_window(id))
    }

    pub fn get_focused_window_mut(&mut self) -> Option<&mut Window> {
        self.focused_window_id.and_then(|id| self.get_window_mut(id))
    }

    pub fn is_window_focused(&self, window_id: u32) -> bool {
        self.focused_window_id == Some(window_id)
    }

    pub fn list_windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn list_visible_windows(&self) -> Vec<&Window> {
        self.windows.iter().filter(|w| w.is_visible()).collect()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn visible_window_count(&self) -> usize {
        self.windows.iter().filter(|w| w.is_visible()).count()
    }

    pub fn minimize_window(&mut self, window_id: u32) -> bool {
        if let Some(window) = self.get_window_mut(window_id) {
            window.set_state(WindowState::Minimized);
            true
        } else {
            false
        }
    }

    pub fn maximize_window(&mut self, window_id: u32) -> bool {
        let (screen_width, screen_height) = (self.screen_width, self.screen_height);
        
        if let Some(window) = self.get_window_mut(window_id) {
            window.set_state(WindowState::Maximized);
            window.set_position(0, 0);
            window.set_size(screen_width, screen_height);
            true
        } else {
            false
        }
    }

    pub fn restore_window(&mut self, window_id: u32) -> bool {
        if let Some(window) = self.get_window_mut(window_id) {
            window.set_state(WindowState::Normal);
            true
        } else {
            false
        }
    }

    pub fn hide_window(&mut self, window_id: u32) -> bool {
        if let Some(window) = self.get_window_mut(window_id) {
            window.set_state(WindowState::Hidden);
            true
        } else {
            false
        }
    }

    pub fn show_window(&mut self, window_id: u32) -> bool {
        if let Some(window) = self.get_window_mut(window_id) {
            window.set_state(WindowState::Normal);
            true
        } else {
            false
        }
    }

    pub fn move_window(&mut self, window_id: u32, x: i32, y: i32) -> bool {
        if let Some(window) = self.get_window_mut(window_id) {
            if window.can_move() {
                window.set_position(x, y);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn resize_window(&mut self, window_id: u32, width: u32, height: u32) -> bool {
        if let Some(window) = self.get_window_mut(window_id) {
            if window.can_resize() {
                window.set_size(width, height);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new(1024, 768)
    }
}

/// Errores del gestor de ventanas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowManagerError {
    WindowNotFound,
    InvalidPosition,
    InvalidSize,
    WindowNotMovable,
    WindowNotResizable,
    WindowAlreadyFocused,
}

impl fmt::Display for WindowManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowManagerError::WindowNotFound => write!(f, "Window not found"),
            WindowManagerError::InvalidPosition => write!(f, "Invalid position"),
            WindowManagerError::InvalidSize => write!(f, "Invalid size"),
            WindowManagerError::WindowNotMovable => write!(f, "Window is not movable"),
            WindowManagerError::WindowNotResizable => write!(f, "Window is not resizable"),
            WindowManagerError::WindowAlreadyFocused => write!(f, "Window is already focused"),
        }
    }
}
