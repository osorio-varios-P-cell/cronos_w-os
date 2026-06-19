//! Graphics System para CRONOS W-OS
//!
//! Este módulo implementa el sistema gráfico adaptado a la arquitectura
//! de exokernel con grafos y sistema de capabilities

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};

/// Color RGB (24 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    pub fn white() -> Self {
        Self { r: 255, g: 255, b: 255 }
    }

    pub fn red() -> Self {
        Self { r: 255, g: 0, b: 0 }
    }

    pub fn green() -> Self {
        Self { r: 0, g: 255, b: 0 }
    }

    pub fn blue() -> Self {
        Self { r: 0, g: 0, b: 255 }
    }

    pub fn yellow() -> Self {
        Self { r: 255, g: 255, b: 0 }
    }

    pub fn cyan() -> Self {
        Self { r: 0, g: 255, b: 255 }
    }

    pub fn magenta() -> Self {
        Self { r: 255, g: 0, b: 255 }
    }

    pub fn gray() -> Self {
        Self { r: 128, g: 128, b: 128 }
    }

    pub fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_u32(value: u32) -> Self {
        Self {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
        }
    }
}

/// Color RGBA (32 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_color(color: Color, a: u8) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a,
        }
    }

    pub fn to_u32(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
}

/// Punto 2D
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// Rectángulo
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

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x < (self.x + self.width as i32) &&
        point.y >= self.y && point.y < (self.y + self.height as i32)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < (other.x + other.width as i32) &&
        (self.x + self.width as i32) > other.x &&
        self.y < (other.y + other.height as i32) &&
        (self.y + self.height as i32) > other.y
    }
}

/// Información del modo de video
#[derive(Debug, Clone, Copy)]
pub struct VideoMode {
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub pitch: u32,
}

impl VideoMode {
    pub fn new(width: u32, height: u32, bpp: u8, pitch: u32) -> Self {
        Self {
            width,
            height,
            bpp,
            pitch,
        }
    }

    pub fn vga_text() -> Self {
        Self {
            width: 80,
            height: 25,
            bpp: 4,
            pitch: 80 * 2,
        }
    }

