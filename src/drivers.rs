//! Redox Drivers Ported and Wrapped in Capabilities
//! 
//! This module contains driver implementations ported from Redox OS,
//! wrapped in the CRONOS capability system for secure access.

use crate::hal::{
    Device, DeviceError, DeviceCapabilities,
    GpuDevice, GpuContext, GpuCommand,
    NvmeDevice,
    XhciDevice, PortStatus, UsbSpeed,
    WifiDevice, WifiNetwork, WifiEncryption, ConnectionStatus,
    AudioDevice,
    NetworkDevice, LinkStatus, NetworkStatistics,
    InputDevice, InputType, InputEvent,
    DeviceCapability,
};
use crate::capability::{Capability, Cell, CapabilityRights, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeType, HardwareType, NodeId};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// GPU Driver (Redox port)
#[derive(Debug, Clone)]
pub struct RedoxGpuDriver {
    name: String,
    vendor_id: u16,
    device_id: u16,
    framebuffer: *mut u8,
    width: u32,
    height: u32,
    initialized: bool,
    next_context_id: u64,
    /// FASE 2: Graph kernel integration
    graph_kernel: Option<Cell<GraphKernel>>,
    graph_node_id: Option<NodeId>,
}

impl RedoxGpuDriver {
    pub fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            name: String::from("redox-gpu"),
            vendor_id,
            device_id,
            framebuffer: 0xb8000 as *mut u8, // Default VGA buffer
            width: 640,
            height: 480,
            initialized: false,
            next_context_id: 1,
            graph_kernel: None,
            graph_node_id: None,
        }
    }

    pub fn with_framebuffer(mut self, framebuffer: *mut u8, width: u32, height: u32) -> Self {
        self.framebuffer = framebuffer;
        self.width = width;
        self.height = height;
        self
    }

    /// FASE 2: Set graph kernel reference
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// FASE 2: Register driver in graph kernel
    pub fn register_in_graph(&mut self) -> Result<NodeId, String> {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Gpu);
            let node_name = format!("redox_gpu_{:04x}_{:04x}", self.vendor_id, self.device_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                self.graph_node_id = Some(id);
                Ok(id)
            } else {
                Err(String::from("Failed to create node in graph"))
            }
        } else {
            Err(String::from("Graph kernel not set"))
        }
    }
}

impl Device for RedoxGpuDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.device_id
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        self.initialized = true;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.initialized = false;
        self.init()
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            dma: true,
            interrupt: true,
            mmio: true,
            pio: false,
        }
    }
}

