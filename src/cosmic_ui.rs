//! COSMIC-inspired UI Layer for CRONOS W-OS
//! 
//! This module implements a UI layer inspired by COSMIC desktop
//! that works on top of the compositor with capability-based access.

use crate::compositor::{Compositor, WindowId, Rect, WindowState, WindowType};
use crate::capability::{Capability, Cell, CapabilityRights, invoke_capability, invoke_capability_mut};
use alloc::string::String;
use alloc::vec::Vec;

/// COSMIC-inspired color palette
#[derive(Debug, Clone, Copy)]
pub struct CosmicPalette {
    /// Primary color (blue)
    pub primary: u32,
    /// Secondary color (purple)
    pub secondary: u32,
    /// Background color
    pub background: u32,
    /// Surface color
    pub surface: u32,
    /// Text color
    pub text: u32,
    /// Accent color
    pub accent: u32,
}

impl Default for CosmicPalette {
    fn default() -> Self {
        Self {
            primary: 0xFF3B82F6,     // Blue
            secondary: 0xFF8B5CF6,   // Purple
            background: 0xFF1E1E2E,   // Dark background
            surface: 0xFF2A2A3C,     // Surface
            text: 0xFFFFFFFF,        // White text
            accent: 0xFF10B981,      // Green accent
        }
    }
}

impl CosmicPalette {
    /// Create a dark theme palette
    pub fn dark() -> Self {
        Self::default()
    }

    /// Create a light theme palette
    pub fn light() -> Self {
        Self {
            primary: 0xFF3B82F6,
            secondary: 0xFF8B5CF6,
            background: 0xFFFFFFFF,
            surface: 0xFFF3F4F6,
            text: 0xFF1F2937,
            accent: 0xFF10B981,
        }
    }
}

/// COSMIC-inspired widget types
#[derive(Debug, Clone, PartialEq)]
pub enum CosmicWidget {
    /// Button widget
    Button {
        label: String,
        action: CosmicAction,
    },
    /// Label widget
    Label {
        text: String,
        size: u32,
    },
    /// Panel widget (taskbar)
    Panel {
        position: PanelPosition,
        items: Vec<PanelItem>,
    },
    /// Launcher widget (app launcher)
    Launcher {
        apps: Vec<AppEntry>,
    },
    /// Window decoration
    WindowDecoration {
        title: String,
        closeable: bool,
        minimizable: bool,
        maximizable: bool,
    },
}

/// Panel position
#[derive(Debug, Clone, PartialEq)]
pub enum PanelPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// Panel item
#[derive(Debug, Clone, PartialEq)]
pub struct PanelItem {
    pub name: String,
    pub icon: Option<String>,
    pub action: CosmicAction,
}

/// App entry for launcher
#[derive(Debug, Clone, PartialEq)]
pub struct AppEntry {
    pub name: String,
    pub icon: Option<String>,
    pub command: String,
}

/// Cosmic action
#[derive(Debug, Clone, PartialEq)]
pub enum CosmicAction {
    /// No action
    None,
    /// Execute command
    Execute(String),
    /// Open window
    OpenWindow(String),
    /// Close window
    CloseWindow(WindowId),
    /// Minimize window
    MinimizeWindow(WindowId),
    /// Maximize window
    MaximizeWindow(WindowId),
    /// Focus window
    FocusWindow(WindowId),
}

/// COSMIC UI layer
#[derive(Debug, Clone)]
pub struct CosmicUi {
    /// Compositor reference
    compositor: Capability<Compositor>,
    /// Color palette
    palette: CosmicPalette,
    /// Panel window ID
    panel_window: Option<WindowId>,
    /// Launcher window ID
    launcher_window: Option<WindowId>,
}

impl CosmicUi {
    /// Create a new COSMIC UI layer
    pub fn new(compositor: Capability<Compositor>, palette: CosmicPalette) -> Self {
        Self {
            compositor,
            palette,
            panel_window: None,
            launcher_window: None,
        }
    }

    /// Initialize the COSMIC UI
    pub fn initialize(&mut self, screen_width: u32, screen_height: u32) {

        // Create panel (taskbar)
        self.create_panel(screen_width, screen_height);

    }

