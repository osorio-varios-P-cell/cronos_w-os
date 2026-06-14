//! Driver PS/2 Avanzado - CRONOS W-OS
//! Implementación real de manejo de scancodes e interrupciones.

use core::ptr::{read_volatile, write_volatile};
use crate::serial_println;

const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const PS2_COMMAND: u16 = 0x64;

pub struct Ps2Keyboard;

impl Ps2Keyboard {
    pub fn initialize() {
        unsafe {
            // 1. Deshabilitar dispositivos
            Self::write_command(0xAD); // Disable keyboard
            Self::write_command(0xA7); // Disable mouse

            // 2. Limpiar buffer de salida
            while (Self::read_status() & 0x01) != 0 {
                let _ = Self::read_data();
            }

            // 3. Configurar Controller Configuration Byte
            Self::write_command(0x20); // Read CCB
            let mut ccb = Self::read_data();
            ccb |= 0x01; // Enable interrupt IRQ1
            ccb &= !0x10; // Enable clock

            Self::write_command(0x60); // Write CCB
            Self::write_data(ccb);

            // 4. Habilitar dispositivo
            Self::write_command(0xAE);
            serial_println!("[PS2] Teclado inicializado con IRQ1 activo.");
        }
    }

    #[inline]
    unsafe fn read_status() -> u8 {
        let mut val: u8 = 0;
        core::arch::asm!("in al, dx", in("dx") PS2_STATUS, out("al") val);
        val
    }

    #[inline]
    unsafe fn read_data() -> u8 {
        let mut val: u8 = 0;
        core::arch::asm!("in al, dx", in("dx") PS2_DATA, out("al") val);
        val
    }

    #[inline]
    unsafe fn write_data(val: u8) {
        core::arch::asm!("out dx, al", in("dx") PS2_DATA, in("al") val);
    }

    #[inline]
    unsafe fn write_command(cmd: u8) {
        core::arch::asm!("out dx, al", in("dx") PS2_COMMAND, in("al") cmd);
    }

    /// Manejador de interrupción de teclado (Scancode Parser)
    pub fn handle_interrupt() -> Option<char> {
        unsafe {
            let scancode = Self::read_data();
            // Traducción básica de Set 1 (Ejemplo: 0x1E = 'A')
            match scancode {
                0x1E => Some('a'),
                0x30 => Some('b'),
                0x2E => Some('c'),
                0x1C => Some('\n'), // Enter
                _ => None, // Otros omitidos por brevedad
            }
        }
    }
}