impl GpuDevice for RedoxGpuDriver {
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), DeviceError> {
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn get_resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn framebuffer(&self) -> *mut u8 {
        self.framebuffer
    }

    fn framebuffer_size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }

    fn swap_buffers(&mut self) -> Result<(), DeviceError> {
        // En una implementación real con hardware, esto dispararía el cambio de registro
        // o esperaría al VSync. Aquí es un placeholder para la orquestación.
        Ok(())
    }

    fn create_context(&mut self) -> Result<GpuContext, DeviceError> {
        let context = GpuContext(self.next_context_id);
        self.next_context_id += 1;
        Ok(context)
    }

    fn destroy_context(&mut self, _context: GpuContext) -> Result<(), DeviceError> {
        Ok(())
    }

    fn execute_command(&mut self, _context: &GpuContext, command: GpuCommand) -> Result<(), DeviceError> {
        match command {
            GpuCommand::Clear { r, g, b, a } => {
                let color = ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);
                let size = self.framebuffer_size();
                unsafe {
                    let fb = self.framebuffer as *mut u32;
                    for i in 0..(size / 4) {
                        *fb.add(i) = color;
                    }
                }
            }
            GpuCommand::DrawRect { x, y, width, height, color } => {
                unsafe {
                    let fb = self.framebuffer as *mut u32;
                    for dy in 0..height {
                        for dx in 0..width {
                            let px = x + dx;
                            let py = y + dy;
                            if px < self.width && py < self.height {
                                let offset = (py * self.width + px) as usize;
                                *fb.add(offset) = color;
                            }
                        }
                    }
                }
            }
            GpuCommand::DrawText { x, y, text } => {
                unsafe {
                    let fb = self.framebuffer as *mut u32;
                    let mut cx = x;
                    for ch in text.bytes() {
                        if !(0x20..=0x7E).contains(&ch) { cx += 8; continue; }
                        let glyph = FONT8x8[(ch - 0x20) as usize];
                        for row in 0..8 {
                            let bits = glyph[row];
                            for col in 0..8 {
                                if (bits >> (7 - col)) & 1 != 0 {
                                    let px = cx + col;
                                    let py = y + row as u32;
                                    if px < self.width && py < self.height {
                                        *fb.add((py * self.width + px) as usize) = 0xFFFFFFFF;
                                    }
                                }
                            }
                        }
                        cx += 8;
                    }
                }
            }
            GpuCommand::DrawCircle { cx, cy, radius, color } => {
                let mut x = radius;
                let mut y = 0;
                let mut err = 0;
                unsafe {
                    let fb = self.framebuffer as *mut u32;
                    while x >= y {
                        let points = [
                            (cx + x, cy + y), (cx + y, cy + x), (cx - y, cy + x), (cx - x, cy + y),
                            (cx - x, cy - y), (cx - y, cy - x), (cx + y, cy - x), (cx + x, cy - y)
                        ];
                        for (px, py) in points {
                            if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                                *fb.add((py as u32 * self.width + px as u32) as usize) = color;
                            }
                        }
                        if err <= 0 { y += 1; err += 2 * y + 1; }
                        if err > 0 { x -= 1; err -= 2 * x + 1; }
                    }
                }
            }
            GpuCommand::FillCircle { cx, cy, radius, color } => {
                unsafe {
                    let fb = self.framebuffer as *mut u32;
                    for y in (cy - radius)..=(cy + radius) {
                        for x in (cx - radius)..=(cx + radius) {
                            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                                let dx = x - cx;
                                let dy = y - cy;
                                if dx * dx + dy * dy <= radius * radius {
                                    *fb.add((y as u32 * self.width + x as u32) as usize) = color;
                                }
                            }
                        }
                    }
                }
            }
            GpuCommand::Blit { src_x, src_y, dst_x, dst_y, width, height } => {
                unsafe {
                    let fb = self.framebuffer as *mut u32;
                    for dy in 0..height {
                        for dx in 0..width {
                            let sx = src_x + dx;
                            let sy = src_y + dy;
                            let dx_pos = dst_x + dx;
                            let dy_pos = dst_y + dy;

                            if sx < self.width && sy < self.height && dx_pos < self.width && dy_pos < self.height {
                                let src_offset = (sy * self.width + sx) as usize;
                                let dst_offset = (dy_pos * self.width + dx_pos) as usize;
                                *fb.add(dst_offset) = *fb.add(src_offset);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

static FONT8x8: [[u8; 8]; 95] = [
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], //  
    [0x18,0x18,0x18,0x18,0x18,0x00,0x18,0x00], // !
    [0x6C,0x6C,0x6C,0x00,0x00,0x00,0x00,0x00], // "
    [0x6C,0x6C,0xFE,0x6C,0xFE,0x6C,0x6C,0x00], // #
    [0x18,0x3E,0x60,0x3C,0x06,0x7C,0x18,0x00], // $
    [0x00,0xC6,0xCC,0x18,0x30,0x66,0xC6,0x00], // %
    [0x38,0x6C,0x38,0x76,0xDC,0xCC,0x76,0x00], // &
    [0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00], // '
    [0x0C,0x18,0x30,0x30,0x30,0x18,0x0C,0x00], // (
    [0x30,0x18,0x0C,0x0C,0x0C,0x18,0x30,0x00], // )
    [0x00,0x66,0x3C,0xFF,0x3C,0x66,0x00,0x00], // *
    [0x00,0x18,0x18,0x7E,0x18,0x18,0x00,0x00], // +
    [0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x30], // ,
    [0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00], // -
    [0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x00], // .
    [0x06,0x0C,0x18,0x30,0x60,0xC0,0x80,0x00], // /
    [0x3C,0x66,0x6E,0x7E,0x76,0x66,0x3C,0x00], // 0
    [0x18,0x38,0x18,0x18,0x18,0x18,0x7E,0x00], // 1
    [0x3C,0x66,0x06,0x0C,0x18,0x30,0x7E,0x00], // 2
    [0x3C,0x66,0x06,0x1C,0x06,0x66,0x3C,0x00], // 3
    [0x0C,0x1C,0x3C,0x6C,0xFE,0x0C,0x0C,0x00], // 4
    [0x7E,0x60,0x7C,0x06,0x06,0x66,0x3C,0x00], // 5
    [0x3C,0x66,0x60,0x7C,0x66,0x66,0x3C,0x00], // 6
    [0x7E,0x06,0x0C,0x18,0x30,0x30,0x30,0x00], // 7
    [0x3C,0x66,0x66,0x3C,0x66,0x66,0x3C,0x00], // 8
    [0x3C,0x66,0x66,0x3E,0x06,0x66,0x3C,0x00], // 9
    [0x00,0x18,0x18,0x00,0x00,0x18,0x18,0x00], // :
    [0x00,0x18,0x18,0x00,0x00,0x18,0x18,0x30], // ;
    [0x0C,0x18,0x30,0x60,0x30,0x18,0x0C,0x00], // <
    [0x00,0x00,0x7E,0x00,0x00,0x7E,0x00,0x00], // =
    [0x30,0x18,0x0C,0x06,0x0C,0x18,0x30,0x00], // >
    [0x3C,0x66,0x06,0x0C,0x18,0x00,0x18,0x00], // ?
    [0x3C,0x66,0x6E,0x6E,0x60,0x66,0x3C,0x00], // @
    [0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x00], // A
    [0x7C,0x66,0x66,0x7C,0x66,0x66,0x7C,0x00], // B
    [0x3C,0x66,0x60,0x60,0x60,0x66,0x3C,0x00], // C
    [0x78,0x6C,0x66,0x66,0x66,0x6C,0x78,0x00], // D
    [0x7E,0x60,0x60,0x7C,0x60,0x60,0x7E,0x00], // E
    [0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x00], // F
    [0x3C,0x66,0x60,0x6E,0x66,0x66,0x3C,0x00], // G
    [0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x00], // H
    [0x7E,0x18,0x18,0x18,0x18,0x18,0x7E,0x00], // I
    [0x3E,0x0C,0x0C,0x0C,0x0C,0x6C,0x38,0x00], // J
    [0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x00], // K
    [0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00], // L
    [0xC6,0xEE,0xFE,0xD6,0xC6,0xC6,0xC6,0x00], // M
    [0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x00], // N
    [0x3C,0x66,0x66,0x66,0x66,0x66,0x3C,0x00], // O
    [0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x00], // P
    [0x3C,0x66,0x66,0x66,0x66,0x3C,0x0E,0x00], // Q
    [0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x00], // R
    [0x3C,0x66,0x60,0x3C,0x06,0x66,0x3C,0x00], // S
    [0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x00], // T
    [0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00], // U
    [0x66,0x66,0x66,0x66,0x66,0x3C,0x18,0x00], // V
    [0xC6,0xC6,0xC6,0xD6,0xFE,0xEE,0xC6,0x00], // W
    [0xC6,0xEE,0x7C,0x38,0x7C,0xEE,0xC6,0x00], // X
    [0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x00], // Y
    [0x7E,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00], // Z
    [0x3C,0x30,0x30,0x30,0x30,0x30,0x3C,0x00], // [
    [0xC0,0x60,0x30,0x18,0x0C,0x06,0x02,0x00], // backslash
    [0x3C,0x0C,0x0C,0x0C,0x0C,0x0C,0x3C,0x00], // ]
    [0x10,0x38,0x6C,0xC6,0x00,0x00,0x00,0x00], // ^
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xFF], // _
    [0x18,0x18,0x0C,0x00,0x00,0x00,0x00,0x00], // `
    [0x00,0x00,0x3C,0x06,0x3E,0x66,0x3E,0x00], // a
    [0x60,0x60,0x7C,0x66,0x66,0x66,0x7C,0x00], // b
    [0x00,0x00,0x3C,0x66,0x60,0x66,0x3C,0x00], // c
    [0x06,0x06,0x3E,0x66,0x66,0x66,0x3E,0x00], // d
    [0x00,0x00,0x3C,0x66,0x7E,0x60,0x3C,0x00], // e
    [0x1C,0x30,0x7C,0x30,0x30,0x30,0x30,0x00], // f
    [0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x3C], // g
    [0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x00], // h
    [0x18,0x00,0x38,0x18,0x18,0x18,0x3C,0x00], // i
    [0x0C,0x00,0x1C,0x0C,0x0C,0x0C,0x6C,0x38], // j
    [0x60,0x60,0x66,0x6C,0x78,0x6C,0x66,0x00], // k
    [0x38,0x18,0x18,0x18,0x18,0x18,0x3C,0x00], // l
    [0x00,0x00,0xEC,0xFE,0xD6,0xC6,0xC6,0x00], // m
    [0x00,0x00,0x7C,0x66,0x66,0x66,0x66,0x00], // n
    [0x00,0x00,0x3C,0x66,0x66,0x66,0x3C,0x00], // o
    [0x00,0x00,0x7C,0x66,0x66,0x7C,0x60,0x60], // p
    [0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x06], // q
    [0x00,0x00,0x7C,0x66,0x60,0x60,0x60,0x00], // r
    [0x00,0x00,0x3E,0x60,0x3C,0x06,0x7C,0x00], // s
    [0x30,0x30,0x7C,0x30,0x30,0x30,0x1C,0x00], // t
    [0x00,0x00,0x66,0x66,0x66,0x66,0x3E,0x00], // u
    [0x00,0x00,0x66,0x66,0x66,0x3C,0x18,0x00], // v
    [0x00,0x00,0xC6,0xC6,0xD6,0xFE,0x6C,0x00], // w
    [0x00,0x00,0x66,0x3C,0x18,0x3C,0x66,0x00], // x
    [0x00,0x00,0x66,0x66,0x66,0x3E,0x06,0x3C], // y
    [0x00,0x00,0x7E,0x0C,0x18,0x30,0x7E,0x00], // z
    [0x0E,0x18,0x18,0x70,0x18,0x18,0x0E,0x00], // {
    [0x18,0x18,0x18,0x00,0x18,0x18,0x18,0x00], // |
    [0x70,0x18,0x18,0x0E,0x18,0x18,0x70,0x00], // }
    [0x76,0xDC,0x00,0x00,0x00,0x00,0x00,0x00], // ~
];

/// NVMe Driver (Redox port)
pub struct RedoxNvmeDriver {
    name: String,
    vendor_id: u16,
    device_id: u16,
    base_address: usize,
    initialized: bool,
    namespace_count: u32,
    /// FASE 2: Graph kernel integration
    graph_kernel: Option<Cell<GraphKernel>>,
    graph_node_id: Option<NodeId>,
}

impl RedoxNvmeDriver {
    pub fn new(vendor_id: u16, device_id: u16, base_address: usize) -> Self {
        Self {
            name: String::from("redox-nvme"),
            vendor_id,
            device_id,
            base_address,
            initialized: false,
            namespace_count: 1,
            graph_kernel: None,
            graph_node_id: None,
        }
    }

    /// FASE 2: Set graph kernel reference
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// FASE 2: Register driver in graph kernel
    pub fn register_in_graph(&mut self) -> Result<NodeId, String> {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Storage);
            let node_name = format!("redox_nvme_{:04x}_{:04x}", self.vendor_id, self.device_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                self.graph_node_id = Some(id);
                Ok(id)
            } else {
                Err(String::from("Failed to create node in graph"))
            }
        } else {
            Err(String::from("Graph kernel not set"))
        }
    }
}

impl Device for RedoxNvmeDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.device_id
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        self.initialized = true;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.initialized = false;
        self.init()
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            dma: true,
            interrupt: true,
            mmio: true,
            pio: false,
        }
    }
}

