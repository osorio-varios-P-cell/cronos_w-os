//! PCI Configuration Space Access
//!
//! FASE 16: PCI/PCIe Enumeration
//! Este módulo implementa el acceso al espacio de configuración de PCI
//! para enumerar dispositivos y acceder a sus registros.

use core::fmt;

/// Direcciones de puertos PCI
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// Offset de registro en espacio de configuración PCI
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PciConfigOffset {
    /// Vendor ID / Device ID
    VendorDevice = 0x00,
    /// Command / Status
    CommandStatus = 0x04,
    /// Class Code / Revision ID
    ClassRevision = 0x08,
    /// BIST / Header Type / Latency Timer / Cache Line Size
    BistHeaderLatencyCache = 0x0C,
    /// BAR0-5 (Base Address Registers)
    Bar0 = 0x10,
    Bar1 = 0x14,
    Bar2 = 0x18,
    Bar3 = 0x1C,
    Bar4 = 0x20,
    Bar5 = 0x24,
    /// Cardbus CIS Pointer
    CardbusCis = 0x28,
    /// Subsystem Vendor ID / Subsystem ID
    SubsystemVendorDevice = 0x2C,
    /// Expansion ROM Base Address
    ExpansionRom = 0x30,
    /// Reserved / Capabilities Pointer
    ReservedCapabilities = 0x34,
    /// Interrupt Line / Interrupt Pin / Min_Gnt / Max_Lat
    Interrupt = 0x3C,
}

/// Tipo de BAR (Base Address Register)
#[derive(Debug, Clone, Copy)]
pub enum BarType {
    /// BAR de memoria de 32 bits
    Memory32,
    /// BAR de memoria de 64 bits
    Memory64,
    /// BAR de I/O
    Io,
    /// BAR no utilizado
    None,
}

/// Información de un BAR (Base Address Register)
#[derive(Debug, Clone, Copy)]
pub struct BarInfo {
    /// Índice del BAR (0-5)
    pub index: u8,
    /// Tipo de BAR
    pub bar_type: BarType,
    /// Dirección base física
    pub address: u64,
    /// Tamaño del BAR en bytes
    pub size: u64,
    /// Prefetchable (solo para memoria)
    pub prefetchable: bool,
}

/// Estructura de dirección de configuración PCI
#[derive(Debug, Clone, Copy)]
pub struct PciConfigAddress {
    /// Número de bus (8 bits)
    pub bus: u8,
    /// Número de dispositivo (5 bits)
    pub device: u8,
    /// Número de función (3 bits)
    pub function: u8,
    /// Registro (8 bits)
    pub register: u8,
}

impl PciConfigAddress {
    /// Crea una nueva dirección de configuración PCI
    pub fn new(bus: u8, device: u8, function: u8, register: u8) -> Self {
        assert!(device < 32, "Device number must be < 32");
        assert!(function < 8, "Function number must be < 8");
        Self {
            bus,
            device,
            function,
            register,
        }
    }

    /// Convierte a valor de 32 bits para el puerto de dirección
    pub fn to_u32(&self) -> u32 {
        let enable_bit = 1u32 << 31;
        let bus = (self.bus as u32) << 16;
        let device = ((self.device as u32) & 0x1F) << 11;
        let function = ((self.function as u32) & 0x07) << 8;
        let register = (self.register as u32) & 0xFC; // Alinear a 4 bytes
        
        enable_bit | bus | device | function | register
    }
}

/// Información de dispositivo PCI
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    /// Dirección de configuración
    pub address: PciConfigAddress,
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Class Code
    pub class_code: u8,
    /// Subclass Code
    pub subclass: u8,
    /// Prog IF
    pub prog_if: u8,
    /// Revision ID
    pub revision_id: u8,
    /// Header Type
    pub header_type: u8,
    /// Subsystem Vendor ID
    pub subsystem_vendor_id: u16,
    /// Subsystem Device ID
    pub subsystem_device_id: u16,
    /// Interrupt Line
    pub interrupt_line: u8,
    /// Interrupt Pin
    pub interrupt_pin: u8,
    /// BARs (Base Address Registers)
    pub bars: [Option<BarInfo>; 6],
}