    pub fn vga_1024x768() -> Self {
        Self {
            width: 1024,
            height: 768,
            bpp: 32,
            pitch: 1024 * 4,
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn size(&self) -> usize {
        (self.width * self.height * (self.bpp as u32) / 8) as usize
    }
}

/// Framebuffer abstracto
pub struct Framebuffer {
    pub buffer: Vec<u8>,
    pub mode: VideoMode,
}

impl Framebuffer {
    pub fn new(mode: VideoMode) -> Self {
        let size = mode.size();
        let buffer = vec![0u8; size];
        
        Self {
            buffer,
            mode,
        }
    }

    pub fn width(&self) -> u32 {
        self.mode.width
    }

    pub fn height(&self) -> u32 {
        self.mode.height
    }

    pub fn bpp(&self) -> u8 {
        self.mode.bpp
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width() || y >= self.height() {
            return;
        }

        let offset = (y * self.mode.pitch + x * (self.bpp() as u32) / 8) as usize;
        
        match self.bpp() {
            24 => {
                if offset + 3 <= self.buffer.len() {
                    self.buffer[offset] = color.b;
                    self.buffer[offset + 1] = color.g;
                    self.buffer[offset + 2] = color.r;
                }
            }
            32 => {
                if offset + 4 <= self.buffer.len() {
                    self.buffer[offset] = color.b;
                    self.buffer[offset + 1] = color.g;
                    self.buffer[offset + 2] = color.r;
                    self.buffer[offset + 3] = 255;
                }
            }
            _ => {}
        }
    }

    pub fn fill(&mut self, color: Color) {
        let color_bytes = [color.b, color.g, color.r];
        let bytes_per_pixel = (self.bpp() as usize) / 8;
        
        for i in (0..self.buffer.len()).step_by(bytes_per_pixel) {
            for j in 0..bytes_per_pixel.min(3) {
                if i + j < self.buffer.len() {
                    self.buffer[i + j] = color_bytes[j];
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill(Color::black());
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Operación Blit: Copia un rectángulo de este framebuffer a otro
    pub fn blit(&self, dest: &mut Framebuffer, src_rect: Rect, dest_point: Point) {
        let bytes_per_pixel = (self.bpp() as u32) / 8;

        for y in 0..src_rect.height {
            let src_y = src_rect.y + y as i32;
            let dest_y = dest_point.y + y as i32;

            if src_y < 0 || src_y >= self.height() as i32 || dest_y < 0 || dest_y >= dest.height() as i32 {
                continue;
            }

            let src_offset = (src_y as u32 * self.mode.pitch + src_rect.x as u32 * bytes_per_pixel) as usize;
            let dest_offset = (dest_y as u32 * dest.mode.pitch + dest_point.x as u32 * bytes_per_pixel) as usize;
            let line_width = (src_rect.width * bytes_per_pixel) as usize;

            if src_offset + line_width <= self.buffer.len() && dest_offset + line_width <= dest.buffer.len() {
                dest.buffer[dest_offset..dest_offset + line_width]
                    .copy_from_slice(&self.buffer[src_offset..src_offset + line_width]);
            }
        }
    }
}

/// Contexto gráfico con soporte para Doble Buffer
pub struct GraphicsContext {
    /// Buffer principal (oculto)
    pub back_buffer: Framebuffer,
    /// Referencia al buffer frontal (hardware/pantalla)
    pub front_buffer_addr: *mut u8,
    pub draw_color: Color,
    pub fill_color: Color,
}

impl GraphicsContext {
    pub fn new(framebuffer: Framebuffer, front_buffer_addr: *mut u8) -> Self {
        Self {
            back_buffer: framebuffer,
            front_buffer_addr,
            draw_color: Color::white(),
            fill_color: Color::black(),
        }
    }

    pub fn set_draw_color(&mut self, color: Color) {
        self.draw_color = color;
    }

    pub fn set_fill_color(&mut self, color: Color) {
        self.fill_color = color;
    }

    /// Vuelca el back_buffer al front_buffer (pantalla)
    pub fn swap_buffers(&mut self) {
        if self.front_buffer_addr.is_null() { return; }

        let size = self.back_buffer.buffer.len();
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.back_buffer.buffer.as_ptr(),
                self.front_buffer_addr,
                size
            );
        }
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            self.back_buffer.set_pixel(x as u32, y as u32, self.draw_color);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_rect(&mut self, rect: Rect) {
        let x1 = rect.x;
        let y1 = rect.y;
        let x2 = rect.x + rect.width as i32 - 1;
        let y2 = rect.y + rect.height as i32 - 1;

        self.draw_line(x1, y1, x2, y1);
        self.draw_line(x1, y2, x2, y2);
        self.draw_line(x1, y1, x1, y2);
        self.draw_line(x2, y1, x2, y2);
    }

    pub fn fill_rect(&mut self, rect: Rect) {
        for y in rect.y..(rect.y + rect.height as i32) {
            for x in rect.x..(rect.x + rect.width as i32) {
                self.back_buffer.set_pixel(x as u32, y as u32, self.fill_color);
            }
        }
    }

    /// Dibuja un rectángulo con bordes redondeados (Estilo Crystal UI)
    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: i32) {
        let x1 = rect.x;
        let y1 = rect.y;
        let x2 = rect.x + rect.width as i32 - 1;
        let y2 = rect.y + rect.height as i32 - 1;

        // Cuerpo central
        self.fill_rect(Rect::new(x1 + radius, y1, rect.width - 2 * radius as u32, rect.height));
        self.fill_rect(Rect::new(x1, y1 + radius, radius as u32, rect.height - 2 * radius as u32));
        self.fill_rect(Rect::new(x2 - radius + 1, y1 + radius, radius as u32, rect.height - 2 * radius as u32));

        // Esquinas (Círculos)
        self.fill_circle(x1 + radius, y1 + radius, radius);
        self.fill_circle(x2 - radius, y1 + radius, radius);
        self.fill_circle(x1 + radius, y2 - radius, radius);
        self.fill_circle(x2 - radius, y2 - radius, radius);
    }

    /// Implementación de desenfoque Gaussiano (Glassmorphism)
    pub fn apply_gaussian_blur(&mut self, rect: Rect, radius: u32) {
        if radius == 0 { return; }

        let width = self.back_buffer.width();
        let height = self.back_buffer.height();
        let bytes_per_pixel = (self.back_buffer.bpp() as u32) / 8;

        // Simulación de desenfoque mediante promedio de área (box blur simplificado)
        for y in rect.y..(rect.y + rect.height as i32) {
            for x in rect.x..(rect.x + rect.width as i32) {
                if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 { continue; }

                let mut r_total: u32 = 0;
                let mut g_total: u32 = 0;
                let mut b_total: u32 = 0;
                let mut count: u32 = 0;

                let r_i = radius as i32;
                for dy in -r_i..=r_i {
                    for dx in -r_i..=r_i {
                        let nx = x + dx;
                        let ny = y + dy;

                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let offset = (ny as u32 * self.back_buffer.mode.pitch + nx as u32 * bytes_per_pixel) as usize;
                            b_total += self.back_buffer.buffer[offset] as u32;
                            g_total += self.back_buffer.buffer[offset + 1] as u32;
                            r_total += self.back_buffer.buffer[offset + 2] as u32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let offset = (y as u32 * self.back_buffer.mode.pitch + x as u32 * bytes_per_pixel) as usize;
                    self.back_buffer.buffer[offset] = (b_total / count) as u8;
                    self.back_buffer.buffer[offset + 1] = (g_total / count) as u8;
                    self.back_buffer.buffer[offset + 2] = (r_total / count) as u8;
                }
            }
        }
    }

    /// Dibuja una sombra dinámica (Drop Shadow)
    pub fn draw_shadow(&mut self, rect: Rect, intensity: u8) {
        let shadow_rect = Rect::new(rect.x + 4, rect.y + 4, rect.width, rect.height);
        let old_fill = self.fill_color;
        self.fill_color = Color::new(0, 0, 0); // Sombra negra

        // Dibujar sombra con transparencia simulada (mezcla con el fondo existente)
        let width = self.back_buffer.width();
        let height = self.back_buffer.height();
        let bytes_per_pixel = (self.back_buffer.bpp() as u32) / 8;

        for y in shadow_rect.y..(shadow_rect.y + shadow_rect.height as i32) {
            for x in shadow_rect.x..(shadow_rect.x + shadow_rect.width as i32) {
                if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 { continue; }

                let offset = (y as u32 * self.back_buffer.mode.pitch + x as u32 * bytes_per_pixel) as usize;
                // Mezcla simple 50/50 con negro para simular sombra
                self.back_buffer.buffer[offset] /= 2;
                self.back_buffer.buffer[offset + 1] /= 2;
                self.back_buffer.buffer[offset + 2] /= 2;
            }
        }
        self.fill_color = old_fill;
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.back_buffer.set_pixel((cx + x) as u32, (cy + y) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx + y) as u32, (cy + x) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx - y) as u32, (cy + x) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx - x) as u32, (cy + y) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx - x) as u32, (cy - y) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx - y) as u32, (cy - x) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx + y) as u32, (cy - x) as u32, self.draw_color);
            self.back_buffer.set_pixel((cx + x) as u32, (cy - y) as u32, self.draw_color);

