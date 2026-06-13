#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::string::String;
use crate::serial_writer::serial_panic;
use crate::boot::BootInfo;

// Import all modules
mod boot;
mod allocator;
mod capability;
mod graph_kernel;
mod hal;
mod drivers;
mod compositor;
mod layers;
mod hive_ai;
mod colmena_integration;
mod hardware;
mod memory;
mod pci;
mod interrupts;
mod gdt;
mod universal_driver;
mod cosmic_ui;
mod exokernel_integration;
mod desktop;
mod crystal_ui;
mod window_manager;
mod graphics;
mod android_virtualization;
mod virtualization;
mod networking;
mod container;
mod syscalls;
mod scheduler;
mod disk_drivers;
mod filesystem;
mod virtual_memory;
mod network_drivers;
mod gpu_drivers;
mod acpi;
mod smbus;
mod temperature_sensors;
mod fan_control;
mod voltage_sensors;
mod smart_data;
mod thermal_throttling;
mod self_preservation;
mod hardware_health_monitoring;
mod hardware_awareness;
mod pattern_learning;
mod unit_tests;
mod integration_tests;
mod qemu_automation;
mod hardware_testing;
mod gdb_stub;
mod logging;
mod profiling;
mod ahci;
mod nvme;
mod e1000e;
mod intel_hd;
mod ps2;
mod usb_xhci;
mod pcb;
mod address_space;
mod fork_exec;
mod signals;
mod user_space;
mod vfs;
mod mount;
mod permissions;
mod block_cache;
mod fs_integration;
mod ethernet;
mod smoltcp;
mod socket;
mod dhcp;
mod dns;
mod network_testing;
mod libc;
mod posix_shell;
mod coreutils;
mod init;
mod userland_testing;
mod agent_architecture;
mod tool_system;
mod agent_memory;
mod rag_system;
mod metacognition;
mod skills_system;
mod learning_loop;
mod knowledge_persistence;
mod user_modeling;
mod multi_model_integration;
mod context_engineering;
mod agent_protocols;
mod agent_security;
mod production_deployment;
mod browser_automation;
mod shell;
mod redox_drivers;
mod redox_ext4;
mod cosmic_compositor;
mod theseus_scheduler;
mod theseus_memory;
mod theseus_live_evolution;
mod redoxfs;
mod redox_scheme_fs;
mod cronos_container_runtime;
mod cronos_advanced_networking;
mod cronos_advanced_security;
mod cronos_cache_system;
mod cronos_backup_system;
mod cronos_package_manager;
mod cronos_update_system;
mod cronos_monitoring_system;
mod cronos_hypervisor;
mod cronos_btrfs;
mod cronos_power_management;
mod kvm_integration;
mod hyperv_integration;
mod android_emulator_integration;
mod virtio_drivers;
mod theseus_genesis;
mod localai_integration;
mod openai_integration;
mod claude_integration;
mod stable_diffusion_integration;
mod http_client;
mod web_navigation;
mod pdf_generation;
mod auto_evolution;
mod linux_vm;
mod windows_vm;
mod macos_vm;
mod android_avd;
mod vm_program_installer;
mod layer_robustness;
mod intrusion_detection;
mod gpu_drivers_real;
mod multi_agent;
mod autonomous_system;
mod serenity_ui;
mod haiku_bfs;
mod genode_components;
mod plan9_9p;
mod fuchsia_capabilities;
mod spinlock;
mod serial_writer;
mod bitmap_frame_allocator;
// FASE 13: Módulos de networking y filesystem
mod fat32_fs;

