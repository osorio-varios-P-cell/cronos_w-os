//! PS/2 Keyboard/Mouse Driver Module
//! 
//! This module implements the PS/2 port driver for keyboard and mouse devices.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Código de scancode
#[derive(Debug, Clone, Copy)]
pub struct Scancode {
    /// Código del scancode
    pub code: u8,
    /// Si es un break code (tecla liberada)
    pub break_code: bool,
}

impl Scancode {
    /// Crear nuevo scancode
    pub fn new(code: u8, break_code: bool) -> Self {
        Self { code, break_code }
    }
}

/// Tecla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Sin tecla
    None,
    /// Escape
    Escape,
    /// Dígito 1-9
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    /// Teclas especiales
    Minus,
    Equals,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Enter,
    LeftControl,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    LeftShift,
    Backslash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
    KeypadAsterisk,
    LeftAlt,
    Space,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    NumLock,
    ScrollLock,
    Keypad7,
    Keypad8,
    Keypad9,
    KeypadMinus,
    Keypad4,
    Keypad5,
    Keypad6,
    KeypadPlus,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad0,
    KeypadPeriod,
    F11,
    F12,
}

/// Evento de teclado
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// Tecla presionada/liberada
    pub key: Key,
    /// Si es un break code (tecla liberada)
    pub released: bool,
    /// Modificadores activos
    pub modifiers: Modifiers,
}

/// Modificadores de teclado
#[derive(Debug, Clone, Copy)]
pub struct Modifiers {
    /// Shift izquierdo
    pub left_shift: bool,
    /// Shift derecho
    pub right_shift: bool,
    /// Control izquierdo
    pub left_control: bool,
    /// Alt izquierdo
    pub left_alt: bool,
    /// Caps Lock
    pub caps_lock: bool,
    /// Num Lock
    pub num_lock: bool,
}

impl Modifiers {
    /// Crear nuevos modificadores
    pub fn new() -> Self {
        Self {
            left_shift: false,
            right_shift: false,
            left_control: false,
            left_alt: false,
            caps_lock: false,
            num_lock: false,
        }
    }

    /// Verificar si shift está presionado
    pub fn is_shift(&self) -> bool {
        self.left_shift || self.right_shift
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::new()
    }
}

/// Paquete de mouse
#[derive(Debug, Clone, Copy)]
pub struct MousePacket {
    /// Botón izquierdo
    pub left_button: bool,
    /// Botón derecho
    pub right_button: bool,
    /// Botón medio
    pub middle_button: bool,
    /// Movimiento X
    pub x_movement: i8,
    /// Movimiento Y
    pub y_movement: i8,
}

/// Evento de mouse
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// Paquete de mouse
    pub packet: MousePacket,
    /// Posición X absoluta
    pub x: u16,
    /// Posición Y absoluta
    pub y: u16,
}

/// Controlador PS/2
pub struct Ps2Controller {
    /// Puerto de datos del teclado (0x60)
    pub keyboard_data_port: u16,
    /// Puerto de comando del teclado (0x64)
    pub keyboard_command_port: u16,
    /// Puerto de datos del mouse (0x60)
    pub mouse_data_port: u16,
    /// Puerto de comando del mouse (0x64)
    pub mouse_command_port: u16,
    /// Modificadores actuales
    pub modifiers: Modifiers,
    /// Habilitado
    pub enabled: bool,
    /// Teclado habilitado
    pub keyboard_enabled: bool,
    /// Mouse habilitado
    pub mouse_enabled: bool,
}

impl Ps2Controller {
    /// Crear nuevo controlador
    pub fn new() -> Self {
        Self {
            keyboard_data_port: 0x60,
            keyboard_command_port: 0x64,
            mouse_data_port: 0x60,
            mouse_command_port: 0x64,
            modifiers: Modifiers::new(),
            enabled: false,
            keyboard_enabled: false,
            mouse_enabled: false,
        }
    }

    /// Inicializar controlador
    pub fn initialize(&mut self) -> Result<(), String> {
        // Resetear controlador PS/2
        self.reset_controller()?;
        
        // Inicializar teclado
        self.init_keyboard()?;
        
        // Inicializar mouse
        self.init_mouse()?;
        
        self.enabled = true;
        Ok(())
    }