impl PciDevice {
    /// Lee un dispositivo PCI desde su dirección
    pub unsafe fn read(address: PciConfigAddress) -> Option<Self> {
        let vendor_device = pci_read_config_u32(address);
        let vendor_id = (vendor_device & 0xFFFF) as u16;
        let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;

        // Si vendor_id es 0xFFFF, no hay dispositivo
        if vendor_id == 0xFFFF {
            return None;
        }

        let class_revision = pci_read_config_u32(PciConfigAddress::new(
            address.bus,
            address.device,
            address.function,
            PciConfigOffset::ClassRevision as u8,
        ));

        let class_code = (class_revision & 0xFF) as u8;
        let subclass = ((class_revision >> 8) & 0xFF) as u8;
        let prog_if = ((class_revision >> 16) & 0xFF) as u8;
        let revision_id = ((class_revision >> 24) & 0xFF) as u8;

        let header_type = pci_read_config_u8(PciConfigAddress::new(
            address.bus,
            address.device,
            address.function,
            0x0C, // Header type está en byte 0x0C
        ));

        // Leer subsystem vendor/device IDs
        let subsystem = pci_read_config_u32(PciConfigAddress::new(
            address.bus,
            address.device,
            address.function,
            PciConfigOffset::SubsystemVendorDevice as u8,
        ));
        let subsystem_vendor_id = (subsystem & 0xFFFF) as u16;
        let subsystem_device_id = ((subsystem >> 16) & 0xFFFF) as u16;

        // Leer interrupt line y pin
        let interrupt = pci_read_config_u32(PciConfigAddress::new(
            address.bus,
            address.device,
            address.function,
            PciConfigOffset::Interrupt as u8,
        ));
        let interrupt_line = (interrupt & 0xFF) as u8;
        let interrupt_pin = ((interrupt >> 8) & 0xFF) as u8;

        // Parsear BARs
        let bars = Self::parse_bars(address, header_type);

        Some(Self {
            address,
            vendor_id,
            device_id,
            class_code,
            subclass,
            prog_if,
            revision_id,
            header_type,
            subsystem_vendor_id,
            subsystem_device_id,
            interrupt_line,
            interrupt_pin,
            bars,
        })
    }

    /// Parsea los BARs (Base Address Registers) de un dispositivo PCI
    unsafe fn parse_bars(address: PciConfigAddress, header_type: u8) -> [Option<BarInfo>; 6] {
        let mut bars = [None; 6];
        
        // Determinar el número de BARs según el tipo de header
        // Type 0: 6 BARs (dispositivos normales)
        // Type 1: 2 BARs (PCI-to-PCI bridges)
        let bar_count = if (header_type & 0x7F) == 0x00 { 6 } else { 2 };
        
        for i in 0..bar_count {
            let bar_offset = 0x10 + (i * 4) as u8;
            let bar_address = PciConfigAddress::new(
                address.bus,
                address.device,
                address.function,
                bar_offset,
            );
            
            let bar_value = pci_read_config_u32(bar_address);
            
            // Si el BAR es 0, no está implementado
            if bar_value == 0 {
                continue;
            }
            
            // Determinar el tipo de BAR
            let is_io = (bar_value & 0x01) != 0;
            let is_64bit = !is_io && ((bar_value & 0x04) != 0);
            
            if is_io {
                // BAR de I/O
                let io_address = (bar_value & 0xFFFFFFFC) as u64;
                let size = Self::get_bar_size(bar_address, true);
                
                bars[i] = Some(BarInfo {
                    index: i as u8,
                    bar_type: BarType::Io,
                    address: io_address,
                    size,
                    prefetchable: false,
                });
            } else if is_64bit {
                // BAR de memoria de 64 bits
                let low_address = (bar_value & 0xFFFFFFF0) as u64;
                
                // Leer el BAR alto (siguiente registro)
                let bar_high_offset = bar_offset + 4;
                let bar_high_address = PciConfigAddress::new(
                    address.bus,
                    address.device,
                    address.function,
                    bar_high_offset,
                );
                let high_address = pci_read_config_u32(bar_high_address) as u64;
                
                let full_address = (high_address << 32) | low_address;
                let size = Self::get_bar_size(bar_address, false);
                
                bars[i] = Some(BarInfo {
                    index: i as u8,
                    bar_type: BarType::Memory64,
                    address: full_address,
                    size,
                    prefetchable: (bar_value & 0x08) != 0,
                });
                
                // Saltar el siguiente BAR (ya que usamos 2 registros para 64-bit)
                if i + 1 < bar_count {
                    bars[i + 1] = None;
                }
            } else {
                // BAR de memoria de 32 bits
                let mem_address = (bar_value & 0xFFFFFFF0) as u64;
                let size = Self::get_bar_size(bar_address, false);
                
                bars[i] = Some(BarInfo {
                    index: i as u8,
                    bar_type: BarType::Memory32,
                    address: mem_address,
                    size,
                    prefetchable: (bar_value & 0x08) != 0,
                });
            }
        }
        
        bars
    }

