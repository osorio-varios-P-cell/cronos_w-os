//! Guest Integration Module - Seamless Mode and Shared Resources
//!
//! This module manages the communication between CRONOS host and guest OSs
//! (Windows, Linux, macOS) for seamless window integration, shared clipboard,
//! and synchronized input events.

use crate::capability::{Capability, Cell, CapabilityId, invoke_capability_mut};
use crate::compositor::{Compositor, WindowId, Rect, WindowType};
use crate::graph_kernel::{GraphKernel, NodeId};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Types of guest messages
#[derive(Debug, Clone)]
pub enum GuestMessage {
    /// Request to create a seamless window for a guest application
    CreateWindow { title: String, width: u32, height: u32, os_type: String },
    /// Update window content (pixel buffer pointer)
    UpdateWindow { window_id: u64, buffer_ptr: u64 },
    /// Shared clipboard data
    ClipboardUpdate { data: String },
    /// Guest event (e.g., app closed)
    GuestEvent { event_id: u32, description: String },
}

/// Information about a synchronized guest window
pub struct GuestWindowInfo {
    pub host_window_id: WindowId,
    pub guest_internal_id: u64,
    pub os_type: String,
}

/// Sovereign Guest Integration Manager
pub struct GuestIntegrationManager {
    compositor: Capability<Compositor>,
    active_guest_windows: BTreeMap<u64, GuestWindowInfo>,
    shared_clipboard: String,
}

impl GuestIntegrationManager {
    pub fn new(compositor: Capability<Compositor>) -> Self {
        Self {
            compositor,
            active_guest_windows: BTreeMap::new(),
            shared_clipboard: String::new(),
        }
    }

    /// Process a message from a Guest Agent (Running inside the VM)
    pub fn handle_guest_message(&mut self, message: GuestMessage) -> Result<(), String> {
        match message {
            GuestMessage::CreateWindow { title, width, height, os_type } => {
                let rect = Rect::new(200, 200, width, height);
                invoke_capability_mut(&self.compositor, |comp| {
                    let window_id = comp.create_window(title, rect);
                    // Set as Foreign OS App for Seamless Blending
                    if let Some(window) = comp.get_window_mut(window_id) {
                        window.window_type = WindowType::ForeignOSApp { os_type: os_type.clone() };
                    }

                    self.active_guest_windows.insert(window_id.0, GuestWindowInfo {
                        host_window_id: window_id,
                        guest_internal_id: 0, // Assigned by guest
                        os_type,
                    });
                });
                Ok(())
            }
            GuestMessage::ClipboardUpdate { data } => {
                self.shared_clipboard = data;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Synchronize mouse event to a guest window
    pub fn sync_input_to_guest(&self, window_id: WindowId, x: i32, y: i32) {
        if self.active_guest_windows.contains_key(&window_id.0) {
            // En hardware real, enviaríamos la interrupción al núcleo de la VM
        }
    }
}
