#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::arch::global_asm;

global_asm!(r#"
.global context_switch_asm
context_switch_asm:
    # rdi = *mut old_rsp
    # rsi = new_rsp

    # Guardar registros callee-saved
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    # Cambiar puntero de pila
    mov [rdi], rsp
    mov rsp, rsi

    # Restaurar registros callee-saved
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp

    ret
"#);

extern "C" {
    pub fn context_switch_asm(old_rsp: *mut u64, new_rsp: u64);
}

extern crate alloc;

use core::panic::PanicInfo;
use alloc::{string::String, format, vec::Vec};
use memory::{MemoryManager, MemoryRegion, MemoryRegionType, MemoryRange};
use exokernel_integration::ExokernelIntegration;
use scheduler::Scheduler;
use cronos_container_runtime::CronosContainerRuntime;
use installer_ledger::InstallerLedger;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref INSTALLER_LEDGER: Mutex<InstallerLedger> = Mutex::new(InstallerLedger::new());
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new(scheduler::SchedulerConfig::default()));
}

// Global mutable state for Hive AI (since Mutex<HiveAi> had Send/Sync issues with intrusive pointers)
pub static mut HIVE_AI_INSTANCE: Option<HiveAi> = None;

// Exportar funciones y tipos para boot.rs
pub use crate::serial_writer::{serial_print_hex, serial_print_dec, serial_panic};
pub use crate::serial_writer::SERIAL_WRITER;
pub use crate::boot::BootInfo;

mod boot;
mod allocator;
mod capability;
mod graph_kernel;
mod hal;
mod drivers;
mod driver_manager;
mod compositor;
mod layers;
mod hive_ai;
mod colmena_integration;
mod hardware;
mod hardware_monitor;
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
mod stress_tests;
mod installer_ledger;
mod gpu_drivers_real;
mod multi_agent;
mod autonomous_system;
mod serenity_ui;
mod haiku_bfs;
mod genode_components;
mod plan9_9p;
mod fuchsia_capabilities;
mod spinlock;
pub mod serial_writer;
pub use crate::serial_writer::*;
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
use boot::FramebufferInfo;