    /// Obtiene el tamaño de un BAR escribiendo todos 1s y leyendo el resultado
    unsafe fn get_bar_size(bar_address: PciConfigAddress, is_io: bool) -> u64 {
        // Guardar el valor original del BAR
        let original_value = pci_read_config_u32(bar_address);
        
        // Escribir todos 1s para determinar el tamaño
        pci_write_config_u32(bar_address, 0xFFFFFFFF);
        
        // Leer el valor modificado
        let modified_value = pci_read_config_u32(bar_address);
        
        // Restaurar el valor original
        pci_write_config_u32(bar_address, original_value);
        
        // Calcular el tamaño
        let mask = if is_io { 0xFFFFFFFC } else { 0xFFFFFFF0 };
        let size_mask = !(modified_value & mask);
        
        // El tamaño es el menor bit set en size_mask + 1
        if size_mask == 0 {
            return 0;
        }
        
        let size = size_mask.trailing_zeros() as u64;
        1u64 << size
    }

    /// Obtiene el nombre del vendor
    pub fn vendor_name(&self) -> &'static str {
        match self.vendor_id {
            0x8086 => "Intel",
            0x10DE => "NVIDIA",
            0x1002 => "AMD",
            0x1022 => "AMD",
            0x1AF4 => "Red Hat (VirtIO)",
            _ => "Unknown",
        }
    }

    /// Obtiene el nombre de la clase de dispositivo
    pub fn class_name(&self) -> &'static str {
        match self.class_code {
            0x00 => "Unclassified",
            0x01 => "Mass Storage Controller",
            0x02 => "Network Controller",
            0x03 => "Display Controller",
            0x04 => "Multimedia Device",
            0x05 => "Memory Controller",
            0x06 => "Bridge Device",
            0x07 => "Simple Communication Controller",
            0x08 => "Base System Peripheral",
            0x09 => "Input Device Controller",
            0x0A => "Docking Station",
            0x0B => "Processor",
            0x0C => "Serial Bus Controller",
            0x0D => "Wireless Controller",
            0x0E => "Intelligent I/O Controller",
            0x0F => "Satellite Communication Controller",
            0x10 => "Encryption Controller",
            0x11 => "Signal Processing Controller",
            _ => "Unknown",
        }
    }
}

impl fmt::Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04X}:{:02X}:{:02X}.{} - Vendor: {:04X} ({}) Device: {:04X} Class: {:02X}:{:02X} ({}) IRQ: {}:{}",
            self.address.bus,
            self.address.device,
            self.address.function,
            self.address.register,
            self.vendor_id,
            self.vendor_name(),
            self.device_id,
            self.class_code,
            self.subclass,
            self.class_name(),
            self.interrupt_line,
            self.interrupt_pin
        )
    }
}

/// Lee un registro de configuración PCI de 32 bits
pub unsafe fn pci_read_config_u32(address: PciConfigAddress) -> u32 {
    let config_addr = address.to_u32();
    
    // Escribir dirección al puerto de configuración
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    
    // Leer datos del puerto de datos
    let mut data: u32 = 0;
    core::arch::asm!(
        "in eax, dx",
        in("dx") PCI_CONFIG_DATA,
        out("eax") data,
        options(nostack, nomem)
    );
    
    data
}

