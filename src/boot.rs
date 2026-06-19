//! Boot Module - Limine Bootloader Protocol
//!
//! FASE 15: Implementación del entry point real del kernel usando el protocolo de Limine
//! Este módulo maneja la transición del bootloader al kernel, parsing de información
//! del bootloader (memoria, framebuffer, ACPI), y setup inicial del sistema.

#![no_std]

use core::panic::PanicInfo;
use limine::{
    BaseRevision, RequestsEndMarker, RequestsStartMarker,
    request::*,
};
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

// Importar desde main.rs (crate root)
use crate::{serial_print_hex, serial_print_dec, serial_writer};

/// FASE 15: Marcadores de inicio y fin de requests
#[used]
#[link_section = ".requests_start_marker"]
static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

/// Revisión base de Limine
/// Usamos revisión 2 para que Limine mapee automáticamente 0->4GiB al HHDM
#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(2);

/// Request de framebuffer
#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Request de HHDM (Higher Half Direct Map)
#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Request de mapa de memoria
#[used]
#[link_section = ".requests"]
static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

/// Request de módulos
#[used]
#[link_section = ".requests"]
static MODULE_REQUEST: ModulesRequest = ModulesRequest::new();

/// Request de RSDP (ACPI)
#[used]
#[link_section = ".requests"]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// FASE 15: Marcador de fin de requests
#[used]
#[link_section = ".requests_end_marker"]
static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

/// Información de boot parseada del bootloader
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UsableRegion {
    pub base: u64,
    pub length: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {
    /// Memoria física disponible
    pub available_memory: u64,
    /// Memoria física total
    pub total_memory: u64,
    /// Dirección base del HHDM
    pub hhdm_offset: u64,
    /// Framebuffer si está disponible
    pub framebuffer: Option<FramebufferInfo>,
    /// RSDP para ACPI
    pub rsdp: Option<u64>,
    /// Regiones de memoria usables para el heap
    pub usable_regions: [Option<UsableRegion>; 16],
}

/// Información del framebuffer
#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    /// Dirección física del framebuffer
    pub address: u64,
    /// Ancho en píxeles
    pub width: u64,
    /// Alto en píxeles
    pub height: u64,
    /// Bits por píxel
    pub bpp: u64,
    /// Pitch (bytes por línea)
    pub pitch: u64,
}

impl BootInfo {
    /// Parsear la información del bootloader
    pub unsafe fn from_limine() -> Self {
        // FASE 15: Usar la nueva API de limine 0.6.x
        // Los requests se usan directamente, el bootloader los llena
        
        // Calcular memoria disponible y total
        let mut available_memory = 0u64;
        let mut total_memory = 0u64;

        // FASE 15: La nueva API usa .response() directamente
        if let Some(memmap) = MEMMAP_REQUEST.response() {
            // Bloqueo total de interrupciones para evitar reentrancia
            unsafe {
                core::arch::asm!("cli");
            }
            
            // Bloqueo manual del Mutex para evitar interleaving durante el bucle
            use core::fmt::Write;
            let mut writer = crate::serial_writer::SERIAL_WRITER.lock();
            
            writeln!(writer, "Memory map entries:").unwrap();
            
            for entry in memmap.entries() {
                total_memory += entry.length;
                
                // PROTOCOLO DE INTEGRIDAD DE DATOS: Copiar entrada a estructura local (stack)
                // Esto previene que la memoria del mapa sea sobrescrita mientras imprimimos
                #[repr(C)]
                struct LocalEntry {
                    base: u64,
                    length: u64,
                    type_: u64,
                }
                
                let local_entry = LocalEntry {
                    base: entry.base,
                    length: entry.length,
                    type_: entry.type_ as u64,
                };
                
                // Escribir desde la copia local, no desde el mapa original
                write!(writer, "  Base: 0x{:x} ", local_entry.base).unwrap();
                write!(writer, "Length: 0x{:x} ", local_entry.length).unwrap();
                write!(writer, "Type: {} | END\n", local_entry.type_).unwrap();
                
                if local_entry.type_ == limine::memmap::MEMMAP_USABLE as u64 {
                    available_memory += local_entry.length;
                }
            }
            
            // Centinela crítico: Esto debe imprimirse si el bucle sale correctamente
            writeln!(writer, "DEBUG: Bucle finalizado correctamente.").unwrap();
            // Añadir terminador de línea para garantizar el fin de bloque
            writeln!(writer, "").unwrap();
            
            // El Mutex se libera automáticamente aquí al salir del scope
            
            // NO habilitar interrupciones aquí. Se hará en kernel_main_impl
            // cuando el sistema esté totalmente listo.
        }
        
        // Centinela después del bucle
        // crate::serial_println!("DEBUG: Fuera del bucle del mapa de memoria");
        
        // Barrera explícita para asegurar que el mapa de memoria termine antes de continuar
        // crate::serial_println!("--- MAPA TERMINADO ---");
        
        // SOLO SI LLEGA HASTA AQUÍ, sigues con el siguiente paso:
        // crate::serial_println!("DEBUG: Parseando framebuffer");
        let framebuffer = if let Some(fb) = FRAMEBUFFER_REQUEST.response() {
            if let Some(primary) = fb.framebuffers().first() {
                Some(FramebufferInfo {
                    address: primary.address() as u64,
                    width: primary.width,
                    height: primary.height,
                    bpp: primary.bpp as u64,
                    pitch: primary.pitch,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Parsear RSDP
        // crate::serial_println!("DEBUG: Parseando RSDP");
        let rsdp = if let Some(rsdp) = RSDP_REQUEST.response() {
            Some(rsdp.address as u64)
        } else {
            None
        };

        // Parsear HHDM offset
        // crate::serial_println!("DEBUG: Parseando HHDM offset");
        let hhdm_offset = if let Some(hhdm) = HHDM_REQUEST.response() {
            hhdm.offset
        } else {
            0
        };

        let mut usable_regions = [None; 16];
        let mut region_idx = 0;
        // crate::serial_println!("DEBUG: Parseando usable regions");
        if let Some(memmap) = MEMMAP_REQUEST.response() {
            for entry in memmap.entries() {
                if entry.type_ == limine::memmap::MEMMAP_USABLE && region_idx < 16 {
                    usable_regions[region_idx] = Some(UsableRegion {
                        base: entry.base,
                        length: entry.length,
                    });
                    region_idx += 1;
                }
            }
        }
        
        // crate::serial_println!("DEBUG: Creando BootInfo");

        BootInfo {
            available_memory,
            total_memory,
            hhdm_offset,
            framebuffer,
            rsdp,
            usable_regions,
        }
    }
}

/// BootInfo global para evitar problemas de optimización del compilador
#[no_mangle]
static mut BOOT_INFO_STATIC: Option<BootInfo> = None;

/// Entry point del kernel llamado por Limine
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'K', options(nostack, nomem));
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'\r', options(nostack, nomem));
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'\n', options(nostack, nomem));
    }

    // Parsear información del bootloader y saltar al kernel principal
    // Usar static para evitar que el compilador optimice mal la referencia
    unsafe {
        BOOT_INFO_STATIC = Some(unsafe { BootInfo::from_limine() });
    }
    let boot_info_ref = unsafe { BOOT_INFO_STATIC.as_ref().unwrap() };
    crate::kernel_main_impl(boot_info_ref);
}