// kernel_main_impl será la implementación real, llamada desde boot.rs
pub fn kernel_main_impl(boot_info: &BootInfo) -> ! {
    // Deshabilitar interrupciones inmediatamente para evitar reentrancia
    unsafe { core::arch::asm!("cli"); }
    
    // DEBUG: debugcon antes de cualquier otro output
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'A');
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'\n');
    }
    
    // LOG 1: Entrada
    crate::serial_println!("Kernel Main: Entrada confirmada.");
    
    // LOG 2: Inicio de inicializacion del kernel
    crate::serial_println!("Iniciando inicializacion del kernel...");
    
    unsafe { core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'B'); }
    
    crate::serial_println!("kernel_main: Iniciando GDT");
    INSTALLER_LEDGER.lock().log_event("GDT", "INIT", "Iniciando configuración de GDT");
    gdt::init_gdt();
    crate::serial_println!("kernel_main: Iniciando IDT");
    INSTALLER_LEDGER.lock().log_event("IDT", "INIT", "Iniciando configuración de IDT");
    interrupts::init_idt();
    crate::serial_println!("kernel_main: Iniciando PIC");
    INSTALLER_LEDGER.lock().log_event("PIC", "INIT", "Iniciando configuración de PIC");
    interrupts::init_pics();
    crate::serial_println!("kernel_main: Iniciando PS2/SYSTEM");
    let _ = ps2::Ps2Keyboard::initialize();
    // DEBUG: leer CS register
    unsafe {
        let mut cs: u16;
        core::arch::asm!("mov {cs}, cs", cs = out(reg) cs);
        crate::serial_println!("kernel_main: registro CS = {:#x}", cs);
    }
    crate::serial_println!("kernel_main: habilitando interrupciones");
    x86_64::instructions::interrupts::enable();
    crate::serial_println!("kernel_main: ¡interrupciones habilitadas!");
    
    // Heap despues de GDT/IDT para manejo de excepciones
    crate::serial_println!("kernel_main: iniciando asignador de memoria");
    // Diagnóstico Gemini: verificar SS register
    let ss_val: u16;
    unsafe { core::arch::asm!("mov {ss_val}, ss", ss_val = out(reg) ss_val); }
    crate::serial_println!("registro SS antes de init_allocator: {:#x}", ss_val);
    // Diagnóstico Gemini: verificar PIC Master IMR
    unsafe {
        let master_mask = x86_64::instructions::port::PortReadOnly::<u8>::new(0x21).read();
        crate::serial_println!("PIC Master IMR: {:#04x}", master_mask);
    }
    INSTALLER_LEDGER.lock().log_event("ALLOCATOR", "INIT", "Iniciando configuración del asignador de memoria");
    init_allocator(boot_info);
    unsafe { core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") b'C'); }
    crate::serial_println!("kernel_main: a punto de crear HardwareScanner");

    let mut scanner = hardware::HardwareScanner::new();
    INSTALLER_LEDGER.lock().log_event("PCI", "SCAN", "Iniciando escaneo del bus PCI");
    let pci_devices = scanner.scan_pci_bus();
    crate::serial_println!("Dispositivos PCI encontrados: {}", pci_devices.len());
    crate::serial_println!("DEBUG: before hex print");
    let hhdm_copy = boot_info.hhdm_offset;
    let rsdp_copy = boot_info.rsdp;
    crate::serial_println!("DEBUG: hhdm_offset = 0x{:x}", hhdm_copy);
    crate::serial_println!("DEBUG: rsdp = 0x{:x}", rsdp_copy.unwrap_or(0));
    crate::serial_println!("DEBUG: after hex print");
    crate::serial_println!("DEBUG: boot_info.rsdp = 0x{:x}", boot_info.rsdp.unwrap_or(0));
    let mut acpi_manager = acpi::AcpiManager::new(boot_info.hhdm_offset, boot_info.rsdp);
    let acpi_status = acpi_manager.initialize();

    let fb_info = boot_info.framebuffer.clone();
    
    let (status, graph_kernel, layer_arch, lumen_layer, hive_ai_opt) = initialize_system_with_graph_and_layers(fb_info.as_ref());

    // Initialize ALL Hive AI subsystems
    if let Some(mut hive) = hive_ai_opt {
        let _ = hive.initialize_broker(1);
        let _ = hive.initialize_agent_manager();
        hive.initialize_swarm();
        hive.initialize_multiversal();
        if let Some(ref gk) = graph_kernel {
            hive.initialize_openai(gk.clone());
            hive.initialize_localai(gk.clone());
        }
        // FASE 2.5: Analizar eventos de instalación para auto-corrección neural
        hive.analyze_installer_events(&INSTALLER_LEDGER.lock());

        unsafe { HIVE_AI_INSTANCE = Some(hive); }
    }

    if status {
        if let Some(ref gk) = graph_kernel {
            scanner.register_in_graph(gk);

            if acpi_status.is_ok() {
                // Reactivar código de AcpiManager
                acpi_manager.set_graph_kernel(gk.clone());
                let _ = acpi_manager.enumerate_acpi_devices();

                // if let Some(madt) = &acpi_manager.madt {
                //     let core_count = madt.processor_count();
                //     serial_println!("SMP: {} cores detected", core_count);
                // }
            }

            for dev in &pci_devices {
                INSTALLER_LEDGER.lock().log_event("PCI", "DEVICE_FOUND", &format!("Detectado dispositivo PCI: {:04x}:{:04x} (Clase: {:02x}, Subclase: {:02x})", dev.vendor_id, dev.device_id, dev.class_id, dev.subclass_id));
                let bar0 = dev.get_bar0_addr();
                let hhdm_off = boot_info.hhdm_offset;
                if dev.class_id == 0x01 && dev.subclass_id == 0x08 {
                    let mut nvme = nvme::NvmeController::new(dev.bus, dev.device);
                    if nvme.initialize(hhdm_off + bar0).is_ok() {
                        INSTALLER_LEDGER.lock().log_event("DRIVER", "NVME_INIT", "Controlador NVMe inicializado correctamente");
                        let nvme_node = gk.create_node(
                            graph_kernel::NodeType::HardwareDevice(graph_kernel::HardwareType::Nvme),
                            format!("nvme_{:02x}:{:02x}", dev.bus, dev.device),
                        );
                        gk.create_edge(gk.root_node().unwrap(), nvme_node, graph_kernel::EdgeType::Ownership);
                    } else {
                        INSTALLER_LEDGER.lock().record_conflict(&format!("nvme_{:02x}:{:02x}", dev.bus, dev.device), "INIT_FAILED", "Critical");
                    }
                } else if dev.class_id == 0x0C && dev.subclass_id == 0x03 {
                    let mut xhci = usb_xhci::XhciController::new(hhdm_off + bar0);
                    if xhci.initialize().is_ok() {
                        INSTALLER_LEDGER.lock().log_event("DRIVER", "XHCI_INIT", "Controlador xHCI inicializado correctamente");
                        xhci.probe_ports();
                        let xhci_node = gk.create_node(
                            graph_kernel::NodeType::HardwareDevice(graph_kernel::HardwareType::Xhci),
                            format!("xhci_{:02x}:{:02x}", dev.bus, dev.device),
                        );
                        gk.create_edge(gk.root_node().unwrap(), xhci_node, graph_kernel::EdgeType::Ownership);
                    } else {
                        INSTALLER_LEDGER.lock().record_conflict(&format!("xhci_{:02x}:{:02x}", dev.bus, dev.device), "INIT_FAILED", "High");
                    }
                } else if dev.class_id == 0x02 && dev.subclass_id == 0x00 {
                    let mut nic = e1000e::E1000eDriver::new(hhdm_off + bar0);
                    if nic.initialize().is_ok() {
                        INSTALLER_LEDGER.lock().log_event("DRIVER", "E1000E_INIT", "Controlador E1000E inicializado correctamente");
                        let nic_node = gk.create_node(
                            graph_kernel::NodeType::HardwareDevice(graph_kernel::HardwareType::Network),
                            format!("eth_{:02x}:{:02x}", dev.bus, dev.device),
                        );
                        gk.create_edge(gk.root_node().unwrap(), nic_node, graph_kernel::EdgeType::Ownership);
                    } else {
                        INSTALLER_LEDGER.lock().record_conflict(&format!("eth_{:02x}:{:02x}", dev.bus, dev.device), "INIT_FAILED", "Medium");
                    }
                }
            }

            crate::serial_println!("A punto de crear el nodo del hipervisor");
            let hv_node = gk.create_node(
                graph_kernel::NodeType::VirtualizationResource,
                String::from("hypervisor_root"),
            );
            crate::serial_println!("Nodo del hipervisor creado");
            gk.create_edge(gk.root_node().unwrap(), hv_node, graph_kernel::EdgeType::Ownership);
            crate::serial_println!("Arista del hipervisor creada");

            if let Some(ref arch) = layer_arch {
                crate::serial_println!("A punto de crear ColmenaObserverBridge");
                let mut colmena = colmena_integration::ColmenaObserverBridge::new(arch.clone());
                crate::serial_println!("ColmenaObserverBridge creado");
                colmena.init();
                crate::serial_println!("Colmena inicializada");
                colmena.register_cpu(scanner.cpu_info.clone());
                colmena.update_metrics();
                crate::serial_println!("Métricas de Colmena actualizadas");
            }

            // ── CRONOS ESENCIA: Exokernel + Memoria + Procesos + Contenedores ──
            crate::serial_println!("Exokernel: inicializando MemoryManager");
            let mut memory_regions = Vec::new();
            for region_opt in &boot_info.usable_regions {
                if let Some(region) = region_opt {
                    memory_regions.push(MemoryRegion {
                        range: MemoryRange { start_frame_number: region.base / 4096, end_frame_number: (region.base + region.length) / 4096 },
                        region_type: MemoryRegionType::Usable,
                    });
                }
            }
            let memory_manager = unsafe {
                MemoryManager::new_with_params(boot_info.hhdm_offset, &memory_regions)
            };
            memory_manager.register_in_graph(gk);
            crate::serial_println!("Exokernel: MemoryManager listo");

            let mut exokernel = ExokernelIntegration::new(gk.clone());
            exokernel.initialize(&memory_manager);
            crate::serial_println!("Exokernel: Integración inicializada, métricas: nodos={}, aristas={}",
                exokernel.get_exokernel_metrics().total_nodes,
                exokernel.get_exokernel_metrics().total_edges);

            {
                let mut scheduler = SCHEDULER.lock();
                scheduler.set_graph_kernel(gk.clone());
                let _init_proc = scheduler.create_process(
                    String::from("cronos_init"),
                    scheduler::ProcessPriority::High,
                );
                crate::serial_println!("Scheduler: inicializado, procesos={}", scheduler.process_count());
            }

            let mut container_runtime = CronosContainerRuntime::new();
            container_runtime.set_graph_kernel(gk.clone());
            let _c1 = container_runtime.create_container(
                String::from("cronos-base"),
                String::from("crOS:v1"),
            );
            crate::serial_println!("Container Runtime: listo, contenedores={}", container_runtime.stats().total_containers);
        }
    }

    if let Some(ref lumen) = lumen_layer {
        invoke_capability_mut(&lumen.compositor(), |comp| {
            // Ventana de Bienvenida
            let welcome = comp.create_window(
                String::from("Bienvenido a CRONOS OS"),
                Rect::new(80, 40, 700, 440),
            );
            if let Some(win) = comp.get_window_mut(welcome) {
                win.background_color = 0xFF2D2D3F;
            }
            // Ventana de Terminal
            let term = comp.create_window(
                String::from("Terminal Soberana - bash"),
                Rect::new(160, 100, 650, 380),
            );
            if let Some(win) = comp.get_window_mut(term) {
                win.background_color = 0xFF1A1A2E;
            }
            // Ventana de Monitor Hive AI
            let hive_mon = comp.create_window(
                String::from("Monitor de IA Colmena"),
                Rect::new(300, 180, 550, 300),
            );
            if let Some(win) = comp.get_window_mut(hive_mon) {
                win.background_color = 0xFF1A1A3E;
            }
            // Panel COSMIC (barra de tareas)
            let fb_w = comp.resolution().0;
            let fb_h = comp.resolution().1;
            let panel = comp.create_window(
                String::from("Panel COSMIC"),
                Rect::new(0, (fb_h - 48) as i32, fb_w, 48),
            );
            if let Some(win) = comp.get_window_mut(panel) {
                win.window_type = compositor::WindowType::Popup;
                win.z_order = 9999;
                win.background_color = 0xFF1E1E2E;
                win.has_shadow = false;
            }
            comp.focus_window(term);
        });
    }

    serial_println!("Estado de inicio del sistema: {}", if status { "OK" } else { "FALLIDO" });
    serial_println!("Dispositivos PCI: {}", pci_devices.len());

    use crate::interrupts::pop_scancode;
    use crate::ps2::scancode_to_char;

    // Rendering loop
    if let Some(ref lumen) = lumen_layer {
        loop {
            // Poll keyboard
            while let Some(sc) = pop_scancode() {
                if let Some(ch) = scancode_to_char(sc) {
                    serial_print!("{}", ch);
                } else {
                    serial_print!("[SC:{:#x}]", sc);
                }
            }

            invoke_capability_mut(&lumen.compositor(), |comp| {
                comp.render();
            });
            for _ in 0..50000 {
                core::hint::spin_loop();
            }
        }
    }

    loop {}
}