    /// Resetear controlador PS/2
    fn reset_controller(&mut self) -> Result<(), String> {
        // En un sistema real, esto enviaría comandos de reset al controlador PS/2
        Ok(())
    }

    /// Inicializar teclado
    fn init_keyboard(&mut self) -> Result<(), String> {
        // En un sistema real, esto configuraría el teclado PS/2
        self.keyboard_enabled = true;
        Ok(())
    }

    /// Inicializar mouse
    fn init_mouse(&mut self) -> Result<(), String> {
        // En un sistema real, esto configuraría el mouse PS/2
        self.mouse_enabled = true;
        Ok(())
    }

    /// Leer byte del puerto de datos del teclado
    unsafe fn read_keyboard_data(&self) -> u8 {
        // En un sistema real, esto leería del puerto 0x60
        0
    }

    /// Escribir byte al puerto de datos del teclado
    unsafe fn write_keyboard_data(&self, value: u8) {
        // En un sistema real, esto escribiría al puerto 0x60
    }

    /// Leer byte del puerto de comando del teclado
    unsafe fn read_keyboard_command(&self) -> u8 {
        // En un sistema real, esto leería del puerto 0x64
        0
    }

    /// Escribir byte al puerto de comando del teclado
    unsafe fn write_keyboard_command(&self, value: u8) {
        // En un sistema real, esto escribiría al puerto 0x64
    }

    /// Leer byte del puerto de datos del mouse
    unsafe fn read_mouse_data(&self) -> u8 {
        // En un sistema real, esto leería del puerto 0x60
        0
    }

    /// Escribir byte al puerto de datos del mouse
    unsafe fn write_mouse_data(&self, value: u8) {
        // En un sistema real, esto escribiría al puerto 0x60
    }

    /// Leer scancode del teclado
    pub fn read_scancode(&mut self) -> Option<Scancode> {
        if !self.keyboard_enabled {
            return None;
        }
        
        // En un sistema real, esto leería un scancode del teclado
        let code = unsafe { self.read_keyboard_data() };
        let break_code = (code & 0x80) != 0;
        let scancode_code = code & 0x7F;
        
        Some(Scancode::new(scancode_code, break_code))
    }

    /// Traducir scancode a tecla
    pub fn scancode_to_key(&self, scancode: Scancode) -> Key {
        // En un sistema real, esto traduciría el scancode a una tecla
        // Para este ejemplo, usamos una traducción simple
        match scancode.code {
            0x01 => Key::Escape,
            0x02 => Key::Digit1,
            0x03 => Key::Digit2,
            0x04 => Key::Digit3,
            0x05 => Key::Digit4,
            0x06 => Key::Digit5,
            0x07 => Key::Digit6,
            0x08 => Key::Digit7,
            0x09 => Key::Digit8,
            0x0A => Key::Digit9,
            0x0B => Key::Digit0,
            0x0E => Key::Backspace,
            0x0F => Key::Tab,
            0x10 => Key::Q,
            0x11 => Key::W,
            0x12 => Key::E,
            0x13 => Key::R,
            0x14 => Key::T,
            0x15 => Key::Y,
            0x16 => Key::U,
            0x17 => Key::I,
            0x18 => Key::O,
            0x19 => Key::P,
            0x1A => Key::LeftBracket,
            0x1B => Key::RightBracket,
            0x1C => Key::Enter,
            0x1D => Key::LeftControl,
            0x1E => Key::A,
            0x1F => Key::S,
            0x20 => Key::D,
            0x21 => Key::F,
            0x22 => Key::G,
            0x23 => Key::H,
            0x24 => Key::J,
            0x25 => Key::K,
            0x26 => Key::L,
            0x27 => Key::Semicolon,
            0x28 => Key::Apostrophe,
            0x2A => Key::LeftShift,
            0x2B => Key::Backslash,
            0x2C => Key::Z,
            0x2D => Key::X,
            0x2E => Key::C,
            0x2F => Key::V,
            0x30 => Key::B,
            0x31 => Key::N,
            0x32 => Key::M,
            0x33 => Key::Comma,
            0x34 => Key::Period,
            0x35 => Key::Slash,
            0x36 => Key::RightShift,
            0x37 => Key::KeypadAsterisk,
            0x38 => Key::LeftAlt,
            0x39 => Key::Space,
            0x3A => Key::CapsLock,
            _ => Key::None,
        }
    }