impl NvmeDevice for RedoxNvmeDriver {
    fn namespace_count(&self) -> u32 {
        self.namespace_count
    }

    fn namespace_size(&self, _namespace_id: u32) -> Result<u64, DeviceError> {
        Ok(1024 * 1024 * 1024) // 1GB default
    }

    fn read(&mut self, _namespace_id: u32, _lba: u64, buffer: &mut [u8]) -> Result<(), DeviceError> {
        // TODO: Implement NVMe read
        for byte in buffer.iter_mut() {
            *byte = 0;
        }
        Ok(())
    }

    fn write(&mut self, _namespace_id: u32, _lba: u64, _buffer: &[u8]) -> Result<(), DeviceError> {
        // TODO: Implement NVMe write
        Ok(())
    }

    fn flush(&mut self, _namespace_id: u32) -> Result<(), DeviceError> {
        Ok(())
    }
}

/// xHCI USB Driver (Redox port)
pub struct RedoxXhciDriver {
    name: String,
    vendor_id: u16,
    device_id: u16,
    base_address: usize,
    initialized: bool,
    port_count: u32,
    /// FASE 2: Graph kernel integration
    graph_kernel: Option<Cell<GraphKernel>>,
    graph_node_id: Option<NodeId>,
}

impl RedoxXhciDriver {
    pub fn new(vendor_id: u16, device_id: u16, base_address: usize) -> Self {
        Self {
            name: String::from("redox-xhci"),
            vendor_id,
            device_id,
            base_address,
            initialized: false,
            port_count: 4,
            graph_kernel: None,
            graph_node_id: None,
        }
    }