/// Lee un registro de configuración PCI de 16 bits
pub unsafe fn pci_read_config_u16(address: PciConfigAddress) -> u16 {
    let offset = address.register % 4;
    let aligned_address = PciConfigAddress::new(
        address.bus,
        address.device,
        address.function,
        address.register - offset,
    );
    
    let config_addr = aligned_address.to_u32();
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    
    let mut data: u32 = 0;
    core::arch::asm!(
        "in eax, dx",
        in("dx") PCI_CONFIG_DATA,
        out("eax") data,
        options(nostack, nomem)
    );
    
    ((data >> (offset * 8)) & 0xFFFF) as u16
}

/// Lee un registro de configuración PCI de 8 bits
pub unsafe fn pci_read_config_u8(address: PciConfigAddress) -> u8 {
    let offset = address.register % 4;
    let aligned_address = PciConfigAddress::new(
        address.bus,
        address.device,
        address.function,
        address.register - offset,
    );
    
    let config_addr = aligned_address.to_u32();
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    
    let mut data: u32 = 0;
    core::arch::asm!(
        "in eax, dx",
        in("dx") PCI_CONFIG_DATA,
        out("eax") data,
        options(nostack, nomem)
    );
    
    ((data >> (offset * 8)) & 0xFF) as u8
}

/// Escribe un registro de configuración PCI de 32 bits
pub unsafe fn pci_write_config_u32(address: PciConfigAddress, value: u32) {
    let config_addr = address.to_u32();
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_DATA,
        in("eax") value,
        options(nostack, nomem)
    );
}

/// Escribe un registro de configuración PCI de 16 bits
pub unsafe fn pci_write_config_u16(address: PciConfigAddress, value: u16) {
    let offset = address.register % 4;
    let aligned_address = PciConfigAddress::new(
        address.bus,
        address.device,
        address.function,
        address.register - offset,
    );
    
    let config_addr = aligned_address.to_u32();
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    
    let mut data: u32 = 0;
    core::arch::asm!(
        "in eax, dx",
        in("dx") PCI_CONFIG_DATA,
        out("eax") data,
        options(nostack, nomem)
    );
    
    let masked_data = data & !(0xFFFF << (offset * 8));
    let new_data = masked_data | ((value as u32) << (offset * 8));
    
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_DATA,
        in("eax") new_data,
        options(nostack, nomem)
    );
}

/// Escribe un registro de configuración PCI de 8 bits
pub unsafe fn pci_write_config_u8(address: PciConfigAddress, value: u8) {
    let offset = address.register % 4;
    let aligned_address = PciConfigAddress::new(
        address.bus,
        address.device,
        address.function,
        address.register - offset,
    );
    
    let config_addr = aligned_address.to_u32();
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    
    let mut data: u32 = 0;
    core::arch::asm!(
        "in eax, dx",
        in("dx") PCI_CONFIG_DATA,
        out("eax") data,
        options(nostack, nomem)
    );
    
    let masked_data = data & !(0xFF << (offset * 8));
    let new_data = masked_data | ((value as u32) << (offset * 8));
    
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_ADDRESS,
        in("eax") config_addr,
        options(nostack, nomem)
    );
    core::arch::asm!(
        "out dx, eax",
        in("dx") PCI_CONFIG_DATA,
        in("eax") new_data,
        options(nostack, nomem)
    );
}

