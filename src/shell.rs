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
        serial_println!("Type 'help' for available commands.");
        
        // In a real system, this would be a loop reading from keyboard
        // For now, we simulate a 'status' command on boot
        self.execute_command("status");
        self.execute_command("list-nodes");
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
                serial_println!("Resource Nodes:");
                // Use capability to access graph nodes
                crate::capability::invoke_capability(&self.graph_kernel.graph_capability(), |graph| {
                    for (id, node) in &graph.nodes {
                        serial_println!("  [{:?}] {} ({:?})", id, node.name, node.node_type);
                    }
                });
            }
            _ => {
                serial_println!("Unknown command: {}", cmd);
            }
        }
    }
}
