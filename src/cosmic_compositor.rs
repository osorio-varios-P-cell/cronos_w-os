//! Compositor moderno de Cosmic adaptado a CRONOS W-OS
//!
//! Este módulo incorpora técnicas de composición modernas de Cosmic
//! adaptadas a la arquitectura de exokernel con grafos y Lumen Layer

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;
use crate::graphics::{Color, Rect, Point};

/// Modo de composición (inspirado en Cosmic)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionMode {
    /// Composición normal
    Normal,
    /// Multiplicación
    Multiply,
    /// Pantalla
    Screen,
    /// Superposición
    Overlay,
    /// Oscurecer
    Darken,
    /// Aclarar
    Lighten,
}

/// Efecto de ventana (inspirado en Cosmic)
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEffect {
    /// Sin efecto
    None,
    /// Sombra
    Shadow,
    /// Desenfoque de fondo
    Blur,
    /// Transparencia
    Transparency(f32),
    /// Bordes redondeados
    RoundedCorners(f32),
}

/// Animación de ventana (inspirado en Cosmic)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowAnimation {
    /// Sin animación
    None,
    /// Desvanecimiento
    Fade,
    /// Escala
    Scale,
    /// Deslizamiento
    Slide,
    /// Rotación
    Rotate,
}

/// Estado de animación
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub animation_type: WindowAnimation,
    pub progress: f32, // 0.0 a 1.0
    pub duration_ms: u32,
    pub start_time_ms: u64,
}

impl AnimationState {
    pub fn new(animation_type: WindowAnimation, duration_ms: u32) -> Self {
        Self {
            animation_type,
            progress: 0.0,
            duration_ms,
            start_time_ms: 0,
        }
    }

    /// Actualizar la animación
    pub fn update(&mut self, elapsed_ms: u64) -> bool {
        let elapsed = (elapsed_ms - self.start_time_ms) as f32;
        self.progress = (elapsed / self.duration_ms as f32).min(1.0);
        self.progress >= 1.0
    }
}

/// Superficie de composición (inspirado en Cosmic)
#[derive(Debug, Clone)]
pub struct CompositionSurface {
    pub surface_id: u64,
    pub rect: Rect,
    pub z_index: i32,
    pub opacity: f32,
    pub composition_mode: CompositionMode,
    pub effects: Vec<WindowEffect>,
    pub animation: Option<AnimationState>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl CompositionSurface {
    pub fn new(surface_id: u64, rect: Rect) -> Self {
        Self {
            surface_id,
            rect,
            z_index: 0,
            opacity: 1.0,
            composition_mode: CompositionMode::Normal,
            effects: Vec::new(),
            animation: None,
            graph_node_id: None,
        }
    }

    /// Agregar un efecto
    pub fn add_effect(&mut self, effect: WindowEffect) {
        self.effects.push(effect);
    }

    /// Establecer la animación
    pub fn set_animation(&mut self, animation: WindowAnimation, duration_ms: u32) {
        self.animation = Some(AnimationState::new(animation, duration_ms));
    }

    /// Actualizar la animación
    pub fn update_animation(&mut self, elapsed_ms: u64) -> bool {
        if let Some(ref mut animation) = self.animation {
            animation.update(elapsed_ms)
        } else {
            false
        }
    }
}

/// Capa de composición (inspirado en Cosmic)
#[derive(Debug, Clone)]
pub struct CompositionLayer {
    pub layer_id: u64,
    pub surfaces: Vec<CompositionSurface>,
    pub visible: bool,
    pub opacity: f32,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl CompositionLayer {
    pub fn new(layer_id: u64) -> Self {
        Self {
            layer_id,
            surfaces: Vec::new(),
            visible: true,
            opacity: 1.0,
            graph_node_id: None,
        }
    }

    /// Agregar una superficie
    pub fn add_surface(&mut self, surface: CompositionSurface) {
        self.surfaces.push(surface);
    }

    /// Ordenar superficies por z-index
    pub fn sort_surfaces(&mut self) {
        self.surfaces.sort_by(|a, b| a.z_index.cmp(&b.z_index));
    }

    /// Obtener superficie por ID
    pub fn get_surface(&self, surface_id: u64) -> Option<&CompositionSurface> {
        self.surfaces.iter().find(|s| s.surface_id == surface_id)
    }

    /// Obtener superficie mutable por ID
    pub fn get_surface_mut(&mut self, surface_id: u64) -> Option<&mut CompositionSurface> {
        self.surfaces.iter_mut().find(|s| s.surface_id == surface_id)
    }
}

/// Compositor moderno de Cosmic adaptado
pub struct CosmicCompositor {
    pub layers: BTreeMap<u64, CompositionLayer>,
    pub next_layer_id: u64,
    pub next_surface_id: u64,
    pub active_layer_id: Option<u64>,
    pub graph_kernel: Option<Cell<GraphKernel>>,
    pub vsync_enabled: bool,
    pub target_fps: u32,
}

impl CosmicCompositor {
    pub fn new() -> Self {
        Self {
            layers: BTreeMap::new(),
            next_layer_id: 1,
            next_surface_id: 1,
            active_layer_id: None,
            graph_kernel: None,
            vsync_enabled: true,
            target_fps: 60,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Crear una nueva capa
    pub fn create_layer(&mut self) -> u64 {
        let layer_id = self.next_layer_id;
        self.next_layer_id += 1;

        let mut layer = CompositionLayer::new(layer_id);

        // Registrar la capa como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::Window;
            let node_name = format!("cosmic_layer_{}", layer_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            layer.graph_node_id = node_id;
        }

        self.layers.insert(layer_id, layer);
        layer_id
    }

    /// Crear una nueva superficie
    pub fn create_surface(&mut self, layer_id: u64, rect: Rect) -> Result<u64, String> {
        if !self.layers.contains_key(&layer_id) {
            return Err(format!("Layer {} not found", layer_id));
        }

        let surface_id = self.next_surface_id;
        self.next_surface_id += 1;

        let mut surface = CompositionSurface::new(surface_id, rect);

        // Registrar la superficie como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::Window;
            let node_name = format!("cosmic_surface_{}", surface_id);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            surface.graph_node_id = node_id;
        }

        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.add_surface(surface);
        }

        Ok(surface_id)
    }

