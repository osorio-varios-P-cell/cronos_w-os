#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod exokernel_graph;
pub mod graph_memory;
pub mod hardware_adaptation;
pub mod universal_driver;
pub mod scheduler;
pub mod filesystem;
pub mod networking;
pub mod security;
pub mod graphics;
pub mod genesis;
pub mod ia_colmena;
pub mod crystal_ui;
pub mod acpi;
pub mod firmware_loader;
pub mod vga_buffer;

/// Punto de entrada del kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("🚀 CRONOS W-OS - Sistema Operativo Soberano Iniciado");
    println!("📍 Versión: 2.0.0");
    println!("🧠 Integración con IA Colmena: Activada");
    println!("🌐 Exokernel con Grafos de Recursos: Activo");
    
    kernel_main();
}

/// Función principal del kernel
pub fn kernel_main() -> ! {
    // Inicializar componentes del kernel
    println!("📦 Inicializando componentes del kernel...");
    
    // Inicializar exokernel con grafos
    println!("🌐 Inicializando Exokernel con Grafos...");
    let exokernel = exokernel_graph::ExokernelGraphSystem::new();
    
    // Inicializar sistema de memoria basado en grafos
    println!("💾 Inicializando Graph Memory System...");
    let graph_memory = graph_memory::GraphMemorySystem::new();
    
    // Inicializar sistema de adaptación de hardware
    println!("🔧 Inicializando Hardware Adaptation System...");
    let hardware_adaptation = hardware_adaptation::HardwareAdaptationSystem::new();
    
    // Inicializar sistema de drivers universales
    println!("🔌 Inicializando Universal Driver System...");
    let universal_driver = universal_driver::UniversalDriverSystem::new();
    
    // Inicializar scheduler de procesos
    println!("⏱️ Inicializando Scheduler de Procesos...");
    let scheduler = scheduler::ProcessScheduler::new();
    
    // Inicializar sistema de archivos
    println!("📁 Inicializando Sistema de Archivos...");
    let filesystem = filesystem::FileSystem::new();
    
    // Inicializar stack de red
    println!("🌐 Inicializando Stack de Red...");
    let networking = networking::NetworkStack::new();
    
    // Inicializar sistema de seguridad AEGIS
    println!("🛡️ Inicializando Sistema de Seguridad AEGIS...");
    let security = security::AegisSecuritySystem::new();
    
    // Inicializar sistema de gráficos LUMEN
    println!("🎨 Inicializando Sistema de Gráficos LUMEN...");
    let graphics = graphics::LumenGraphicsSystem::new();
    
    // Inicializar sistema de auto-creación GENESIS
    println!("⚙️ Inicializando Sistema de Auto-creación GENESIS...");
    let genesis = genesis::GenesisAutoCreationSystem::new();
    
    // Inicializar integración IA Colmena
    println!("🤖 Inicializando Integración IA Colmena...");
    let ia_colmena = ia_colmena::ColmenaIntegration::new();
    
    // Inicializar Crystal UI
    println!("🖥️ Inicializando Crystal UI...");
    let crystal_ui = crystal_ui::CrystalUI::new();
    
    println!("✅ Todos los componentes inicializados exitosamente");
    
    // Entrar en bucle principal del kernel
    kernel_loop();
}

/// Bucle principal del kernel
fn kernel_loop() -> ! {
    println!("🔄 Entrando en bucle principal del kernel...");
    loop {
        // Procesar eventos del sistema
        // Actualizar métricas
        // Optimizar recursos
        unsafe { core::arch::asm!("hlt") };
    }
}

/// Manejador de panic
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("❌ PANIC: {}", info);
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