            if err <= 0 {
                y += 1;
                err += 2 * y + 1;
            }

            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32) {
        for y in (cy - radius)..=(cy + radius) {
            for x in (cx - radius)..=(cx + radius) {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius {
                    self.back_buffer.set_pixel(x as u32, y as u32, self.fill_color);
                }
            }
        }
    }

    pub fn draw_char(&mut self, c: char, x: i32, y: i32) {
        let font_data = self.get_font_data(c);
        let font_height = 16;
        let font_width = 8;

        for row in 0..font_height {
            for col in 0..font_width {
                let byte_index = row;
                let bit_index = 7 - col;
                
                if byte_index < font_data.len() {
                    let byte = font_data[byte_index];
                    if (byte >> bit_index) & 1 == 1 {
                        self.back_buffer.set_pixel((x + col as i32) as u32, (y + row as i32) as u32, self.draw_color);
                    }
                }
            }
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32) {
        let mut current_x = x;
        let mut current_y = y;
        let char_width = 8;
        let char_height = 16;

        for c in text.chars() {
            if c == '\n' {
                current_x = x;
                current_y += char_height;
            } else if c == '\r' {
                current_x = x;
            } else if c == '\t' {
                current_x += char_width * 4;
            } else {
                self.draw_char(c, current_x, current_y);
                current_x += char_width;
            }
        }
    }

    fn get_font_data(&self, c: char) -> &'static [u8; 16] {
        match c {
            ' ' => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            'A' => &[0x00, 0x00, 0x18, 0x3C, 0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00],
            'B' => &[0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x7C, 0x00, 0x00, 0x00, 0x00],
            'C' => &[0x00, 0x00, 0x3C, 0x66, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            'D' => &[0x00, 0x00, 0x78, 0x6C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00, 0x00, 0x00, 0x00],
            'E' => &[0x00, 0x00, 0x7E, 0x60, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00, 0x00, 0x00, 0x00],
            'F' => &[0x00, 0x00, 0x7E, 0x60, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x60, 0x60, 0x00, 0x00, 0x00, 0x00],
            'G' => &[0x00, 0x00, 0x3C, 0x66, 0x60, 0x60, 0x6E, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            'H' => &[0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00],
            'I' => &[0x00, 0x00, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00],
            'J' => &[0x00, 0x00, 0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78, 0x00, 0x00, 0x00, 0x00],
            'K' => &[0x00, 0x00, 0xE6, 0x66, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00],
            'L' => &[0x00, 0x00, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00, 0x00, 0x00, 0x00],
            'M' => &[0x00, 0x00, 0x63, 0x77, 0x7F, 0x7B, 0x73, 0x63, 0x63, 0x63, 0x63, 0x63, 0x00, 0x00, 0x00, 0x00],
            'N' => &[0x00, 0x00, 0x66, 0x76, 0x7E, 0x7E, 0x76, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00],
            'O' => &[0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            'P' => &[0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x60, 0x60, 0x00, 0x00, 0x00, 0x00],
            'Q' => &[0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x0C, 0x0E, 0x00, 0x00],
            'R' => &[0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00],
            'S' => &[0x00, 0x00, 0x3C, 0x66, 0x60, 0x60, 0x3C, 0x06, 0x06, 0x06, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            'T' => &[0x00, 0x00, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00],
            'U' => &[0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            'V' => &[0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00, 0x00, 0x00, 0x00],
            'W' => &[0x00, 0x00, 0x63, 0x63, 0x63, 0x63, 0x6B, 0x7F, 0x7F, 0x77, 0x63, 0x63, 0x00, 0x00, 0x00, 0x00],
            'X' => &[0x00, 0x00, 0xC3, 0xC3, 0x66, 0x3C, 0x18, 0x18, 0x3C, 0x66, 0xC3, 0xC3, 0x00, 0x00, 0x00, 0x00],
            'Y' => &[0x00, 0x00, 0xC3, 0xC3, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00],
            'Z' => &[0x00, 0x00, 0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x70, 0x06, 0x7E, 0x00, 0x00, 0x00, 0x00],
            '0' => &[0x00, 0x00, 0x3C, 0x66, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            '1' => &[0x00, 0x00, 0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00, 0x00, 0x00, 0x00],
            '2' => &[0x00, 0x00, 0x3C, 0x66, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x66, 0x7E, 0x00, 0x00, 0x00, 0x00],
            '3' => &[0x00, 0x00, 0x3C, 0x66, 0x06, 0x06, 0x0C, 0x0C, 0x06, 0x06, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            '4' => &[0x00, 0x00, 0x0C, 0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x0C, 0x0C, 0x1E, 0x00, 0x00, 0x00, 0x00],
            '5' => &[0x00, 0x00, 0x7E, 0x60, 0x60, 0x60, 0x7C, 0x06, 0x06, 0x06, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            '6' => &[0x00, 0x00, 0x1C, 0x30, 0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            '7' => &[0x00, 0x00, 0x7E, 0x06, 0x0C, 0x18, 0x18, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00, 0x00, 0x00, 0x00],
            '8' => &[0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00],
            '9' => &[0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x18, 0x70, 0x00, 0x00, 0x00, 0x00],
            _ => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        }
    }
}

/// Errores gráficos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsError {
    InvalidMode,
    InvalidAddress,
    BufferOverflow,
    UnsupportedBpp,
}

impl fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicsError::InvalidMode => write!(f, "Modo de video inválido"),
            GraphicsError::InvalidAddress => write!(f, "Dirección de framebuffer inválida"),
            GraphicsError::BufferOverflow => write!(f, "Desbordamiento de buffer"),
            GraphicsError::UnsupportedBpp => write!(f, "Bits por píxel no soportados"),
        }
    }
}