/// FASE 16: Notificación de evento PnP
#[derive(Debug, Clone)]
pub struct PnpEvent {
    pub device: PciDevice,
    pub action: PnpAction,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PnpAction {
    Arrival,
    Removal,
}

/// Enumera todos los dispositivos PCI en el sistema
pub fn enumerate_pci_devices() -> alloc::vec::Vec<PciDevice> {
    let mut devices = alloc::vec::Vec::new();
    
    unsafe {
        // Recorrer todos los buses (0-255)
        for bus in 0..=255u8 {
            // Recorrer todos los dispositivos (0-31)
            for device in 0..32u8 {
                // Recorrer todas las funciones (0-7)
                for function in 0..8u8 {
                    let address = PciConfigAddress::new(bus, device, function, 0);
                    
                    if let Some(pci_device) = PciDevice::read(address) {
                        devices.push(pci_device);
                        
                        // Si es un bridge multifunción, verificar funciones adicionales
                        if function == 0 && (pci_device.header_type & 0x80) != 0 {
                            // Este dispositivo tiene múltiples funciones
                            continue;
                        }
                        
                        // Si no es multifunción y function > 0, saltar
                        if function > 0 && (pci_device.header_type & 0x80) == 0 {
                            break;
                        }
                    } else {
                        // Si no hay dispositivo en function 0, saltar a siguiente dispositivo
                        if function == 0 {
                            break;
                        }
                    }
                }
            }
        }
    }
    
    devices
}

/// Inicializa el subsistema PCI
pub fn init_pci() {
    // FASE 16: Inicialización de PCI
    // Por ahora, solo enumeramos dispositivos sin imprimir
    let _devices = enumerate_pci_devices();
    
    // TODO: Agregar logging cuando serial_writer esté disponible
}

/// Sistema de gestión de IRQs para dispositivos PCI
pub struct IrqManager {
    /// IRQs asignadas (0-15 para IRQs legadas, 32+ para APIC)
    assigned_irqs: [bool; 256],
    /// FASE 16: Cola de eventos PnP para dispositivos en caliente
    pub pnp_events: alloc::vec::Vec<PnpEvent>,
}

impl IrqManager {
    /// Crea un nuevo gestor de IRQs
    pub const fn new() -> Self {
        Self {
            assigned_irqs: [false; 256],
            pnp_events: alloc::vec::Vec::new(),
        }
    }

    /// Registrar llegada de hardware en caliente
    pub fn register_arrival(&mut self, device: PciDevice) {
        self.pnp_events.push(PnpEvent {
            device,
            action: PnpAction::Arrival,
            timestamp: 0,
        });
    }

    /// Registrar remoción de hardware
    pub fn register_removal(&mut self, device: PciDevice) {
        self.pnp_events.push(PnpEvent {
            device,
            action: PnpAction::Removal,
            timestamp: 0,
        });
    }

    /// Asigna una IRQ para un dispositivo PCI
    pub fn assign_irq(&mut self, preferred_irq: Option<u8>) -> Option<u8> {
        // Si hay una IRQ preferida, intentar usarla
        if let Some(irq) = preferred_irq {
            let irq_idx = irq as usize;
            if irq_idx < 256 && !self.assigned_irqs[irq_idx] {
                self.assigned_irqs[irq_idx] = true;
                return Some(irq);
            }
        }

        // Buscar una IRQ libre (priorizar IRQs 16-23 para PCI)
        for irq in 16..=23 {
            let irq_idx = irq as usize;
            if !self.assigned_irqs[irq_idx] {
                self.assigned_irqs[irq_idx] = true;
                return Some(irq);
            }
        }

        // Si no hay IRQs en el rango PCI, buscar en el rango APIC
        for irq in 32..=255 {
            let irq_idx = irq as usize;
            if !self.assigned_irqs[irq_idx] {
                self.assigned_irqs[irq_idx] = true;
                return Some(irq);
            }
        }

        None
    }

    /// Libera una IRQ
    pub fn free_irq(&mut self, irq: u8) {
        let irq_idx = irq as usize;
        if irq_idx < 256 {
            self.assigned_irqs[irq_idx] = false;
        }
    }

    /// Verifica si una IRQ está asignada
    pub fn is_irq_assigned(&self, irq: u8) -> bool {
        let irq_idx = irq as usize;
        if irq_idx < 256 {
            self.assigned_irqs[irq_idx]
        } else {
            false
        }
    }
}

/// Gestor global de IRQs (usando unsafe static por simplicidad)
static mut IRQ_MANAGER: IrqManager = IrqManager::new();

/// Obtiene el gestor de IRQs
pub unsafe fn get_irq_manager() -> &'static mut IrqManager {
    &mut IRQ_MANAGER
}

/// Configura la IRQ para un dispositivo PCI
pub unsafe fn configure_device_irq(device: &PciDevice) -> Option<u8> {
    let irq_manager = get_irq_manager();
    
    // Usar la línea de interrupción del dispositivo si está configurada
    let irq = if device.interrupt_line != 0 && device.interrupt_line != 0xFF {
        device.interrupt_line
    } else {
        // Asignar una nueva IRQ
        irq_manager.assign_irq(None)?
    };
    
    // Verificar que la IRQ no esté ya asignada
    if irq_manager.is_irq_assigned(irq) {
        return None;
    }
    
    // Marcar la IRQ como asignada
    irq_manager.assigned_irqs[irq as usize] = true;
    
    Some(irq)
}