    /// FASE 2: Set graph kernel reference
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// FASE 2: Register driver in graph kernel
    pub fn register_in_graph(&mut self) -> Result<NodeId, String> {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Xhci);
            let node_name = format!("redox_xhci_{:04x}_{:04x}", self.vendor_id, self.device_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                self.graph_node_id = Some(id);
                Ok(id)
            } else {
                Err(String::from("Failed to create node in graph"))
            }
        } else {
            Err(String::from("Graph kernel not set"))
        }
    }
}

impl Device for RedoxXhciDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.device_id
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        self.initialized = true;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.initialized = false;
        self.init()
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            dma: true,
            interrupt: true,
            mmio: true,
            pio: false,
        }
    }
}

impl XhciDevice for RedoxXhciDriver {
    fn port_count(&self) -> u32 {
        self.port_count
    }

    fn reset_port(&mut self, _port: u32) -> Result<(), DeviceError> {
        Ok(())
    }

    fn enable_port(&mut self, _port: u32) -> Result<(), DeviceError> {
        Ok(())
    }

    fn disable_port(&mut self, _port: u32) -> Result<(), DeviceError> {
        Ok(())
    }

    fn port_status(&self, _port: u32) -> Result<PortStatus, DeviceError> {
        Ok(PortStatus {
            connected: false,
            enabled: false,
            speed: UsbSpeed::High,
        })
    }

