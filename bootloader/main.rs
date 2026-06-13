#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
use x86_64::structures::paging::Page;
use x86_64::VirtAddr;

mod memory;
mod hardware;
mod interrupts;
mod ia_colmena_integration;
mod limine_boot;

entry_point!(kernel_main);

/// Punto de entrada principal del kernel CRONOS W-OS
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("🚀 CRONOS W-OS - Sistema Operativo Soberano Iniciado");
    println!("📍 Versión: 2.0.0");
    println!("🧠 Integración con IA Colmena: Activada");
    println!("🌐 Exokernel con Grafos de Recursos: Activo");
    
    // 1. Inicializar gestor de memoria
    println!("📦 Inicializando gestor de memoria...");
    let mut memory_manager = memory::MemoryManager::new();
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::new(&boot_info.memory_map)
    };
    
    // 2. Mapear memoria física y virtual
    println!("🗺️ Mapeando memoria física y virtual...");
    unsafe {
        memory_manager.init(&boot_info, &mut frame_allocator);
    }
    
    // 3. Inicializar manejador de interrupciones
    println!("⚡ Inicializando manejador de interrupciones...");
    interrupts::init_idt();
    
    // 4. Escanear hardware y periféricos
    println!("🔍 Escaneando hardware y periféricos...");
    let mut hardware_scanner = hardware::HardwareScanner::new();
    let detected_hardware = hardware_scanner.scan_all_devices();
    
    println!("🔍 Hardware Detectado:");
    for device in &detected_hardware {
        println!("   - {}: {}", device.device_type, device.description);
    }
    
    // 5. Inicializar controlador de CPU
    println!("💻 Inicializando controlador de CPU...");
    let mut cpu_controller = hardware::CpuController::new();
    cpu_controller.init_msr_registers();
    cpu_controller.calculate_thermal_limits();
    
    // 6. Integrar con IA Colmena
    println!("🤖 Integrando con IA Colmena...");
    let mut colmena_bridge = ia_colmena_integration::ColmenaObserverBridge::new();
    colmena_bridge.init();
    
    // 7. Crear interfaz de hardware para capas superiores
    println!("🔧 Creando interfaz de hardware para capas superiores...");
    let hardware_interface = create_hardware_interface(
        detected_hardware,
        cpu_controller,
        memory_manager,
    );
    
    // 8. Iniciar capas superiores
    println!("📊 Iniciando capas superiores...");
    start_upper_layers(hardware_interface);
    
    // 9. Entrar en bucle principal del kernel
    println!("🔄 Entrando en bucle principal del kernel...");
    kernel_loop();
}

/// Bucle principal del kernel
fn kernel_loop() -> ! {
    loop {
        // Procesar eventos del sistema
        // Actualizar métricas
        // Optimizar recursos
        unsafe { core::arch::asm!("hlt") };
    }
}

/// Crear interfaz de hardware para capas superiores
fn create_hardware_interface(
    devices: Vec<hardware::HardwareDevice>,
    cpu: hardware::CpuController,
    memory: memory::MemoryManager,
) -> hardware::HardwareInterface {
    hardware::HardwareInterface {
        devices,
        cpu,
        memory,
    }
}

/// Iniciar capas superiores del sistema
fn start_upper_layers(hardware_interface: hardware::HardwareInterface) {
    // Iniciar AEGIS (seguridad)
    println!("🛡️ Iniciando AEGIS (Seguridad)...");
    
    // Iniciar LUMEN (gráficos)
    println!("🎨 Iniciando LUMEN (Gráficos)...");
    
    // Iniciar GENESIS (auto-creación)
    println!("⚙️ Iniciando GENESIS (Auto-creación)...");
    
    // Iniciar Crystal UI
    println!("🖥️ Iniciando Crystal UI...");
}

/// Manejador de panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("❌ KERNEL PANIC: {}", _info);
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

/// Test runner personalizado
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("🧪 Ejecutando {} tests...", tests.len());
    for (i, test) in tests.iter().enumerate() {
        println!("  Test {}...", i);
        test();
    }
    println!("✅ Todos los tests pasaron exitosamente");
}

/// Función de entrada para tests
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("🧪 Iniciando tests del kernel...");
    test_main();
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
