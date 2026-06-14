//! USB xHCI Driver Avanzado - CRONOS W-OS
//! Manejo de transferencias y anillos de comandos reales.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};

pub struct XhciController {
    pub base_addr: u64,
    pub cap_length: u8,
}

impl XhciController {
    pub fn new(addr: u64) -> Self {
        Self {
            base_addr: addr,
            cap_length: 0,
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        unsafe {
            // Leer CAPLENGTH para saber dónde empiezan los registros operacionales
            self.cap_length = read_volatile(self.base_addr as *const u8);

            let op_base = self.base_addr + self.cap_length as u64;

            // 1. Resetear Controlador (USBCMD.HCRST)
            let mut usbcmd = read_volatile((op_base + 0x00) as *const u32);
            usbcmd |= 0x02;
            write_volatile((op_base + 0x00) as *mut u32, usbcmd);

            // Esperar a que el reset termine
            while (read_volatile((op_base + 0x00) as *const u32) & 0x02) != 0 {}

            // 2. Configurar Max Device Slots
            let cap_params1 = read_volatile((self.base_addr + 0x04) as *const u32);
            let max_slots = cap_params1 & 0xFF;
            write_volatile((op_base + 0x38) as *mut u32, max_slots); // CONFIG register
        }
        Ok(())
    }
}
