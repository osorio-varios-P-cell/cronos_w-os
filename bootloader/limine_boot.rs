//! Bootloader Limine para CRONOS OS
//! 
//! Este módulo implementa el bootloader Limine del CronosOS existente,
//! adaptado para funcionar con nuestro exokernel con grafos

use core::panic::PanicInfo;
use limine::{
    BaseRevision,
    bootloader::BootloaderRequest,
    framebuffer::FramebufferRequest,
    hhdm::HhdmRequest,
    memmap::MemmapRequest,
    module::ModuleRequest,
    terminal::TerminalRequest,
};

#[used]
#[link_section = ".limine_reqs"]
static BASE_REVISION: BaseRevision = BaseRevision::new(2);

#[used]
#[link_section = ".limine_reqs"]
static BOOTLOADER_REQUEST: BootloaderRequest = BootloaderRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static TERMINAL_REQUEST: TerminalRequest = TerminalRequest::new();

/// Información del framebuffer
pub struct FramebufferInfo {
    /// Dirección base del framebuffer
    pub address: *mut u8,
    /// Ancho en píxeles
    pub width: u64,
    /// Altura en píxeles
    pub height: u64,
    /// Bits por píxel
    pub bpp: u16,
    /// Stride en bytes
    pub stride: u64,
}

/// Información de memoria
pub struct MemoryInfo {
    /// Memoria total en MB
    pub total_mb: u64,
    /// Regiones de memoria
    pub regions: Vec<MemoryRegion>,
}

/// Región de memoria
pub struct MemoryRegion {
    /// Dirección base
    pub base: u64,
    /// Longitud
    pub length: u64,
    /// Tipo de región
    pub region_type: MemoryRegionType,
    /// Atributos
    pub attributes: u64,
}

/// Tipo de región de memoria
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    /// Usable
    Usable,
    /// Reservada
    Reserved,
    /// ACPI Reclaimable
    AcpiReclaimable,
    /// ACPI Non-volatile
    AcpiNonVolatile,
    /// Memoria mapeada incorrecta
    BadMemory,
    /// Bootloader
    Bootloader,
    /// Kernel/Modules
    KernelModules,
    /// Framebuffer
    Framebuffer,
}

/// Obtiene información del framebuffer
pub fn get_framebuffer_info() -> Option<FramebufferInfo> {
    let response = FRAMEBUFFER_REQUEST.get_response()?;
    
    Some(FramebufferInfo {
        address: response.addr as *mut u8,
        width: response.width,
        height: response.height,
        bpp: response.bpp,
        stride: response.pitch,
    })
}

/// Obtiene información de memoria
pub fn get_memory_info() -> Option<MemoryInfo> {
    let response = MEMMAP_REQUEST.get_response()?;
    
    let mut regions = Vec::new();
    let mut total_mb = 0u64;
    
    for entry in response.entries() {
        let region_type = match entry.typ {
            limine::memmap::MemoryMapEntryType::Usable => MemoryRegionType::Usable,
            limine::memmap::MemoryMapEntryType::Reserved => MemoryRegionType::Reserved,
            limine::memmap::MemoryMapEntryType::AcpiReclaimable => MemoryRegionType::AcpiReclaimable,
            limine::memmap::MemoryMapEntryType::AcpiNonVolatile => MemoryRegionType::AcpiNonVolatile,
            limine::memmap::MemoryMapEntryType::BadMemory => MemoryRegionType::BadMemory,
            limine::memmap::MemoryMapEntryType::Bootloader => MemoryRegionType::Bootloader,
            limine::memmap::MemoryMapEntryType::KernelModules => MemoryRegionType::KernelModules,
            limine::memmap::MemoryMapEntryType::Framebuffer => MemoryRegionType::Framebuffer,
        };
        
        let region = MemoryRegion {
            base: entry.base,
            length: entry.length,
            region_type,
            attributes: entry.attributes,
        };
        
        if region_type == MemoryRegionType::Usable {
            total_mb += region.length / (1024 * 1024);
        }
        
        regions.push(region);
    }
    
    Some(MemoryInfo {
        total_mb,
        regions,
    })
}

/// Obtiene dirección física más alta
pub fn get_highest_address() -> Option<u64> {
    let response = HHDM_REQUEST.get_response()?;
    Some(response.highest_address)
}

/// Obtiene información de módulos
pub fn get_module_info() -> Option<Vec<ModuleInfo>> {
    let response = MODULE_REQUEST.get_response()?;
    
    let mut modules = Vec::new();
    
    for module in response.modules() {
        modules.push(ModuleInfo {
            address: module.base as *mut u8,
            length: module.size,
            path: unsafe { core::ffi::CStr::from_ptr(module.path.as_ptr()) }
                .to_string_lossy()
                .into_owned(),
        });
    }
    
    Some(modules)
}

/// Información de módulo
pub struct ModuleInfo {
    /// Dirección base
    pub address: *mut u8,
    /// Longitud
    pub length: u64,
    /// Path del módulo
    pub path: String,
}

