//! Módulo de Gráficos LUMEN para CRONOS W-OS
//! Implementa sistema de gráficos avanzado con compositor y rendering

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Resolución de pantalla
#[derive(Debug, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Color RGBA
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Crea un nuevo color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Color negro
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    /// Color blanco
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    /// Color rojo
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    /// Color verde
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    /// Color azul
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
}

/// Rectángulo
#[derive(Debug, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Framebuffer
#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bpp: u16,
    pub buffer: Vec<u8>,
}

impl Framebuffer {
    /// Crea un nuevo framebuffer
    pub fn new(width: u32, height: u32, bpp: u16) -> Self {
        let stride = width * (bpp as u32 / 8);
        let buffer_size = (stride * height) as usize;
        
        Framebuffer {
            width,
            height,
            stride,
            bpp,
            buffer: vec![0; buffer_size],
        }
    }

    /// Establece un pixel
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let offset = ((y * self.stride + x * (self.bpp as u32 / 8)) as usize);
            match self.bpp {
                32 => {
                    self.buffer[offset] = color.b;
                    self.buffer[offset + 1] = color.g;
                    self.buffer[offset + 2] = color.r;
                    self.buffer[offset + 3] = color.a;
                }
                24 => {
                    self.buffer[offset] = color.b;
                    self.buffer[offset + 1] = color.g;
                    self.buffer[offset + 2] = color.r;
                }
                _ => {}
            }
        }
    }

    /// Obtiene un pixel
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let offset = ((y * self.stride + x * (self.bpp as u32 / 8)) as usize);
            match self.bpp {
                32 => {
                    Some(Color {
                        b: self.buffer[offset],
                        g: self.buffer[offset + 1],
                        r: self.buffer[offset + 2],
                        a: self.buffer[offset + 3],
                    })
                }
                24 => {
                    Some(Color {
                        b: self.buffer[offset],
                        g: self.buffer[offset + 1],
                        r: self.buffer[offset + 2],
                        a: 255,
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Llena el framebuffer con un color
    pub fn fill(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }

    /// Dibuja un rectángulo
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..(rect.y + rect.height as i32) {
            for x in rect.x..(rect.x + rect.width as i32) {
                if x >= 0 && y >= 0 {
                    self.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Ventana
#[derive(Debug, Clone)]
pub struct Window {
    pub id: u64,
    pub title: String,
    pub rect: Rect,
    pub visible: bool,
    pub framebuffer: Framebuffer,
}

/// Compositor
#[derive(Debug, Clone)]
pub struct Compositor {
    pub windows: BTreeMap<u64, Window>,
    pub next_window_id: u64,
    pub background_color: Color,
}

impl Compositor {
    /// Crea un nuevo compositor
    pub fn new() -> Self {
        Compositor {
            windows: BTreeMap::new(),
            next_window_id: 1,
            background_color: Color::BLACK,
        }
    }

    /// Crea una nueva ventana
    pub fn create_window(&mut self, title: String, rect: Rect) -> u64 {
        let window_id = self.next_window_id;
        self.next_window_id += 1;

        let framebuffer = Framebuffer::new(rect.width, rect.height, 32);

        let window = Window {
            id: window_id,
            title,
            rect,
            visible: true,
            framebuffer,
        };

        self.windows.insert(window_id, window);
        println!("🪟 Ventana creada: ID={}, {}", window_id, window.title);
        window_id
    }

    /// Elimina una ventana
    pub fn destroy_window(&mut self, window_id: u64) {
        self.windows.remove(&window_id);
        println!("🗑️ Ventana eliminada: ID={}", window_id);
    }

    /// Mueve una ventana
    pub fn move_window(&mut self, window_id: u64, x: i32, y: i32) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.rect.x = x;
            window.rect.y = y;
            println!("🔄 Ventana movida: ID={}, x={}, y={}", window_id, x, y);
        }
    }

    /// Redimensiona una ventana
    pub fn resize_window(&mut self, window_id: u64, width: u32, height: u32) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.rect.width = width;
            window.rect.height = height;
            window.framebuffer = Framebuffer::new(width, height, 32);
            println!("📐 Ventana redimensionada: ID={}, {}x{}", window_id, width, height);
        }
    }

    /// Muestra una ventana
    pub fn show_window(&mut self, window_id: u64) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.visible = true;
            println!("👁️ Ventana mostrada: ID={}", window_id);
        }
    }

    /// Oculta una ventana
    pub fn hide_window(&mut self, window_id: u64) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.visible = false;
            println!("🙈 Ventana oculta: ID={}", window_id);
        }
    }

    /// Composita todas las ventanas
    pub fn composite(&self, target: &mut Framebuffer) {
        // Llenar con color de fondo
        target.fill(self.background_color);

        // Compositar ventanas en orden
        for window in self.windows.values() {
            if window.visible {
                self.composite_window(window, target);
            }
        }
    }

    /// Composita una ventana
    fn composite_window(&self, window: &Window, target: &mut Framebuffer) {
        let src = &window.framebuffer;
        let dst = target;

        for y in 0..window.rect.height {
            for x in 0..window.rect.width {
                let src_x = x;
                let src_y = y;
                let dst_x = (window.rect.x + x as i32) as u32;
                let dst_y = (window.rect.y + y as i32) as u32;

                if let Some(color) = src.get_pixel(src_x, src_y) {
                    if dst_x < dst.width && dst_y < dst.height {
                        dst.set_pixel(dst_x, dst_y, color);
                    }
                }
            }
        }
    }
}