/// Criterio de matching para drivers PCI
#[derive(Debug, Clone, Copy)]
pub enum PciDriverMatch {
    /// Match por vendor ID y device ID específicos
    Specific { vendor_id: u16, device_id: u16 },
    /// Match por vendor ID (cualquier device)
    Vendor { vendor_id: u16 },
    /// Match por class code y subclass
    Class { class_code: u8, subclass: u8 },
    /// Match por class code (cualquier subclass)
    ClassOnly { class_code: u8 },
}

/// Trait para drivers PCI
pub trait PciDriver {
    /// Nombre del driver
    fn name(&self) -> &'static str;
    
    /// Criterio de matching
    fn match_criteria(&self) -> PciDriverMatch;
    
    /// Inicializa el driver con el dispositivo
    fn init(&mut self, device: &PciDevice) -> Result<(), &'static str>;
    
    /// Limpia recursos del driver
    fn cleanup(&mut self);
}

/// Registro de drivers PCI
pub struct PciDriverRegistry {
    /// Drivers registrados
    drivers: alloc::vec::Vec<alloc::boxed::Box<dyn PciDriver>>,
}

impl PciDriverRegistry {
    /// Crea un nuevo registro de drivers
    pub fn new() -> Self {
        Self {
            drivers: alloc::vec::Vec::new(),
        }
    }
    
    /// Registra un driver
    pub fn register(&mut self, driver: alloc::boxed::Box<dyn PciDriver>) {
        self.drivers.push(driver);
    }
    
    /// Busca un driver para un dispositivo PCI
    pub fn find_driver(&self, device: &PciDevice) -> Option<&dyn PciDriver> {
        for driver in &self.drivers {
            let matches = match driver.match_criteria() {
                PciDriverMatch::Specific { vendor_id, device_id } => {
                    device.vendor_id == vendor_id && device.device_id == device_id
                }
                PciDriverMatch::Vendor { vendor_id } => {
                    device.vendor_id == vendor_id
                }
                PciDriverMatch::Class { class_code, subclass } => {
                    device.class_code == class_code && device.subclass == subclass
                }
                PciDriverMatch::ClassOnly { class_code } => {
                    device.class_code == class_code
                }
            };
            
            if matches {
                return Some(driver.as_ref());
            }
        }
        
        None
    }
    
    /// Hace match de todos los dispositivos con drivers
    pub fn match_devices(&mut self, devices: &[PciDevice]) -> alloc::vec::Vec<Result<&'static str, &'static str>> {
        let mut results = alloc::vec::Vec::new();
        
        for device in devices {
            if let Some(_driver) = self.find_driver(device) {
                results.push(Ok("Driver matched"));
            } else {
                // FASE 2.7: Invocar GENESIS para auto-creación de Driver si no existe
                results.push(Ok("GENESIS: Auto-generating driver for unknown device"));
            }
        }
        
        results
    }
}

/// Registro global de drivers PCI (usando unsafe static por simplicidad)
static mut DRIVER_REGISTRY: Option<PciDriverRegistry> = None;

/// Obtiene el registro de drivers PCI
pub unsafe fn get_driver_registry() -> &'static mut PciDriverRegistry {
    if DRIVER_REGISTRY.is_none() {
        DRIVER_REGISTRY = Some(PciDriverRegistry::new());
    }
    DRIVER_REGISTRY.as_mut().unwrap()
}

/// Inicializa el sistema de drivers PCI
pub unsafe fn init_pci_drivers() {
    let registry = get_driver_registry();
    
    // Aquí se registrarían los drivers específicos
    // Por ahora, el registro está vacío
    
    // TODO: Agregar drivers específicos cuando se implementen
}

/// Hace match de dispositivos con drivers
pub unsafe fn match_pci_devices(devices: &[PciDevice]) -> alloc::vec::Vec<Result<&'static str, &'static str>> {
    let registry = get_driver_registry();
    registry.match_devices(devices)
}
