//! Compositor - Window System as GraphNode with GPU Capability
//! 
//! This module implements the compositor as a GraphNode where windows are
//! nodes in the graph and GPU access is mediated through capabilities.
//! Inspired by Theseus OS compositor with region-based optimization.

use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};
use crate::capability::{Capability, Cell, CapabilityRights, invoke_capability_mut};
use crate::drivers::RedoxGpuDriver;
use crate::hal::{GpuDevice, Device, GpuContext, GpuCommand};
use alloc::collections::BTreeMap;
use alloc::{string::String, format, vec};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use core::ops::Range;

/// Window ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

impl WindowId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        WindowId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Rectangle for window positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        px >= self.x && px < (self.x + self.width as i32) &&
        py >= self.y && py < (self.y + self.height as i32)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32 &&
        self.x + self.width as i32 > other.x &&
        self.y < other.y + other.height as i32 &&
        self.y + self.height as i32 > other.y
    }

    /// Returns the range of rows covered by this rectangle (Theseus-style optimization)
    pub fn row_range(&self) -> Range<isize> {
        self.y as isize..(self.y + self.height as i32) as isize
    }

    /// Returns the size in pixels (Theseus-style optimization)
    pub fn size(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Returns the intersection rectangle with another rectangle
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width as i32).min(other.x + other.width as i32);
        let y2 = (self.y + self.height as i32).min(other.y + other.height as i32);

        if x2 > x1 && y2 > y1 {
            Some(Rect::new(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32))
        } else {
            None
        }
    }
}

/// Window state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Hidden,
}

/// Window type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowType {
    Normal,
    Popup,
    Menu,
    Tooltip,
    Dialog,
}

/// Window in the compositor
#[derive(Debug)]
pub struct Window {
    pub id: WindowId,
    pub node_id: NodeId,
    pub title: String,
    pub rect: Rect,
    pub state: WindowState,
    pub window_type: WindowType,
    pub visible: bool,
    pub focused: bool,
    pub z_order: u32,
    pub background_color: u32,
    pub created_at: u64,
}