fn initialize_system(_vga_buffer: *mut u8) -> bool {
    let _ = initialize_system_with_graph_and_layers(None);
    true
}

fn initialize_system_with_graph_and_layers(fb_info: Option<&FramebufferInfo>) -> (bool, Option<GraphKernel>, Option<LayerArchitecture>, Option<LumenLayer>, Option<HiveAi>) {
    let mut graph_kernel = GraphKernel::new();
    graph_kernel.initialize();
    
    let gk_return = graph_kernel.clone();

    let mut layer_architecture = LayerArchitecture::new(graph_kernel);
    layer_architecture.initialize();
    
    let arch_return = layer_architecture.clone();

    let kernel_layer = KernelLayer::new(layer_architecture.clone());
    kernel_layer.initialize();
    
    let aegis_layer = AegisLayer::new(layer_architecture.clone());
    aegis_layer.initialize();

    let (fb_addr, fb_width, fb_height) = if let Some(fb) = fb_info {
        (fb.address as *mut u8, fb.width as u32, fb.height as u32)
    } else {
        (0xb8000 as *mut u8, 640, 480)
    };
    
    let gpu_driver_cap = DriverFactory::create_gpu(0x1234, 0x5678, fb_addr, fb_width, fb_height);
    
    let mut compositor = Compositor::new(layer_architecture.graph_kernel().clone());
    compositor.set_resolution(fb_width, fb_height);
    compositor.initialize(gpu_driver_cap.capability());

    let lumen_layer = LumenLayer::new(layer_architecture.clone(), compositor);
    lumen_layer.initialize();
    
    let genesis_layer = GenesisLayer::new(layer_architecture.clone());
    genesis_layer.initialize();
    
    let hive_ai = HiveAi::new(layer_architecture);
    hive_ai.initialize();
    
    (true, Some(gk_return), Some(arch_return), Some(lumen_layer), Some(hive_ai))
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
fn panic(_info: &PanicInfo) -> ! {
    // Escritura cruda usando instrucción out para puerto serial
    // Esto es solo para diagnóstico
    unsafe {
        let msg = b"PANIC DETECTADO\r\n";
        for &byte in msg {
            core::arch::asm!("out dx, al", in("dx") 0x3f8u16, in("al") byte, options(nomem, nostack));
        }
    }
    loop { unsafe { core::arch::asm!("hlt"); } }
}
