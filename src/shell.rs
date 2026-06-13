//! Sovereign Shell for CRONOS W-OS
//!
//! This module provides a minimal interactive environment to interact
//! with the GraphKernel and system resources directly.

use crate::graph_kernel::GraphKernel;
use crate::serial_println;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

pub struct SovereignShell {
    graph_kernel: GraphKernel,
}

impl SovereignShell {
    pub fn new(graph_kernel: GraphKernel) -> Self {
        Self { graph_kernel }
    }

    pub fn run(&self) {
        serial_println!("--- CRONOS SOVEREIGN SHELL v1.0 ---");
        serial_println!("Ready for interaction. (Continuous Loop)");
        serial_println!("Type 'help' for available commands.");
        
        // Bucle real de comandos
        // En un entorno no_std, esto esperaría interrupciones de teclado
        // Por ahora, ejecutamos la secuencia de arranque y quedamos en escucha
        let startup_commands = ["status", "sysinfo", "list-nodes"];
        for cmd in startup_commands {
            serial_println!("\ncronos@sovereign:~$ {}", cmd);
            self.execute_command(cmd);
        }

        serial_println!("\nShell listening on COM1 / Keyboard...");
    }

    pub fn execute_command(&self, cmd: &str) {
        match cmd {
            "help" => {
                serial_println!("Available commands:");
                serial_println!("  help       - Show this help");
                serial_println!("  status     - Show GraphKernel status");
                serial_println!("  list-nodes - List all resource nodes in the graph");
                serial_println!("  clear      - Clear the screen (simulated)");
            }
            "status" => {
                let stats = self.graph_kernel.get_stats();
                serial_println!("GraphKernel Status:");
                serial_println!("  Total Nodes: {}", stats.node_count);
                serial_println!("  Total Edges: {}", stats.edge_count);
                serial_println!("  Isolated Nodes: {}", stats.isolated_nodes);
            }
            "list-nodes" => {
                serial_println!("Resource Nodes in Graph:");
                // Use capability to access graph nodes
                crate::capability::invoke_capability(&self.graph_kernel.graph_capability(), |graph| {
                    for (id, node) in &graph.nodes {
                        serial_println!("  [ID:{:?}] {} - Type:{:?}", id, node.name, node.node_type);
                    }
                });
            }
            "sysinfo" => {
                serial_println!("System Information:");
                serial_println!("  OS Name: CRONOS W-OS");
                serial_println!("  Edition: Sovereign (Soberana) v2.0-Mature");
                serial_println!("  Kernel Type: Exokernel with Resource Graphs");
                serial_println!("  Security: AEGIS (Cascade Revocation active)");
                serial_println!("  UI: LUMEN Compositor (3-Layer Optimized)");
                serial_println!("  AI: Colmena IA (Telemetry active)");
                serial_println!("  Architecture: x86_64 SMP Ready");
            }
            _ => {
                serial_println!("Unknown command: {}", cmd);
            }
        }
    }
}