    fn submit_transfer(&mut self, _endpoint: u8, _buffer: &[u8]) -> Result<(), DeviceError> {
        Ok(())
    }

    fn receive_transfer(&mut self, _endpoint: u8, buffer: &mut [u8]) -> Result<usize, DeviceError> {
        Ok(0)
    }
}

/// WiFi Driver (Redox port)
pub struct RedoxWifiDriver {
    name: String,
    vendor_id: u16,
    device_id: u16,
    mac_address: [u8; 6],
    initialized: bool,
    connection_status: ConnectionStatus,
    /// FASE 2: Graph kernel integration
    graph_kernel: Option<Cell<GraphKernel>>,
    graph_node_id: Option<NodeId>,
}

impl RedoxWifiDriver {
    pub fn new(vendor_id: u16, device_id: u16, mac_address: [u8; 6]) -> Self {
        Self {
            name: String::from("redox-wifi"),
            vendor_id,
            device_id,
            mac_address,
            initialized: false,
            connection_status: ConnectionStatus::Disconnected,
            graph_kernel: None,
            graph_node_id: None,
        }
    }

    /// FASE 2: Set graph kernel reference
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// FASE 2: Register driver in graph kernel
    pub fn register_in_graph(&mut self) -> Result<NodeId, String> {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Network);
            let node_name = format!("redox_wifi_{:04x}_{:04x}", self.vendor_id, self.device_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                self.graph_node_id = Some(id);
                Ok(id)
            } else {
                Err(String::from("Failed to create node in graph"))
            }
        } else {
            Err(String::from("Graph kernel not set"))
        }
    }
}