/// Sistema de gráficos LUMEN
pub struct LumenGraphicsSystem {
    pub compositor: Compositor,
    pub screen: Framebuffer,
    pub resolution: Resolution,
}

impl LumenGraphicsSystem {
    /// Crea un nuevo sistema de gráficos LUMEN
    pub fn new() -> Self {
        let resolution = Resolution {
            width: 1920,
            height: 1080,
        };

        let screen = Framebuffer::new(resolution.width, resolution.height, 32);
        let compositor = Compositor::new();

        LumenGraphicsSystem {
            compositor,
            screen,
            resolution,
        }
    }

    /// Inicializa el sistema de gráficos
    pub fn initialize(&mut self) {
        println!("🎨 Inicializando Sistema de Gráficos LUMEN...");
        println!("   - Resolución: {}x{}", self.resolution.width, self.resolution.height);
        println!("   - Compositor: Inicializado");
        
        // Crear ventana de escritorio
        let desktop_rect = Rect {
            x: 0,
            y: 0,
            width: self.resolution.width,
            height: self.resolution.height,
        };
        
        self.compositor.create_window(String::from("Desktop"), desktop_rect);
        
        println!("✅ Sistema de Gráficos LUMEN inicializado");
    }

    /// Establece la resolución
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.resolution = Resolution { width, height };
        self.screen = Framebuffer::new(width, height, 32);
        println!("📐 Resolución establecida: {}x{}", width, height);
    }

    /// Obtiene el framebuffer de pantalla
    pub fn get_screen(&mut self) -> &mut Framebuffer {
        &mut self.screen
    }

    /// Renderiza un frame
    pub fn render_frame(&mut self) {
        self.compositor.composite(&mut self.screen);
    }

    /// Crea una ventana
    pub fn create_window(&mut self, title: String, rect: Rect) -> u64 {
        self.compositor.create_window(title, rect)
    }

    /// Elimina una ventana
    pub fn destroy_window(&mut self, window_id: u64) {
        self.compositor.destroy_window(window_id);
    }

    /// Genera reporte de gráficos
    pub fn generate_report(&self) -> GraphicsReport {
        GraphicsReport {
            resolution: self.resolution.clone(),
            total_windows: self.compositor.windows.len(),
            visible_windows: self.compositor.windows.values().filter(|w| w.visible).count(),
        }
    }
}

/// Reporte de gráficos
#[derive(Debug, Clone)]
pub struct GraphicsReport {
    pub resolution: Resolution,
    pub total_windows: usize,
    pub visible_windows: usize,
}