    /// Create the panel (taskbar)
    fn create_panel(&mut self, screen_width: u32, screen_height: u32) {
        let panel_height = 48;
        let panel_rect = Rect::new(0, (screen_height - panel_height) as i32, screen_width, panel_height);

        invoke_capability_mut(&self.compositor, |compositor| {
            let panel_window = compositor.create_window(
                String::from("COSMIC Panel"),
                panel_rect,
            );
            
            // Set panel properties
            if let Some(window) = compositor.get_window_mut(panel_window) {
                window.window_type = WindowType::Popup;
                window.z_order = 9999; // Always on top
                window.background_color = self.palette.surface;
            }

            self.panel_window = Some(panel_window);
        });
    }

    /// Show the launcher
    pub fn show_launcher(&mut self, screen_width: u32, screen_height: u32) {
        let launcher_width = 600;
        let launcher_height = 400;
        let launcher_x = ((screen_width - launcher_width) / 2) as i32;
        let launcher_y = ((screen_height - launcher_height) / 2) as i32;
        let launcher_rect = Rect::new(launcher_x, launcher_y, launcher_width, launcher_height);

        invoke_capability_mut(&self.compositor, |compositor| {
            let launcher_window = compositor.create_window(
                String::from("COSMIC Launcher"),
                launcher_rect,
            );
            
            // Set launcher properties
            if let Some(window) = compositor.get_window_mut(launcher_window) {
                window.window_type = WindowType::Popup;
                window.z_order = 10000; // Above panel
                window.background_color = self.palette.background;
            }

            self.launcher_window = Some(launcher_window);
        });
    }

    /// Hide the launcher
    pub fn hide_launcher(&mut self) {
        if let Some(launcher_window) = self.launcher_window {
            invoke_capability_mut(&self.compositor, |compositor| {
                compositor.destroy_window(launcher_window);
            });
            self.launcher_window = None;
        }
    }

    /// Toggle the launcher
    pub fn toggle_launcher(&mut self, screen_width: u32, screen_height: u32) {
        if self.launcher_window.is_some() {
            self.hide_launcher();
        } else {
            self.show_launcher(screen_width, screen_height);
        }
    }

    /// Handle a cosmic action
    pub fn handle_action(&mut self, action: CosmicAction) {
        match action {
            CosmicAction::None => {}
            CosmicAction::Execute(_) => {
            }
            CosmicAction::OpenWindow(title) => {
                invoke_capability_mut(&self.compositor, |compositor| {
                    let rect = Rect::new(100, 100, 800, 600);
                    compositor.create_window(title, rect);
                });
            }
            CosmicAction::CloseWindow(window_id) => {
                invoke_capability_mut(&self.compositor, |compositor| {
                    compositor.destroy_window(window_id);
                });
            }
            CosmicAction::MinimizeWindow(window_id) => {
                invoke_capability_mut(&self.compositor, |compositor| {
                    compositor.set_window_state(window_id, WindowState::Minimized);
                });
            }
            CosmicAction::MaximizeWindow(window_id) => {
                invoke_capability_mut(&self.compositor, |compositor| {
                    compositor.set_window_state(window_id, WindowState::Maximized);
                });
            }
            CosmicAction::FocusWindow(window_id) => {
                invoke_capability_mut(&self.compositor, |compositor| {
                    compositor.focus_window(window_id);
                });
            }
        }
    }

    /// Get the color palette
    pub fn palette(&self) -> CosmicPalette {
        self.palette
    }

    /// Set the color palette
    pub fn set_palette(&mut self, palette: CosmicPalette) {
        self.palette = palette;
    }
}

/// COSMIC UI capability for external access
pub struct CosmicUiCapability {
    cosmic_ui: Cell<CosmicUi>,
    rights: CapabilityRights,
}

impl CosmicUiCapability {
    pub fn new(cosmic_ui: CosmicUi, rights: CapabilityRights) -> Self {
        Self {
            cosmic_ui: Cell::new(cosmic_ui),
            rights,
        }
    }

    pub fn capability(&self) -> Capability<CosmicUi> {
        self.cosmic_ui.capability_with_rights(self.rights)
    }
}