impl Window {
    pub fn new(id: WindowId, node_id: NodeId, title: String, rect: Rect) -> Self {
        Self {
            id,
            node_id,
            title,
            rect,
            state: WindowState::Normal,
            window_type: WindowType::Normal,
            visible: true,
            focused: false,
            z_order: 0,
            background_color: 0xFF000000, // Black with full alpha
            created_at: 0,
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

    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_z_order(&mut self, z_order: u32) {
        self.z_order = z_order;
    }
}

/// Compositor Layer for performance optimization
#[derive(Debug, Clone)]
pub struct CompositorLayer {
    pub id: u32,
    pub name: String,
    pub active: bool,
    pub alpha: f32,
}

/// Compositor - manages windows as graph nodes
pub struct Compositor {
    graph_kernel: GraphKernel,
    compositor_node: Option<NodeId>,
    gpu_capability: Option<Capability<RedoxGpuDriver>>,
    windows: BTreeMap<WindowId, Window>,
    layers: Vec<CompositorLayer>,
    focused_window: Option<WindowId>,
    next_z_order: u32,
    screen_width: u32,
    screen_height: u32,
}

impl Compositor {
    pub fn new(graph_kernel: GraphKernel) -> Self {
        Self {
            graph_kernel,
            compositor_node: None,
            gpu_capability: None,
            windows: BTreeMap::new(),
            layers: vec![
                CompositorLayer { id: 0, name: String::from("Background"), active: true, alpha: 1.0 },
                CompositorLayer { id: 1, name: String::from("Applications"), active: true, alpha: 1.0 },
                CompositorLayer { id: 2, name: String::from("Overlay"), active: true, alpha: 1.0 },
            ],
            focused_window: None,
            next_z_order: 1,
            screen_width: 1920,
            screen_height: 1080,
        }
    }

    /// Initialize the compositor
    pub fn initialize(&mut self, gpu_capability: Capability<RedoxGpuDriver>) {
        // Create compositor node in the graph
        let compositor_node = self.graph_kernel.create_node(
            NodeType::Window,
            String::from("compositor"),
        );
        self.compositor_node = Some(compositor_node);
        self.gpu_capability = Some(gpu_capability);

        // Initialize GPU
        if let Some(ref gpu_cap) = self.gpu_capability {
            invoke_capability_mut(gpu_cap, |gpu| {
                let _ = gpu.init();
                let _ = gpu.set_resolution(self.screen_width, self.screen_height);
            });
        }
    }

    /// Set screen resolution
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
        if let Some(ref gpu_cap) = self.gpu_capability {
            invoke_capability_mut(gpu_cap, |gpu| {
                let _ = gpu.set_resolution(width, height);
            });
        }
    }

    /// Get screen resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    /// Create a new window
    pub fn create_window(&mut self, title: String, rect: Rect) -> WindowId {
        let window_id = WindowId::new();
        let window_name = format!("window_{}:{}", window_id.0, title);
        
        // Create window node in the graph
        let window_node = self.graph_kernel.create_node(
            NodeType::Window,
            window_name,
        );

        // Add metadata to the window node
        self.graph_kernel.invoke_node_operation_mut::<(), _, _>(window_node, |node| {
            node.set_metadata(String::from("width"), format!("{}", rect.width));
            node.set_metadata(String::from("height"), format!("{}", rect.height));
            node.set_metadata(String::from("x"), format!("{}", rect.x));
            node.set_metadata(String::from("y"), format!("{}", rect.y));
            node.set_metadata(String::from("window_id"), format!("{}", window_id.0));
        });

        // Connect window to compositor
        if let Some(compositor_node) = self.compositor_node {
            self.graph_kernel.create_edge(
                compositor_node,
                window_node,
                EdgeType::Ownership,
            );
        }

        // Create window object
        let window = Window::new(window_id, window_node, title, rect);
        self.windows.insert(window_id, window);

        window_id
    }

    /// Destroy a window
    pub fn destroy_window(&mut self, window_id: WindowId) -> bool {
        if let Some(window) = self.windows.remove(&window_id) {
            // Remove node from graph
            self.graph_kernel.remove_node(window.node_id);
            
            // Update focus if needed
            if self.focused_window == Some(window_id) {
                self.focused_window = None;
            }
            
            true
        } else {
            false
        }
    }

    /// Get a window by ID
    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id)
    }

    /// Get a mutable window by ID
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&window_id)
    }

    /// Set window position
    pub fn set_window_position(&mut self, window_id: WindowId, x: i32, y: i32) -> bool {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.set_position(x, y);
            true
        } else {
            false
        }
    }

    /// Set window size
    pub fn set_window_size(&mut self, window_id: WindowId, width: u32, height: u32) -> bool {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.set_size(width, height);
            true
        } else {
            false
        }
    }

    /// Set window state
    pub fn set_window_state(&mut self, window_id: WindowId, state: WindowState) -> bool {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.set_state(state);
            true
        } else {
            false
        }
    }

    /// Set window visibility
    pub fn set_window_visible(&mut self, window_id: WindowId, visible: bool) -> bool {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.set_visible(visible);
            true
        } else {
            false
        }
    }

    /// Focus a window
    pub fn focus_window(&mut self, window_id: WindowId) -> bool {
        // Unfocus previous window first
        if let Some(prev_id) = self.focused_window {
            if let Some(prev_window) = self.windows.get_mut(&prev_id) {
                prev_window.set_focused(false);
            }
        }

        // Focus new window
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.set_focused(true);
            window.set_z_order(self.next_z_order);
            self.next_z_order += 1;
            self.focused_window = Some(window_id);
            true
        } else {
            false
        }
    }

    /// Get focused window
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// List all windows
    pub fn list_windows(&self) -> Vec<WindowId> {
        self.windows.keys().cloned().collect()
    }

    /// Get windows in z-order (top to bottom)
    pub fn windows_in_z_order(&self) -> Vec<WindowId> {
        let mut windows: Vec<_> = self.windows.iter().collect();
        windows.sort_by(|a, b| b.1.z_order.cmp(&a.1.z_order));
        windows.into_iter().map(|(id, _)| *id).collect()
    }

    /// Render the compositor
    pub fn render(&mut self) {
        if let Some(ref gpu_cap) = self.gpu_capability {
            // Clear screen
            invoke_capability_mut(gpu_cap, |gpu| {
                let _ = gpu.execute_command(&GpuContext(0), 
                    GpuCommand::Clear { r: 20, g: 20, b: 30, a: 255 });
            });

            // Render windows in z-order (bottom to top)
            let windows_in_order = {
                let mut windows: Vec<_> = self.windows.iter().collect();
                windows.sort_by(|a, b| a.1.z_order.cmp(&b.1.z_order));
                windows
            };

            for (_window_id, window) in windows_in_order {
                if window.visible && window.state == WindowState::Normal {
                    self.render_window(gpu_cap, window);
                }
            }
        }
    }

    /// Render only specific regions (Theseus-style optimization)
    pub fn render_regions(&mut self, regions: &[Rect]) {
        if let Some(ref gpu_cap) = self.gpu_capability {
            for region in regions {
                self.render_region(gpu_cap, region);
            }
        }
    }

    /// Render a specific region (Theseus-style optimization)
    fn render_region(&self, gpu_cap: &Capability<RedoxGpuDriver>, region: &Rect) {
        // Find windows that intersect with this region
        let windows_in_order = {
            let mut windows: Vec<_> = self.windows.iter()
                .filter(|(_, w)| w.visible && w.state == WindowState::Normal && w.rect.intersects(region))
                .collect();
            windows.sort_by(|a, b| a.1.z_order.cmp(&b.1.z_order));
            windows
        };

        // Render only the intersecting parts
        for (_window_id, window) in windows_in_order {
            if let Some(intersection) = window.rect.intersection(region) {
                self.render_window_region(gpu_cap, window, &intersection);
            }
        }
    }

    /// Render a specific region of a window (Theseus-style optimization)
    fn render_window_region(&self, gpu_cap: &Capability<RedoxGpuDriver>, window: &Window, region: &Rect) {
        invoke_capability_mut(gpu_cap, |gpu| {
            let color = if window.focused {
                0xFF3B82F6 // Blue when focused
            } else {
                0xFF6B7280 // Gray when unfocused
            };

            // Draw window background region
            let _ = gpu.execute_command(&GpuContext(0),
                GpuCommand::DrawRect {
                    x: region.x as u32,
                    y: region.y as u32,
                    width: region.width,
                    height: region.height,
                    color,
                });
        });
    }

    /// Render a single window
    fn render_window(&self, gpu_cap: &Capability<RedoxGpuDriver>, window: &Window) {
        invoke_capability_mut(gpu_cap, |gpu| {
            let rect = window.rect;
            let color = if window.focused {
                0xFF3B82F6 // Blue when focused
            } else {
                0xFF6B7280 // Gray when unfocused
            };

            // Draw window background
            let _ = gpu.execute_command(&GpuContext(0),
                GpuCommand::DrawRect {
                    x: rect.x as u32,
                    y: rect.y as u32,
                    width: rect.width,
                    height: rect.height,
                    color,
                });

            // Draw window title bar (darker)
            let title_bar_height = 24;
            let title_color = if window.focused {
                0xFF1E40AF // Darker blue
            } else {
                0xFF4B5563 // Darker gray
            };

            let _ = gpu.execute_command(&GpuContext(0),
                GpuCommand::DrawRect {
                    x: rect.x as u32,
                    y: rect.y as u32,
                    width: rect.width,
                    height: title_bar_height,
                    color: title_color,
                });
        });
    }

    /// Handle input event
    pub fn handle_input(&mut self, x: i32, y: i32, button: bool) -> Option<WindowId> {
        // Find window under cursor (top to bottom)
        let windows_in_order = self.windows_in_z_order();
        
        for window_id in windows_in_order {
            if let Some(window) = self.windows.get(&window_id) {
                if window.visible && window.state == WindowState::Normal {
                    if window.rect.contains(x, y) {
                        if button {
                            self.focus_window(window_id);
                        }
                        return Some(window_id);
                    }
                }
            }
        }
        
        None
    }

    /// Get compositor node ID
    pub fn compositor_node(&self) -> Option<NodeId> {
        self.compositor_node
    }

    /// Get GPU capability
    pub fn gpu_capability(&self) -> Option<&Capability<RedoxGpuDriver>> {
        self.gpu_capability.as_ref()
    }
}

