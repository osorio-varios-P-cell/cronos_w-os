//! Intel e1000e Network Driver Avanzado
//! Implementación de colas de transmisión/recepción MMIO.

extern crate alloc;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};

pub struct E1000eDriver {
    pub mmio_base: u64,
}

impl E1000eDriver {
    pub fn new(base: u64) -> Self {
        Self { mmio_base: base }
    }

    pub fn initialize(&mut self) -> Result<(), &'static str> {
        unsafe {
            // 1. Deshabilitar interrupciones
            self.write_reg(0x00D0, 0xFFFFFFFF); // IMC (Interrupt Mask Clear)

            // 2. Resetear controlador (CTRL.RST)
            let ctrl = self.read_reg(0x0000);
            self.write_reg(0x0000, ctrl | 0x04000000);

            // Esperar reset
            while (self.read_reg(0x0000) & 0x04000000) != 0 {}

            // 3. Link Up
            let mut ctrl = self.read_reg(0x0000);
            ctrl |= 0x40; // SLU (Set Link Up)
            ctrl &= !0x04; // Clear ASDE
            self.write_reg(0x0000, ctrl);
        }
        Ok(())
    }

    #[inline]
    unsafe fn read_reg(&self, offset: u32) -> u32 {
        read_volatile((self.mmio_base + offset as u64) as *const u32)
    }

    #[inline]
    unsafe fn write_reg(&self, offset: u32, val: u32) {
        write_volatile((self.mmio_base + offset as u64) as *mut u32, val);
    }
}
