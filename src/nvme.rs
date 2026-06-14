//! NVMe Driver Avanzado - CRONOS W-OS
//! Implementación MMIO con punteros volátiles para hardware real.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};

pub struct NvmeController {
    pub mmio_base: u64,
    pub enabled: bool,
    pub max_queues: u16,
}

impl NvmeController {
    pub fn new(bus: u8, dev: u8) -> Self {
        // En un sistema real, mmio_base vendría del BAR de PCI
        Self {
            mmio_base: 0,
            enabled: false,
            max_queues: 0,
        }
    }

    pub fn initialize(&mut self, base_addr: u64) -> Result<(), String> {
        self.mmio_base = base_addr;
        
        unsafe {
            // 1. Leer Capabilities (CAP) - 64 bits
            let cap = self.read_reg64(0x00);
            self.max_queues = ((cap >> 16) & 0xFFFF) as u16;

            // 2. Configurar Controller Configuration (CC)
            // Bit 0 = EN (Enable). Ponemos a 0 para resetear.
            self.write_reg32(0x14, 0);
            
            // Esperar a que el controlador confirme el reset (CSTS.RDY)
            while (self.read_reg32(0x1C) & 0x01) != 0 {}

            // 3. Activar Controlador
            self.write_reg32(0x14, 0x00460001); // Enable + I/O Command Sets

            // Esperar a Ready
            while (self.read_reg32(0x1C) & 0x01) == 0 {}
        }

        self.enabled = true;
        Ok(())
    }

    #[inline]
    unsafe fn read_reg32(&self, offset: u32) -> u32 {
        read_volatile((self.mmio_base + offset as u64) as *const u32)
    }

    #[inline]
    unsafe fn write_reg32(&self, offset: u32, val: u32) {
        write_volatile((self.mmio_base + offset as u64) as *mut u32, val);
    }

    #[inline]
    unsafe fn read_reg64(&self, offset: u32) -> u64 {
        read_volatile((self.mmio_base + offset as u64) as *const u64)
    }
}