use allocator::init_allocator;
use capability::{Capability, Cell, CapabilityRights};
use graph_kernel::GraphKernel;
use drivers::DriverFactory;
use compositor::{Compositor, Rect};
use layers::{LayerArchitecture, Layer, KernelLayer, AegisLayer, LumenLayer, GenesisLayer};
use hive_ai::{HiveAi, SystemMetrics};

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfo) -> ! {
    // FASE 15: Initialize global allocator con información del bootloader
    init_allocator();
    
    let vga_buffer = 0xb8000 as *mut u8;
    
    unsafe {
        // Clear screen
        for i in 0..2000 {
            *vga_buffer.offset(i * 2) = b' ';
            *vga_buffer.offset(i * 2 + 1) = 0x0f;
        }
        
        // Write boot message
        let boot_msg = b"CRONOS W-OS v2.0 - Exokernel with Capabilities";
        for (i, &byte) in boot_msg.iter().enumerate() {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0f;
        }
        
        // Initialize the system
        let status = initialize_system(vga_buffer);
        
        // Write status
        let status_msg = if status {
            b"System initialized successfully - Capabilities active"
        } else {
            b"System initialization failed - Capabilities inactive "
        };
        
        for (i, &byte) in status_msg.iter().enumerate() {
            *vga_buffer.offset((i + 80) as isize * 2) = byte;
            *vga_buffer.offset((i + 80) as isize * 2 + 1) = 0x0f;
        }
        
        // Write architecture info
        let arch_msg = b"4-Layer: AEGIS, LUMEN, GENESIS, Kernel - Hive AI Bridge";
        for (i, &byte) in arch_msg.iter().enumerate() {
            *vga_buffer.offset((i + 160) as isize * 2) = byte;
            *vga_buffer.offset((i + 160) as isize * 2 + 1) = 0x0f;
        }
        
        // Write driver info
        let driver_msg = b"Drivers: GPU, NVMe, xHCI, WiFi, Audio, Net (Redox ports)";
        for (i, &byte) in driver_msg.iter().enumerate() {
            *vga_buffer.offset((i + 240) as isize * 2) = byte;
            *vga_buffer.offset((i + 240) as isize * 2 + 1) = 0x0f;
        }
        
        // Write compositor info
        let comp_msg = b"Compositor: GraphNode windows + GPU capability - No syscalls";
        for (i, &byte) in comp_msg.iter().enumerate() {
            *vga_buffer.offset((i + 320) as isize * 2) = byte;
            *vga_buffer.offset((i + 320) as isize * 2 + 1) = 0x0f;
        }
    }
    
    loop {}
}

fn initialize_system(vga_buffer: *mut u8) -> bool {
    // Simplified initialization for no_std environment
    // The full architecture is implemented in the modules
    
    // Step 1: Initialize GraphKernel (core of exokernel)
    let mut graph_kernel = GraphKernel::new();
    graph_kernel.initialize();
    
    // Step 2: Initialize 4-Layer Architecture
    let mut layer_architecture = LayerArchitecture::new(graph_kernel);
    layer_architecture.initialize();
    
    // Step 3: Initialize individual layers
    let kernel_layer = KernelLayer::new(layer_architecture.clone());
    kernel_layer.initialize();
    
    let aegis_layer = AegisLayer::new(layer_architecture.clone());
    aegis_layer.initialize();
    
    // Step 4: Create GPU driver wrapped in capability
    let gpu_driver_cap = DriverFactory::create_gpu(0x1234, 0x5678, vga_buffer, 640, 480);
    
    // Step 5: Initialize Compositor with GPU capability
    let compositor = Compositor::new(layer_architecture.graph_kernel().clone());
    let mut compositor = compositor;
    compositor.initialize(gpu_driver_cap.capability());
    
    // Step 6: Initialize LUMEN layer with compositor
    let lumen_layer = LumenLayer::new(layer_architecture.clone(), compositor);
    lumen_layer.initialize();
    
    // Step 7: Initialize GENESIS layer
    let genesis_layer = GenesisLayer::new(layer_architecture.clone());
    genesis_layer.initialize();
    
    // Step 8: Initialize Hive AI as bridge capability between all layers
    let mut hive_ai = HiveAi::new(layer_architecture);
    hive_ai.initialize();
    
    true
}

// Helper function to invoke capability mut
fn invoke_capability_mut<T, R, F>(cap: &Capability<T>, f: F) -> Option<R>
where
    T: ?Sized,
    F: FnOnce(&mut T) -> R,
{
    unsafe {
        cap.get_mut().map(f)
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // BUG #12 corregido: usar serial_panic para imprimir archivo, línea y mensaje
    serial_panic(info);
    loop {}
}
