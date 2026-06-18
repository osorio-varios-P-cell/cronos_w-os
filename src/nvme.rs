//! NVMe Driver Avanzado - CRONOS W-OS
//! Implementación MMIO con punteros volátiles para hardware real.

extern crate alloc;
use alloc::string::String;
use core::ptr::{read_volatile, write_volatile};

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct NvmeCommand {
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub nsid: u32,
    pub reserved: [u64; 2],
    pub metadata_ptr: u64,
    pub data_ptr: u64,
    pub command_specific: [u32; 6],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct NvmeCompletion {
    pub command_specific: u32,
    pub reserved: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub command_id: u16,
    pub status: u16,
}

pub struct NvmeController {
    pub mmio_base: u64,
    pub enabled: bool,
    pub max_queues: u16,
    pub admin_sq_phys: u64,
    pub admin_cq_phys: u64,
    pub doorbell_stride: u32,
}

impl NvmeController {
    pub fn new(_bus: u8, _dev: u8) -> Self {
        Self {
            mmio_base: 0,
            enabled: false,
            max_queues: 0,
            admin_sq_phys: 0,
            admin_cq_phys: 0,
            doorbell_stride: 0,
        }
    }

    pub fn initialize(&mut self, base_addr: u64) -> Result<(), String> {
        self.mmio_base = base_addr;
        
        unsafe {
            // 1. Leer Capabilities (CAP)
            let cap = self.read_reg64(0x00);
            self.max_queues = ((cap >> 16) & 0xFFFF) as u16;
            self.doorbell_stride = 1 << (2 + ((cap >> 32) & 0xF) as u32);

            // 2. Reset del controlador
            self.write_reg32(0x14, 0); // CC.EN = 0
            while (self.read_reg32(0x1C) & 0x01) != 0 {} // Esperar CSTS.RDY == 0

            // 3. Configurar colas de administración (Admin Queues)
            // Aquí deberíamos asignar memoria física real.
            // Como placeholder avanzado, usamos una dirección fija simulada para el mapa MMIO.
            self.admin_sq_phys = 0x1000000; // 16MB (Placeholder para frame real)
            self.admin_cq_phys = 0x1001000; // 16MB + 4KB

            self.write_reg64(0x28, self.admin_sq_phys); // ASQ
            self.write_reg64(0x30, self.admin_cq_phys); // ACQ

            // 4. Configurar tamaños de cola (AQA)
            // 63 comandos en cada cola (0-indexed, así que 63 significa 64 entradas)
            self.write_reg32(0x24, (63 << 16) | 63);

            // 5. Activar Controlador (CC.EN = 1)
            // MPS=0 (4KB), CSS=0 (NVM), AMS=0 (RR)
            self.write_reg32(0x14, 0x00460001);

            // Esperar a Ready (CSTS.RDY == 1)
            while (self.read_reg32(0x1C) & 0x01) == 0 {}
        }

        self.enabled = true;
        Ok(())
    }

    /// Identificar el controlador (Identify Controller)
    pub fn identify(&self) -> Result<String, String> {
        if !self.enabled { return Err(String::from("NVMe no habilitado")); }

        // En una implementación real, aquí llenaríamos un NvmeCommand
        // con opcode 0x06 (Identify), lo pondríamos en ASQ y pulsaríamos el Doorbell.
        Ok(String::from("CRONOS NVMe: Identificación exitosa (Simulada)"))
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

    #[inline]
    unsafe fn write_reg64(&self, offset: u32, val: u64) {
        write_volatile((self.mmio_base + offset as u64) as *mut u64, val);
    }
}