/// Compositor capability for external access
pub struct CompositorCapability {
    compositor: Cell<Compositor>,
    rights: CapabilityRights,
}

impl CompositorCapability {
    pub fn new(compositor: Compositor, rights: CapabilityRights) -> Self {
        Self {
            compositor: Cell::new(compositor),
            rights,
        }
    }

    pub fn capability(&self) -> Capability<Compositor> {
        self.compositor.capability_with_rights(self.rights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_kernel::GraphKernel;

    #[test]
    fn test_compositor_creation() {
        let graph_kernel = GraphKernel::new();
        let compositor = Compositor::new(graph_kernel);
        assert_eq!(compositor.list_windows().len(), 0);
    }

    #[test]
    fn test_window_creation() {
        let mut graph_kernel = GraphKernel::new();
        graph_kernel.initialize();
        let mut compositor = Compositor::new(graph_kernel);
        
        let gpu_cap = DriverFactory::create_gpu(0x1234, 0x5678, 0xb8000 as *mut u8, 1920, 1080).capability();
        compositor.initialize(gpu_cap);
        
        let window_id = compositor.create_window(
            String::from("Test Window"),
            Rect::new(100, 100, 400, 300),
        );
        
        assert!(compositor.get_window(window_id).is_some());
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 100);
        assert!(rect.contains(50, 50));
        assert!(!rect.contains(5, 5));
        assert!(!rect.contains(150, 150));
    }
}