    /// Leer evento de teclado
    pub fn read_keyboard_event(&mut self) -> Option<KeyboardEvent> {
        let scancode = self.read_scancode()?;
        let key = self.scancode_to_key(scancode);
        
        // Actualizar modificadores
        self.update_modifiers(key, scancode.break_code);
        
        Some(KeyboardEvent {
            key,
            released: scancode.break_code,
            modifiers: self.modifiers,
        })
    }

    /// Actualizar modificadores
    fn update_modifiers(&mut self, key: Key, released: bool) {
        match key {
            Key::LeftShift => self.modifiers.left_shift = !released,
            Key::RightShift => self.modifiers.right_shift = !released,
            Key::LeftControl => self.modifiers.left_control = !released,
            Key::LeftAlt => self.modifiers.left_alt = !released,
            Key::CapsLock if !released => self.modifiers.caps_lock = !self.modifiers.caps_lock,
            Key::NumLock if !released => self.modifiers.num_lock = !self.modifiers.num_lock,
            _ => {}
        }
    }

    /// Leer paquete de mouse
    pub fn read_mouse_packet(&mut self) -> Option<MousePacket> {
        if !self.mouse_enabled {
            return None;
        }
        
        // En un sistema real, esto leería 3 bytes del mouse
        let byte1 = unsafe { self.read_mouse_data() };
        let byte2 = unsafe { self.read_mouse_data() };
        let byte3 = unsafe { self.read_mouse_data() };
        
        Some(MousePacket {
            left_button: (byte1 & 0x01) != 0,
            right_button: (byte1 & 0x02) != 0,
            middle_button: (byte1 & 0x04) != 0,
            x_movement: byte2 as i8,
            y_movement: byte3 as i8,
        })
    }

    /// Leer evento de mouse
    pub fn read_mouse_event(&mut self, x: u16, y: u16) -> Option<MouseEvent> {
        let packet = self.read_mouse_packet()?;
        
        Some(MouseEvent {
            packet,
            x,
            y,
        })
    }

    /// Obtener modificadores actuales
    pub fn get_modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Verificar si está habilitado
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Verificar si el teclado está habilitado
    pub fn is_keyboard_enabled(&self) -> bool {
        self.keyboard_enabled
    }

    /// Verificar si el mouse está habilitado
    pub fn is_mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("PS/2 Controller Status\n");
        report.push_str("======================\n\n");
        
        report.push_str(&format!("Enabled: {}\n", self.enabled));
        report.push_str(&format!("Keyboard Enabled: {}\n", self.keyboard_enabled));
        report.push_str(&format!("Mouse Enabled: {}\n", self.mouse_enabled));
        
        report.push_str("\nModifiers:\n");
        report.push_str(&format!("  Left Shift: {}\n", self.modifiers.left_shift));
        report.push_str(&format!("  Right Shift: {}\n", self.modifiers.right_shift));
        report.push_str(&format!("  Left Control: {}\n", self.modifiers.left_control));
        report.push_str(&format!("  Left Alt: {}\n", self.modifiers.left_alt));
        report.push_str(&format!("  Caps Lock: {}\n", self.modifiers.caps_lock));
        report.push_str(&format!("  Num Lock: {}\n", self.modifiers.num_lock));
        
        report
    }
}

impl Default for Ps2Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades para PS/2
pub struct Ps2Utils;

impl Ps2Utils {
    /// Verificar soporte de PS/2 en el sistema
    pub fn check_ps2_support() -> bool {
        // En un sistema real, esto verificaría si hay dispositivos PS/2 disponibles
        true // Simulado
    }

    /// Verificar si hay teclado PS/2
    pub fn check_keyboard_present() -> bool {
        // En un sistema real, esto verificaría si hay un teclado PS/2
        true // Simulado
    }

    /// Verificar si hay mouse PS/2
    pub fn check_mouse_present() -> bool {
        // En un sistema real, esto verificaría si hay un mouse PS/2
        true // Simulado
    }
}