    /// Establecer la capa activa
    pub fn set_active_layer(&mut self, layer_id: u64) -> Result<(), String> {
        if self.layers.contains_key(&layer_id) {
            self.active_layer_id = Some(layer_id);
            Ok(())
        } else {
            Err(format!("Layer {} not found", layer_id))
        }
    }

    /// Obtener la capa activa
    pub fn get_active_layer(&self) -> Option<&CompositionLayer> {
        self.active_layer_id.and_then(|id| self.layers.get(&id))
    }

    /// Obtener la capa activa mutable
    pub fn get_active_layer_mut(&mut self) -> Option<&mut CompositionLayer> {
        self.active_layer_id.and_then(|id| self.layers.get_mut(&id))
    }

    /// Componer todas las capas
    pub fn compose(&mut self) -> Result<(), String> {
        // En un sistema real, aquí se:
        // 1. Ordenaría las capas por z-index
        // 2. Compondría cada superficie con su modo de composición
        // 3. Aplicaría los efectos de ventana
        // 4. Renderizaría el resultado final al framebuffer

        // Ordenar superficies en cada capa
        for layer in self.layers.values_mut() {
            layer.sort_surfaces();
        }

        Ok(())
    }

    /// Actualizar animaciones
    pub fn update_animations(&mut self, elapsed_ms: u64) {
        for layer in self.layers.values_mut() {
            for surface in layer.surfaces.iter_mut() {
                if surface.update_animation(elapsed_ms) {
                    // Animación completada
                    surface.animation = None;
                }
            }
        }
    }

    /// Habilitar/deshabilitar vsync
    pub fn set_vsync(&mut self, enabled: bool) {
        self.vsync_enabled = enabled;
    }

    /// Establecer FPS objetivo
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
    }

    /// Obtener número de capas
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Obtener número total de superficies
    pub fn surface_count(&self) -> usize {
        self.layers.values().map(|l| l.surfaces.len()).sum()
    }

    /// Eliminar una capa
    pub fn remove_layer(&mut self, layer_id: u64) -> Result<(), String> {
        if self.layers.remove(&layer_id).is_some() {
            if self.active_layer_id == Some(layer_id) {
                self.active_layer_id = None;
            }
            Ok(())
        } else {
            Err(format!("Layer {} not found", layer_id))
        }
    }

    /// Eliminar una superficie
    pub fn remove_surface(&mut self, layer_id: u64, surface_id: u64) -> Result<(), String> {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            if let Some(pos) = layer.surfaces.iter().position(|s| s.surface_id == surface_id) {
                layer.surfaces.remove(pos);
                Ok(())
            } else {
                Err(format!("Surface {} not found", surface_id))
            }
        } else {
            Err(format!("Layer {} not found", layer_id))
        }
    }
}

impl Default for CosmicCompositor {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del compositor Cosmic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CosmicCompositorError {
    LayerNotFound,
    SurfaceNotFound,
    InvalidRect,
    CompositionFailed,
    AnimationFailed,
}

impl fmt::Display for CosmicCompositorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CosmicCompositorError::LayerNotFound => write!(f, "Layer not found"),
            CosmicCompositorError::SurfaceNotFound => write!(f, "Surface not found"),
            CosmicCompositorError::InvalidRect => write!(f, "Invalid rectangle"),
            CosmicCompositorError::CompositionFailed => write!(f, "Composition failed"),
            CosmicCompositorError::AnimationFailed => write!(f, "Animation failed"),
        }
    }
}
