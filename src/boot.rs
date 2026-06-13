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

/// FASE 15: Marcadores de inicio y fin de requests
#[unsafe(link_section = ".requests_start")]
static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

/// Revisión base de Limine
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Request de framebuffer
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Request de HHDM (Higher Half Direct Map)
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Request de mapa de memoria
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

/// Request de módulos
#[unsafe(link_section = ".requests")]
static MODULE_REQUEST: ModulesRequest = ModulesRequest::new();

/// Request de RSDP (ACPI)
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// FASE 15: Marcador de fin de requests
#[unsafe(link_section = ".requests_end")]
static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

/// Información de boot parseada del bootloader
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
            for entry in memmap.entries() {
                total_memory += entry.length;
                if entry.type_ == limine::memmap::MEMMAP_USABLE {
                    available_memory += entry.length;
                }
            }
        }

        // Parsear framebuffer si está disponible
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
        let rsdp = if let Some(rsdp) = RSDP_REQUEST.response() {
            Some(rsdp.address as u64)
        } else {
            None
        };

        // Parsear HHDM offset
        let hhdm_offset = if let Some(hhdm) = HHDM_REQUEST.response() {
            hhdm.offset
        } else {
            0
        };

        BootInfo {
            available_memory,
            total_memory,
            hhdm_offset,
            framebuffer,
            rsdp,
        }
    }
}

/// Entry point del kernel llamado por Limine
#[no_mangle]
extern "C" fn _start() -> ! {
    // FASE 15: Setup inicial de stack (el bootloader ya configuró el stack)
    unsafe {
        // Verificar stack pointer
        let stack_ptr: *mut u8;
        core::arch::asm!(
            "mov {}, rsp",
            out(reg) stack_ptr,
            options(nomem, nostack)
        );
        
        serial_print("Stack pointer: 0x");
        serial_print_hex(stack_ptr as u64);
        serial_println("");
    }

    // FASE 15: Parsear información del bootloader
    let boot_info = unsafe { BootInfo::from_limine() };

    // FASE 15: Imprimir información de boot
    unsafe {
        serial_println("CRONOS W-OS v2.0 - Booting with Limine");
        serial_print("Available memory: ");
        serial_print_dec(boot_info.available_memory / 1024 / 1024);
        serial_println(" MB");
        serial_print("Total memory: ");
        serial_print_dec(boot_info.total_memory / 1024 / 1024);
        serial_println(" MB");
        serial_print("HHDM offset: 0x");
        serial_print_hex(boot_info.hhdm_offset);
        serial_println("");

        if let Some(ref fb) = boot_info.framebuffer {
            serial_print("Framebuffer: ");
            serial_print_dec(fb.width);
            serial_print("x");
            serial_print_dec(fb.height);
            serial_print(" @ ");
            serial_print_dec(fb.bpp);
            serial_println(" bpp");
        } else {
            serial_println("No framebuffer available");
        }

        if let Some(rsdp) = boot_info.rsdp {
            serial_print("RSDP at: 0x");
            serial_print_hex(rsdp);
            serial_println("");
        } else {
            serial_println("No RSDP available");
        }
    }

    // FASE 15: Setup de heap y memoria
    unsafe {
        // Inicializar el heap global
        crate::memory::init_heap();
        
        serial_println("Heap initialized");
        
        // FASE 15: Configurar MemoryManager con información de Limine
        use crate::memory::{MemoryManager, MemoryRegion, MemoryRegionType, MemoryRange};
        
        let mut memory_manager = MemoryManager::new_with_params(
            boot_info.hhdm_offset,
            &[]
        );
        
        serial_print("Memory manager initialized with HHDM offset: 0x");
        serial_print_hex(boot_info.hhdm_offset);
        serial_println("");
    }

    // FASE 15: Llamar al kernel main con la información de boot
    extern "C" {
        fn kernel_main(boot_info: &BootInfo) -> !;
    }

    unsafe {
        kernel_main(&boot_info);
    }
}

/// Serial print para debugging
unsafe fn serial_print(s: &str) {
    // FASE 15: Implementación básica de serial output
    let port = 0x3f8u16;
    for byte in s.bytes() {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") byte,
            options(nomem, nostack)
        );
    }
}

/// Serial print con newline
unsafe fn serial_println(s: &str) {
    serial_print(s);
    serial_print("\r\n");
}

/// Serial print hexadecimal
unsafe fn serial_print_hex(mut value: u64) {
    let port = 0x3f8u16;
    let mut buffer = [0u8; 16];
    let mut i = 0;
    
    if value == 0 {
        serial_print("0");
        return;
    }
    
    while value > 0 {
        let digit = (value & 0xF) as u8;
        buffer[i] = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + (digit - 10)
        };
        value >>= 4;
        i += 1;
    }
    
    // Imprimir en orden inverso
    for j in (0..i).rev() {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") buffer[j],
            options(nomem, nostack)
        );
    }
}

/// Serial print decimal
unsafe fn serial_print_dec(mut value: u64) {
    let port = 0x3f8u16;
    let mut buffer = [0u8; 20];
    let mut i = 0;
    
    if value == 0 {
        serial_print("0");
        return;
    }
    
    while value > 0 {
        let digit = (value % 10) as u8;
        buffer[i] = b'0' + digit;
        value /= 10;
        i += 1;
    }
    
    // Imprimir en orden inverso
    for j in (0..i).rev() {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") buffer[j],
            options(nomem, nostack)
        );
    }
}
