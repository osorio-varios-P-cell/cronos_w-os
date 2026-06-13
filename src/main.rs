#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::{string::String, format};
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
mod guest_integration;
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
mod audio_bridge;
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
mod neural_fable_tests;
mod hive_multiversal;
mod hardware_bridge;
mod hive_swarm;
mod resource_orchestrator;
mod media_engine;
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
    // BUG #16 Corregido: Usar boot_info real para inicializar memoria
    init_allocator(boot_info);
    
    let vga_buffer = 0xb8000 as *mut u8;
    
    unsafe {
        // Clear screen
        for i in 0..2000 {
            *vga_buffer.offset(i * 2) = b' ';
            *vga_buffer.offset(i * 2 + 1) = 0x0f;
        }
        
        // Write boot message
        let boot_msg = b"CRONOS W-OS v2.0 - SO SOBERANO (Exokernel + Grafos)";
        for (i, &byte) in boot_msg.iter().enumerate() {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0b; // Cyan
        }
    }

    // Inicializar GDT e IDT
    gdt::init_gdt();
    interrupts::init_idt();
    interrupts::init_pics();
    x86_64::instructions::interrupts::enable();

    // Inicializar hardware real (PCI y ACPI)
    let mut scanner = hardware::HardwareScanner::new();
    let pci_devices = scanner.scan_pci_bus();

    let mut acpi_manager = acpi::AcpiManager::new();
    let acpi_status = acpi_manager.initialize();

    // Initialize the system architecture
    let (status, graph_kernel, layer_arch) = initialize_system_with_graph_and_layers(vga_buffer);

    if status {
        if let Some(gk) = graph_kernel {
            scanner.register_in_graph(&gk);

            // Registrar ACPI y habilitar SMP (Multinúcleo)
            if acpi_status.is_ok() {
                acpi_manager.set_graph_kernel(gk.clone());
                let _ = acpi_manager.enumerate_acpi_devices();

                if let Some(madt) = &acpi_manager.madt {
                    let core_count = madt.processor_count();
                    let msg = format!("SOPORTE MULTINUCLEO DETECTADO: {} CORES", core_count);
                    for (i, byte) in msg.as_bytes().iter().enumerate() {
                        unsafe {
                            *vga_buffer.offset((i + 320) as isize * 2) = *byte;
                            *vga_buffer.offset((i + 320) as isize * 2 + 1) = 0x0d; // Light Magenta
                        }
                    }
                }
            }

            // FASE 2: Inicializar NVMe y xHCI con direcciones reales del scanner PCI
            for dev in &pci_devices {
                if dev.class_id == 0x01 && dev.subclass_id == 0x08 { // NVMe
                    let mut nvme = nvme::NvmeController::new(0, 0);
                    if nvme.initialize().is_ok() {
                        let nvme_node = gk.create_node(
                            graph_kernel::NodeType::HardwareDevice(graph_kernel::HardwareType::Nvme),
                            format!("nvme_{:02x}:{:02x}", dev.bus, dev.device),
                        );
                        gk.create_edge(gk.root_node().unwrap(), nvme_node, graph_kernel::EdgeType::Ownership);
                    }
                } else if dev.class_id == 0x0C && dev.subclass_id == 0x03 { // xHCI (USB 3.0)
                    let mut xhci = usb_xhci::XhciController::new(0);
                    if xhci.initialize().is_ok() {
                        let xhci_node = gk.create_node(
                            graph_kernel::NodeType::HardwareDevice(graph_kernel::HardwareType::Xhci),
                            format!("xhci_{:02x}:{:02x}", dev.bus, dev.device),
                        );
                        gk.create_edge(gk.root_node().unwrap(), xhci_node, graph_kernel::EdgeType::Ownership);
                    }
                }
            }

            // FASE 2.5: Virtualización y Contenedores en el Grafo
            let hv_node = gk.create_node(
                graph_kernel::NodeType::VirtualizationResource,
                String::from("hypervisor_root"),
            );
            gk.create_edge(gk.root_node().unwrap(), hv_node, graph_kernel::EdgeType::Ownership);

            // FASE 5: IA Colmena
            if let Some(arch) = layer_arch {
                let mut colmena = colmena_integration::ColmenaObserverBridge::new(arch);
                colmena.init();
                colmena.register_cpu(scanner.cpu_info.clone());
                colmena.update_metrics();
            }

            // Iniciar Shell Soberana
            let mut shell = shell::SovereignShell::new(gk.clone());
            shell.run();
        }
    }

    unsafe {
        // Write status
        let status_msg = if status {
            b"SISTEMA INICIALIZADO - CAPABILITIES Y GRAFOS ACTIVOS"
        } else {
            b"ERROR EN INICIALIZACION DE ARQUITECTURA CRONOS      "
        };
        
        for (i, &byte) in status_msg.iter().enumerate() {
            *vga_buffer.offset((i + 80) as isize * 2) = byte;
            *vga_buffer.offset((i + 80) as isize * 2 + 1) = if status { 0x0a } else { 0x0c }; // Green or Red
        }
        
        // Write architecture info
        let arch_msg = b"CAPAS: [KERNEL] -> [AEGIS] -> [LUMEN] -> [GENESIS]";
        for (i, &byte) in arch_msg.iter().enumerate() {
            *vga_buffer.offset((i + 160) as isize * 2) = byte;
            *vga_buffer.offset((i + 160) as isize * 2 + 1) = 0x0e; // Yellow
        }
        
        let dev_count_msg = format!("DISPOSITIVOS PCI DETECTADOS: {}", pci_devices.len());
        for (i, byte) in dev_count_msg.as_bytes().iter().enumerate() {
            *vga_buffer.offset((i + 240) as isize * 2) = *byte;
            *vga_buffer.offset((i + 240) as isize * 2 + 1) = 0x07; // Light Gray
        }
    }
    
    loop {}
}

fn initialize_system(vga_buffer: *mut u8) -> bool {
    initialize_system_with_graph_and_layers(vga_buffer).0
}

fn initialize_system_with_graph_and_layers(vga_buffer: *mut u8) -> (bool, Option<GraphKernel>, Option<LayerArchitecture>) {
    // Simplified initialization for no_std environment
    // The full architecture is implemented in the modules
    
    // Step 1: Initialize GraphKernel (core of exokernel)
    let mut graph_kernel = GraphKernel::new();
    graph_kernel.initialize();
    
    let gk_return = graph_kernel.clone();

    // Step 2: Initialize 4-Layer Architecture
    let mut layer_architecture = LayerArchitecture::new(graph_kernel);
    layer_architecture.initialize();
    
    let arch_return = layer_architecture.clone();

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
    let hive_ai = HiveAi::new(layer_architecture);
    hive_ai.initialize();
    
    (true, Some(gk_return), Some(arch_return))
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