impl Device for RedoxWifiDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.device_id
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        self.initialized = true;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.initialized = false;
        self.init()
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            dma: true,
            interrupt: true,
            mmio: true,
            pio: false,
        }
    }
}

impl WifiDevice for RedoxWifiDriver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac_address
    }

    fn scan_networks(&mut self) -> Result<Vec<WifiNetwork>, DeviceError> {
        Ok(Vec::new())
    }

    fn connect(&mut self, _ssid: &str, _password: &str) -> Result<(), DeviceError> {
        self.connection_status = ConnectionStatus::Connected;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), DeviceError> {
        self.connection_status = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.connection_status.clone()
    }

    fn send_packet(&mut self, _packet: &[u8]) -> Result<(), DeviceError> {
        Ok(())
    }

    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, DeviceError> {
        Ok(0)
    }
}

/// Audio Driver (Redox port)
pub struct RedoxAudioDriver {
    name: String,
    vendor_id: u16,
    device_id: u16,
    base_address: usize,
    initialized: bool,
    channel_count: u32,
    sample_rate: u32,
    volume: f32,
    /// FASE 2: Graph kernel integration
    graph_kernel: Option<Cell<GraphKernel>>,
    graph_node_id: Option<NodeId>,
}

impl RedoxAudioDriver {
    pub fn new(vendor_id: u16, device_id: u16, base_address: usize) -> Self {
        Self {
            name: String::from("redox-audio"),
            vendor_id,
            device_id,
            base_address,
            initialized: false,
            channel_count: 2,
            sample_rate: 44100,
            volume: 1.0,
            graph_kernel: None,
            graph_node_id: None,
        }
    }

    /// FASE 2: Set graph kernel reference
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// FASE 2: Register driver in graph kernel
    pub fn register_in_graph(&mut self) -> Result<NodeId, String> {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Audio);
            let node_name = format!("redox_audio_{:04x}_{:04x}", self.vendor_id, self.device_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                self.graph_node_id = Some(id);
                Ok(id)
            } else {
                Err(String::from("Failed to create node in graph"))
            }
        } else {
            Err(String::from("Graph kernel not set"))
        }
    }
}

impl Device for RedoxAudioDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.device_id
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        self.initialized = true;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.initialized = false;
        self.init()
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            dma: true,
            interrupt: true,
            mmio: true,
            pio: false,
        }
    }
}

impl AudioDevice for RedoxAudioDriver {
    fn channel_count(&self) -> u32 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn set_sample_rate(&mut self, rate: u32) -> Result<(), DeviceError> {
        self.sample_rate = rate;
        Ok(())
    }

    fn buffer_size(&self) -> usize {
        4096
    }

    fn play(&mut self, _buffer: &[u8]) -> Result<(), DeviceError> {
        Ok(())
    }

    fn record(&mut self, _buffer: &mut [u8]) -> Result<usize, DeviceError> {
        Ok(0)
    }

    fn stop(&mut self) -> Result<(), DeviceError> {
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), DeviceError> {
        self.volume = volume.clamp(0.0, 1.0);
        Ok(())
    }

    fn volume(&self) -> f32 {
        self.volume
    }
}

/// Network Driver (Redox port)
pub struct RedoxNetworkDriver {
    name: String,
    vendor_id: u16,
    device_id: u16,
    mac_address: [u8; 6],
    base_address: usize,
    initialized: bool,
    mtu: u32,
    link_status: LinkStatus,
    statistics: NetworkStatistics,
    /// FASE 2: Graph kernel integration
    graph_kernel: Option<Cell<GraphKernel>>,
    graph_node_id: Option<NodeId>,
}

impl RedoxNetworkDriver {
    pub fn new(vendor_id: u16, device_id: u16, mac_address: [u8; 6], base_address: usize) -> Self {
        Self {
            name: String::from("redox-network"),
            vendor_id,
            device_id,
            mac_address,
            base_address,
            initialized: false,
            mtu: 1500,
            link_status: LinkStatus::Up,
            statistics: NetworkStatistics::default(),
            graph_kernel: None,
            graph_node_id: None,
        }
    }