/// Inicializa el terminal
pub fn init_terminal() {
    let response = TERMINAL_REQUEST.get_response();
    
    if let Some(terminal) = response {
        // Configurar terminal para output
        terminal.write("🚀 CRONOS OS - Exokernel con Grafos\n");
        terminal.write("📍 Bootloader: Limine\n");
        terminal.write("🌐 Arquitectura: Exokernel + Grafos + IA Colmena\n");
        terminal.write("✅ Boot completado\n");
    }
}

/// Escribe en el terminal
pub fn terminal_print(text: &str) {
    if let Some(terminal) = TERMINAL_REQUEST.get_response() {
        terminal.write(text);
    }
}

/// Lee del terminal
pub fn terminal_read() -> Option<char> {
    if let Some(terminal) = TERMINAL_REQUEST.get_response() {
        terminal.read()
    } else {
        None
    }
}

/// Verifica si el bootloader es compatible
pub fn verify_bootloader() -> bool {
    BOOTLOADER_REQUEST.get_response().is_some()
}

/// Obtiene versión del bootloader
pub fn get_bootloader_version() -> Option<String> {
    if let Some(response) = BOOTLOADER_REQUEST.get_response() {
        Some(format!("Limine v{}", response.version))
    } else {
        None
    }
}

/// Prepara el sistema para el kernel
pub fn prepare_for_kernel() -> KernelBootInfo {
    let framebuffer_info = get_framebuffer_info();
    let memory_info = get_memory_info();
    let highest_address = get_highest_address();
    let modules = get_module_info();
    
    KernelBootInfo {
        framebuffer: framebuffer_info,
        memory: memory_info,
        highest_address,
        modules,
        bootloader_version: get_bootloader_version(),
    }
}

/// Información de boot para el kernel
pub struct KernelBootInfo {
    /// Información del framebuffer
    pub framebuffer: Option<FramebufferInfo>,
    /// Información de memoria
    pub memory: Option<MemoryInfo>,
    /// Dirección física más alta
    pub highest_address: Option<u64>,
    /// Módulos cargados
    pub modules: Option<Vec<ModuleInfo>>,
    /// Versión del bootloader
    pub bootloader_version: Option<String>,
}

#[panic_handler]
fn panic(info: &PanicInfo) {
    if let Some(terminal) = TERMINAL_REQUEST.get_response() {
        terminal.write("\n💥 KERNEL PANIC!\n");
        terminal.write("📍 Location: ");
        
        if let Some(location) = info.location() {
            terminal.write(&format!("{}:{}\n", location.file(), location.line()));
        }
        
        terminal.write("📝 Message: ");
        terminal.write(info.message());
        terminal.write("\n");
        
        terminal.write("🔄 El sistema se reiniciará en 5 segundos...\n");
        
        // Esperar 5 segundos
        for i in (1..=5).rev() {
            terminal.write(&format!("⏰ {}\n", i));
            // Simple delay
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
        }
        
        terminal.write("🔄 Reiniciando...\n");
    }
    
    // Reiniciar el sistema
    unsafe {
        core::arch::asm!("cli; hlt");
        loop {}
    }
}

/// Función principal del bootloader
#[no_mangle]
pub extern "C" fn limine_main() -> ! {
    // Inicializar terminal primero para poder mostrar mensajes
    init_terminal();
    
    terminal_print("� Iniciando CRONOS OS - Exokernel con Grafos\n");
    
    // Verificar bootloader
    if !verify_bootloader() {
        terminal_print("❌ Error: Bootloader no compatible\n");
        loop {
            unsafe { core::arch::asm!("hlt") };
        }
    }
    
    terminal_print("✅ Bootloader Limine verificado\n");
    
    // Mostrar información del sistema
    if let Some(version) = get_bootloader_version() {
        terminal_print(&format!("📋 Versión: {}\n", version));
    }
    
    // Mostrar información de memoria
    if let Some(memory_info) = get_memory_info() {
        terminal_print(&format!("💾 Memoria total: {} MB\n", memory_info.total_mb));
        terminal_print(&format!("📊 Regiones de memoria: {}\n", memory_info.regions.len()));
    }
    
    // Mostrar información de framebuffer
    if let Some(fb_info) = get_framebuffer_info() {
        terminal_print(&format!("🖥️ Framebuffer: {}x{} @ {}bpp\n", 
            fb_info.width, fb_info.height, fb_info.bpp));
    }
    
    // Mostrar información de módulos
    if let Some(modules) = get_module_info() {
        terminal_print(&format!("📦 Módulos cargados: {}\n", modules.len()));
        for module in modules {
            terminal_print(&format!("   📄 {}\n", module.path));
        }
    }
    
    // Preparar información para el kernel
    let boot_info = prepare_for_kernel();
    
    terminal_print("🔄 Preparando para saltar al kernel...\n");
    
    // Saltar al kernel principal
    unsafe {
        kernel_main(boot_info);
    }
}

/// Función externa para el kernel principal
extern "C" fn kernel_main(boot_info: KernelBootInfo) -> ! {
    // Esta función será implementada en el kernel principal
    // Por ahora, solo mostramos información y entramos en loop
    
    if let Some(terminal) = TERMINAL_REQUEST.get_response() {
        terminal.write("🧠 Kernel CRONOS iniciado\n");
        terminal.write("🌐 Exokernel con grafos listo\n");
        terminal.write("🔄 Entrando en modo operativo...\n");
    }
    
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