    /// FASE 2: Set graph kernel reference
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// FASE 2: Register driver in graph kernel
    pub fn register_in_graph(&mut self) -> Result<NodeId, String> {
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Network);
            let node_name = format!("redox_network_{:04x}_{:04x}", self.vendor_id, self.device_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                self.graph_node_id = Some(id);
                Ok(id)
            } else {
                Err(String::from("Failed to create node in graph"))
            }
        } else {
            Err(String::from("Graph kernel not set"))
        }
    }
}

impl Device for RedoxNetworkDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.device_id
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        self.initialized = true;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.initialized = false;
        self.init()
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            dma: true,
            interrupt: true,
            mmio: true,
            pio: false,
        }
    }
}

impl NetworkDevice for RedoxNetworkDriver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac_address
    }

    fn mtu(&self) -> u32 {
        self.mtu
    }

    fn set_mtu(&mut self, mtu: u32) -> Result<(), DeviceError> {
        self.mtu = mtu;
        Ok(())
    }

    fn link_status(&self) -> LinkStatus {
        self.link_status.clone()
    }

    fn send_packet(&mut self, packet: &[u8]) -> Result<(), DeviceError> {
        self.statistics.bytes_sent += packet.len() as u64;
        self.statistics.packets_sent += 1;
        Ok(())
    }

    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, DeviceError> {
        self.statistics.packets_received += 1;
        Ok(0)
    }

    fn statistics(&self) -> NetworkStatistics {
        self.statistics.clone()
    }
}

/// Driver factory for creating wrapped drivers
pub struct DriverFactory;

impl DriverFactory {
    pub fn create_gpu(vendor_id: u16, device_id: u16, framebuffer: *mut u8, width: u32, height: u32) -> DeviceCapability<RedoxGpuDriver> {
        let driver = RedoxGpuDriver::new(vendor_id, device_id)
            .with_framebuffer(framebuffer, width, height);
        DeviceCapability::new(driver, CapabilityRights::FULL)
    }

    pub fn create_nvme(vendor_id: u16, device_id: u16, base_address: usize) -> DeviceCapability<RedoxNvmeDriver> {
        let driver = RedoxNvmeDriver::new(vendor_id, device_id, base_address);
        DeviceCapability::new(driver, CapabilityRights::FULL)
    }

    pub fn create_xhci(vendor_id: u16, device_id: u16, base_address: usize) -> DeviceCapability<RedoxXhciDriver> {
        let driver = RedoxXhciDriver::new(vendor_id, device_id, base_address);
        DeviceCapability::new(driver, CapabilityRights::FULL)
    }

    pub fn create_wifi(vendor_id: u16, device_id: u16, mac_address: [u8; 6]) -> DeviceCapability<RedoxWifiDriver> {
        let driver = RedoxWifiDriver::new(vendor_id, device_id, mac_address);
        DeviceCapability::new(driver, CapabilityRights::FULL)
    }

    pub fn create_audio(vendor_id: u16, device_id: u16, base_address: usize) -> DeviceCapability<RedoxAudioDriver> {
        let driver = RedoxAudioDriver::new(vendor_id, device_id, base_address);
        DeviceCapability::new(driver, CapabilityRights::FULL)
    }

    pub fn create_network(vendor_id: u16, device_id: u16, mac_address: [u8; 6], base_address: usize) -> DeviceCapability<RedoxNetworkDriver> {
        let driver = RedoxNetworkDriver::new(vendor_id, device_id, mac_address, base_address);
        DeviceCapability::new(driver, CapabilityRights::FULL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_driver_creation() {
        let driver = RedoxGpuDriver::new(0x1234, 0x5678);
        assert_eq!(driver.vendor_id(), 0x1234);
        assert_eq!(driver.device_id(), 0x5678);
    }

    #[test]
    fn test_nvme_driver_creation() {
        let driver = RedoxNvmeDriver::new(0x1234, 0x5678, 0x4000);
        assert_eq!(driver.vendor_id(), 0x1234);
        assert_eq!(driver.device_id(), 0x5678);
    }
}
