//! ACPI (Advanced Configuration and Power Interface) para CRONOS W-OS
//!
//! Este módulo implementa ACPI para gestión de energía y hardware,
//! adaptado a la arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::{BTreeMap, BTreeSet};
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Estado de energía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerState {
    /// Trabajando (S0)
    Working,
    /// Sueño (S1-S3)
    Sleep,
    /// Hibernación (S4)
    Hibernate,
    /// Apagado suave (S5)
    SoftOff,
    /// Apagado mecánico (G3)
    MechanicalOff,
}

/// C-state (CPU power state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CState {
    /// C0 - CPU ejecutando
    C0,
    /// C1 - Halt
    C1,
    /// C2 - Stop Grant
    C2,
    /// C3 - Sleep
    C3,
    /// C4 - Deep Sleep
    C4,
    /// C5 - Deep Sleep Plus
    C5,
    /// C6 - Deep Power Down
    C6,
}

impl CState {
    /// Obtener el nivel de profundidad del C-state
    pub fn depth(&self) -> u8 {
        match self {
            CState::C0 => 0,
            CState::C1 => 1,
            CState::C2 => 2,
            CState::C3 => 3,
            CState::C4 => 4,
            CState::C5 => 5,
            CState::C6 => 6,
        }
    }

    /// Obtener el nombre del C-state
    pub fn name(&self) -> &'static str {
        match self {
            CState::C0 => "C0",
            CState::C1 => "C1",
            CState::C2 => "C2",
            CState::C3 => "C3",
            CState::C4 => "C4",
            CState::C5 => "C5",
            CState::C6 => "C6",
        }
    }
}

/// Información de C-state
#[derive(Debug, Clone)]
pub struct CStateInfo {
    pub c_state: CState,
    pub latency: u32, // Latencia en microsegundos
    pub power: u32, // Consumo de energía en miliwatts
    pub supported: bool,
}

impl CStateInfo {
    pub fn new(c_state: CState, latency: u32, power: u32) -> Self {
        Self {
            c_state,
            latency,
            power,
            supported: true,
        }
    }
}

/// Gestor de C-states
pub struct CStateManager {
    pub c_states: Vec<CStateInfo>,
    pub current_c_state: CState,
}

impl CStateManager {
    pub fn new() -> Self {
        Self {
            c_states: Vec::new(),
            current_c_state: CState::C0,
        }
    }

    /// Agregar un C-state soportado
    pub fn add_c_state(&mut self, c_state: CState, latency: u32, power: u32) {
        self.c_states.push(CStateInfo::new(c_state, latency, power));
    }

    /// Obtener el C-state más profundo disponible
    pub fn get_deepest_c_state(&self) -> Option<CState> {
        self.c_states.iter()
            .filter(|cs| cs.supported)
            .map(|cs| cs.c_state)
            .max_by_key(|cs| cs.depth())
    }

    /// Entrar en un C-state específico
    pub fn enter_c_state(&mut self, c_state: CState) -> Result<(), String> {
        // Verificar si el C-state es soportado
        if !self.c_states.iter().any(|cs| cs.c_state == c_state && cs.supported) {
            return Err(format!("C-state {} not supported", c_state.name()));
        }

        // En un sistema real, aquí se:
        // 1. Verificar si el C-state es seguro de entrar
        // 2. Configurar los registros de energía
        // 3. Ejecutar la instrucción de espera (HLT, MWAIT, etc.)

        self.current_c_state = c_state;
        Ok(())
    }

    /// Obtener el C-state actual
    pub fn current_c_state(&self) -> CState {
        self.current_c_state
    }

    /// Calcular el C-state óptimo basado en la latencia permitida
    pub fn calculate_optimal_c_state(&self, max_latency: u32) -> Option<CState> {
        self.c_states.iter()
            .filter(|cs| cs.supported && cs.latency <= max_latency)
            .max_by_key(|cs| cs.c_state.depth())
            .map(|cs| cs.c_state)
    }
}

impl Default for CStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estado del sistema ACPI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpiState {
    /// No inicializado
    Uninitialized,
    /// Buscando RSDP
    SearchingRsdp,
    /// Analizando RSDT
    ParsingRsdt,
    /// Analizando FADT
    ParsingFadt,
    /// Listo
    Ready,
    /// Error
    Error(String),
}

/// Tabla ACPI
#[derive(Debug, Clone)]
pub struct AcpiTable {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub asl_compiler_id: [u8; 4],
    pub asl_compiler_revision: u32,
}

impl AcpiTable {
    pub fn new(signature: [u8; 4], length: u32) -> Self {
        Self {
            signature,
            length,
            revision: 0,
            checksum: 0,
            oem_id: [0; 6],
            oem_table_id: [0; 8],
            oem_revision: 0,
            asl_compiler_id: [0; 4],
            asl_compiler_revision: 0,
        }
    }

    /// Verificar si la firma es válida
    pub fn is_valid(&self) -> bool {
        self.signature.iter().all(|&b| b != 0)
    }

    /// Verificar el checksum de la tabla ACPI
    pub fn verify_checksum(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const AcpiTable as *const u8,
                self.length as usize
            )
        };
        let sum: u8 = bytes.iter().fold(0, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }

    /// Obtener la firma como string
    pub fn signature_str(&self) -> String {
        String::from_utf8_lossy(&self.signature).to_string()
    }
}

/// RSDP (Root System Description Pointer)
#[derive(Debug, Clone)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
}

impl Rsdp {
    pub fn new() -> Self {
        Self {
            signature: [b'R', b'S', b'D', b' ', b'P', b'T', b'R', b' '],
            checksum: 0,
            oem_id: [0; 6],
            revision: 0,
            rsdt_address: 0,
            length: 0,
            xsdt_address: 0,
            extended_checksum: 0,
        }
    }

    /// Verificar si la firma es válida
    pub fn is_valid(&self) -> bool {
        &self.signature == b"RSD PTR "
    }

    /// Verificar el checksum del RSDP (ACPI 1.0)
    pub fn verify_checksum(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const Rsdp as *const u8,
                20 // Primeros 20 bytes para ACPI 1.0
            )
        };
        let sum: u8 = bytes.iter().fold(0, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }

    /// Verificar el checksum extendido del RSDP (ACPI 2.0+)
    pub fn verify_extended_checksum(&self) -> bool {
        if self.revision < 2 {
            return true; // ACPI 1.0 no tiene checksum extendido
        }
        
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const Rsdp as *const u8,
                self.length as usize
            )
        };
        let sum: u8 = bytes.iter().fold(0, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }
}

impl Default for Rsdp {
    fn default() -> Self {
        Self::new()
    }
}

/// FADT (Fixed ACPI Description Table)
#[derive(Debug, Clone)]
pub struct Fadt {
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
}

/// Tipos de entradas MADT
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MadtEntryType {
    /// Processor Local APIC
    ProcessorLocalApic = 0,
    /// I/O APIC
    IoApic = 1,
    /// Interrupt Source Override
    InterruptSourceOverride = 2,
    /// NMI Source
    NmiSource = 3,
    /// Local APIC NMI
    LocalApicNmi = 4,
    /// Local APIC Address Override
    LocalApicAddressOverride = 5,
    /// I/O SAPIC
    IoSapic = 6,
    /// Local SAPIC
    LocalSapic = 7,
    /// Platform Interrupt Sources
    PlatformInterruptSource = 8,
    /// Processor Local x2APIC
    ProcessorLocalX2Apic = 9,
    /// Local x2APIC NMI
    LocalX2ApicNmi = 10,
}

/// Entrada de Processor Local APIC en MADT
#[derive(Debug, Clone, Copy)]
pub struct ProcessorLocalApicEntry {
    pub acpi_processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

/// Entrada de I/O APIC en MADT
#[derive(Debug, Clone, Copy)]
pub struct IoApicEntry {
    pub io_apic_id: u8,
    pub reserved: u8,
    pub io_apic_address: u32,
    pub global_system_interrupt_base: u32,
}

/// MADT (Multiple APIC Description Table)
#[derive(Debug, Clone)]
pub struct Madt {
    pub local_apic_address: u32,
    pub flags: u32,
    pub processors: Vec<ProcessorLocalApicEntry>,
    pub io_apics: Vec<IoApicEntry>,
}

/// AML (ACPI Machine Language) Opcode
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AmlOpcode {
    /// ZeroOp
    Zero = 0x00,
    /// OneOp
    One = 0x01,
    /// ByteConst
    ByteConst = 0x0A,
    /// WordConst
    WordConst = 0x0B,
    /// DWordConst
    DWordConst = 0x0C,
    /// String
    String = 0x0D,
    /// ScopeOp
    Scope = 0x10,
    /// DeviceOp
    Device = 0x82,
    /// ProcessorOp
    Processor = 0x5B,
    /// MethodOp
    Method = 0x14,
    /// IfOp
    If = 0xA0,
    /// ElseOp
    Else = 0xA1,
    /// ReturnOp
    Return = 0xA4,
    /// NameOp
    Name = 0x08,
}

/// AML Value
#[derive(Debug, Clone)]
pub enum AmlValue {
    Integer(u64),
    String(String),
    Buffer(Vec<u8>),
    Package(Vec<AmlValue>),
    Reference(String),
    Uninitialized,
}

/// AML Interpreter
pub struct AmlInterpreter {
    pub dsdt: Option<AcpiTable>,
    pub namespace: BTreeMap<String, AmlValue>,
}

impl AmlInterpreter {
    pub fn new() -> Self {
        Self {
            dsdt: None,
            namespace: BTreeMap::new(),
        }
    }

    /// Establecer la tabla DSDT
    pub fn set_dsdt(&mut self, dsdt: AcpiTable) {
        self.dsdt = Some(dsdt);
    }

    /// Parsear un byte desde el código AML
    unsafe fn parse_byte(&self, offset: usize) -> Option<u8> {
        if let Some(ref dsdt) = self.dsdt {
            let dsdt_ptr = dsdt as *const AcpiTable as *const u8;
            if offset < dsdt.length as usize {
                Some(*dsdt_ptr.add(offset))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parsear un opcode AML
    unsafe fn parse_opcode(&self, offset: usize) -> Option<AmlOpcode> {
        let byte = self.parse_byte(offset)?;
        match byte {
            0x00 => Some(AmlOpcode::Zero),
            0x01 => Some(AmlOpcode::One),
            0x0A => Some(AmlOpcode::ByteConst),
            0x0B => Some(AmlOpcode::WordConst),
            0x0C => Some(AmlOpcode::DWordConst),
            0x0D => Some(AmlOpcode::String),
            0x10 => Some(AmlOpcode::Scope),
            0x82 => Some(AmlOpcode::Device),
            0x5B => Some(AmlOpcode::Processor),
            0x14 => Some(AmlOpcode::Method),
            0xA0 => Some(AmlOpcode::If),
            0xA1 => Some(AmlOpcode::Else),
            0xA4 => Some(AmlOpcode::Return),
            0x08 => Some(AmlOpcode::Name),
            _ => None,
        }
    }

    /// Parsear un nombre AML (4 caracteres)
    unsafe fn parse_name(&self, offset: usize) -> Option<String> {
        if let Some(ref dsdt) = self.dsdt {
            let dsdt_ptr = dsdt as *const AcpiTable as *const u8;
            if offset + 4 <= dsdt.length as usize {
                let name = core::str::from_utf8_unchecked(
                    core::slice::from_raw_parts(dsdt_ptr.add(offset), 4)
                );
                Some(String::from(name))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parsear un valor AML
    unsafe fn parse_value(&self, offset: usize) -> Option<(AmlValue, usize)> {
        let opcode = self.parse_opcode(offset)?;
        
        match opcode {
            AmlOpcode::Zero => Some((AmlValue::Integer(0), 1)),
            AmlOpcode::One => Some((AmlValue::Integer(1), 1)),
            AmlOpcode::ByteConst => {
                let value = self.parse_byte(offset + 1)?;
                Some((AmlValue::Integer(value as u64), 2))
            }
            AmlOpcode::WordConst => {
                if let Some(ref dsdt) = self.dsdt {
                    let dsdt_ptr = dsdt as *const AcpiTable as *const u8;
                    let value = *(dsdt_ptr.add(offset + 1) as *const u16);
                    Some((AmlValue::Integer(value as u64), 3))
                } else {
                    None
                }
            }
            AmlOpcode::DWordConst => {
                if let Some(ref dsdt) = self.dsdt {
                    let dsdt_ptr = dsdt as *const AcpiTable as *const u8;
                    let value = *(dsdt_ptr.add(offset + 1) as *const u32);
                    Some((AmlValue::Integer(value as u64), 5))
                } else {
                    None
                }
            }
            AmlOpcode::String => {
                if let Some(ref dsdt) = self.dsdt {
                    let dsdt_ptr = dsdt as *const AcpiTable as *const u8;
                    let mut string_end = offset + 1;
                    while string_end < dsdt.length as usize && *dsdt_ptr.add(string_end) != 0 {
                        string_end += 1;
                    }
                    let string = core::str::from_utf8_unchecked(
                        core::slice::from_raw_parts(dsdt_ptr.add(offset + 1), string_end - offset - 1)
                    );
                    Some((AmlValue::String(String::from(string)), string_end - offset + 1))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Ejecutar el código AML (simplificado)
    pub unsafe fn execute(&mut self) -> Result<(), String> {
        if self.dsdt.is_none() {
            return Err(String::from("DSDT not set"));
        }

        let dsdt = self.dsdt.as_ref().unwrap();
        let dsdt_ptr = dsdt as *const AcpiTable as *const u8;
        
        // Comenzar después del header (36 bytes)
        let mut offset = 36;
        
        while offset < dsdt.length as usize {
            let opcode = self.parse_opcode(offset);
            
            if let Some(opcode) = opcode {
                match opcode {
                    AmlOpcode::Name => {
                        // Parsear Name (NameOp, NameString, Value)
                        let name = self.parse_name(offset + 1);
                        if let Some(name) = name {
                            if let Some((value, size)) = self.parse_value(offset + 5) {
                                self.namespace.insert(name, value);
                                offset += 5 + size;
                            } else {
                                offset += 5;
                            }
                        } else {
                            offset += 1;
                        }
                    }
                    AmlOpcode::Scope | AmlOpcode::Device => {
                        // Parsear Scope/Device (PkgLength, Name, TermList)
                        let pkg_length = self.parse_byte(offset + 1).unwrap_or(0) as usize;
                        offset += pkg_length;
                    }
                    AmlOpcode::Method => {
                        // Parsear Method (PkgLength, Name, ByteFlags, TermList)
                        let pkg_length = self.parse_byte(offset + 1).unwrap_or(0) as usize;
                        offset += pkg_length;
                    }
                    _ => {
                        // Otros opcodes no implementados
                        offset += 1;
                    }
                }
            } else {
                offset += 1;
            }
        }

        Ok(())
    }

    /// Obtener un valor del namespace
    pub fn get_value(&self, name: &str) -> Option<&AmlValue> {
        self.namespace.get(name)
    }
}

impl Madt {
    pub fn new() -> Self {
        Self {
            local_apic_address: 0xFEE00000, // Dirección por defecto del Local APIC
            flags: 0,
            processors: Vec::new(),
            io_apics: Vec::new(),
        }
    }

    /// Parsear MADT desde una tabla ACPI
    pub unsafe fn from_acpi_table(table: &AcpiTable) -> Option<Self> {
        if table.signature != [b'A', b'P', b'I', b'C'] {
            return None;
        }

        let madt_ptr = table as *const AcpiTable as *const u8;
        
        // Leer local_apic_address y flags (offset 36 y 40)
        let local_apic_address = *((madt_ptr.add(36)) as *const u32);
        let flags = *((madt_ptr.add(40)) as *const u32);

        let mut madt = Madt {
            local_apic_address,
            flags,
            processors: Vec::new(),
            io_apics: Vec::new(),
        };

        // Parsear entradas MADT (comienzan en offset 44)
        let mut offset = 44;
        while offset < table.length as usize {
            let entry_type = *((madt_ptr.add(offset)) as *const u8);
            let entry_length = *((madt_ptr.add(offset + 1)) as *const u8);

            match entry_type {
                0 => {
                    // Processor Local APIC
                    let acpi_processor_id = *((madt_ptr.add(offset + 2)) as *const u8);
                    let apic_id = *((madt_ptr.add(offset + 3)) as *const u8);
                    let flags = *((madt_ptr.add(offset + 4)) as *const u32);
                    
                    madt.processors.push(ProcessorLocalApicEntry {
                        acpi_processor_id,
                        apic_id,
                        flags,
                    });
                }
                1 => {
                    // I/O APIC
                    let io_apic_id = *((madt_ptr.add(offset + 2)) as *const u8);
                    let reserved = *((madt_ptr.add(offset + 3)) as *const u8);
                    let io_apic_address = *((madt_ptr.add(offset + 4)) as *const u32);
                    let global_system_interrupt_base = *((madt_ptr.add(offset + 8)) as *const u32);
                    
                    madt.io_apics.push(IoApicEntry {
                        io_apic_id,
                        reserved,
                        io_apic_address,
                        global_system_interrupt_base,
                    });
                }
                _ => {
                    // Otros tipos de entradas no implementados aún
                }
            }

            offset += entry_length as usize;
        }

        Some(madt)
    }

    /// Obtener el número de procesadores
    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }

    /// Verificar si un procesador está habilitado
    pub fn is_processor_enabled(&self, index: usize) -> bool {
        if index < self.processors.len() {
            (self.processors[index].flags & 1) != 0
        } else {
            false
        }
    }
}

impl Fadt {
    pub fn new() -> Self {
        Self {
            pm1a_event_block: 0,
            pm1b_event_block: 0,
            pm1a_control_block: 0,
            pm1b_control_block: 0,
            pm2_control_block: 0,
            pm_timer_block: 0,
            gpe0_block: 0,
            gpe1_block: 0,
        }
    }
}

impl Default for Fadt {
    fn default() -> Self {
        Self::new()
    }
}

/// Dispositivo ACPI
#[derive(Debug, Clone)]
pub struct AcpiDevice {
    pub hid: String, // Hardware ID
    pub uid: String, // Unique ID
    pub status: u32,
    pub resources: Vec<AcpiResource>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

/// Recurso ACPI
#[derive(Debug, Clone)]
pub enum AcpiResource {
    Memory { address: u64, length: u64 },
    Io { port: u16, length: u16 },
    Irq { irq: u32 },
    Dma { channel: u8 },
}

/// Información de energía
#[derive(Debug, Clone)]
pub struct PowerInfo {
    pub battery_present: bool,
    pub battery_level: u8, // 0-100
    pub ac_connected: bool,
    pub power_state: PowerState,
}

impl PowerInfo {
    pub fn new() -> Self {
        Self {
            battery_present: false,
            battery_level: 100,
            ac_connected: true,
            power_state: PowerState::Working,
        }
    }
}

impl Default for PowerInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Información de temperatura
#[derive(Debug, Clone)]
pub struct ThermalInfo {
    pub cpu_temperature: f32, // En grados Celsius
    pub gpu_temperature: f32,
    pub fan_speed: u32, // RPM
}

impl ThermalInfo {
    pub fn new() -> Self {
        Self {
            cpu_temperature: 45.0,
            gpu_temperature: 40.0,
            fan_speed: 1000,
        }
    }
}

impl Default for ThermalInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestor ACPI
pub struct AcpiManager {
    pub state: AcpiState,
    pub rsdp: Option<Rsdp>,
    pub rsdt: Option<AcpiTable>,
    pub xsdt: Option<AcpiTable>,
    pub fadt: Option<Fadt>,
    pub madt: Option<Madt>,
    pub aml_interpreter: Option<AmlInterpreter>,
    pub c_state_manager: CStateManager,
    pub tables: BTreeMap<String, AcpiTable>,
    pub devices: Vec<AcpiDevice>,
    pub power_info: PowerInfo,
    pub thermal_info: ThermalInfo,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl AcpiManager {
    pub fn new() -> Self {
        Self {
            state: AcpiState::Uninitialized,
            rsdp: None,
            rsdt: None,
            xsdt: None,
            fadt: None,
            madt: None,
            aml_interpreter: None,
            c_state_manager: CStateManager::new(),
            tables: BTreeMap::new(),
            devices: Vec::new(),
            power_info: PowerInfo::new(),
            thermal_info: ThermalInfo::new(),
            graph_kernel: None,
        }
    }

    /// Buscar el RSDP en memoria
    unsafe fn search_rsdp() -> Option<Rsdp> {
        // Buscar en el área EBDA (Extended BIOS Data Area)
        // La dirección de EBDA está en 0x40E
        let ebda_ptr: *const u16 = 0x40E as *const u16;
        let ebda_address = (*ebda_ptr as u64) * 16;
        
        if ebda_address != 0 {
            // Buscar en los primeros 1KB del EBDA
            for offset in 0..1024 {
                let addr = (ebda_address + offset) as *const Rsdp;
                let rsdp = &*addr;
                if rsdp.is_valid() && rsdp.verify_checksum() {
                    return Some(rsdp.clone());
                }
            }
        }
        
        // Buscar en el área de memoria baja (0xE0000-0xFFFFF)
        for addr in (0xE0000..=0xFFFFF).step_by(16) {
            let rsdp_ptr = addr as *const Rsdp;
            let rsdp = &*rsdp_ptr;
            if rsdp.is_valid() && rsdp.verify_checksum() {
                return Some(rsdp.clone());
            }
        }
        
        None
    }

    /// Parsear RSDT desde la dirección especificada
    unsafe fn parse_rsdt(rsdt_address: u32) -> Option<AcpiTable> {
        let rsdt_ptr = rsdt_address as *const AcpiTable;
        let rsdt = &*rsdt_ptr;
        
        if rsdt.is_valid() && rsdt.verify_checksum() {
            Some(rsdt.clone())
        } else {
            None
        }
    }

    /// Parsear XSDT desde la dirección especificada
    unsafe fn parse_xsdt(xsdt_address: u64) -> Option<AcpiTable> {
        let xsdt_ptr = xsdt_address as *const AcpiTable;
        let xsdt = &*xsdt_ptr;
        
        if xsdt.is_valid() && xsdt.verify_checksum() {
            Some(xsdt.clone())
        } else {
            None
        }
    }

    /// Enumerar tablas ACPI desde RSDT/XSDT
    unsafe fn enumerate_tables(&mut self, use_xsdt: bool) {
        let table_ptr = if use_xsdt {
            if let Some(ref xsdt) = self.xsdt {
                xsdt as *const AcpiTable as u64 + 36 // Saltar el header de la tabla
            } else {
                return;
            }
        } else {
            if let Some(ref rsdt) = self.rsdt {
                rsdt as *const AcpiTable as u64 + 36 // Saltar el header de la tabla
            } else {
                return;
            }
        };

        let entry_size = if use_xsdt { 8 } else { 4 }; // XSDT usa 64-bit, RSDT usa 32-bit
        let table_count = if use_xsdt {
            if let Some(ref xsdt) = self.xsdt {
                (xsdt.length - 36) / 8
            } else {
                0
            }
        } else {
            if let Some(ref rsdt) = self.rsdt {
                (rsdt.length - 36) / 4
            } else {
                0
            }
        };

        for i in 0..table_count {
            let entry_offset = table_ptr + (i as u64 * entry_size);
            let table_address = if use_xsdt {
                *(entry_offset as *const u64)
            } else {
                *(entry_offset as *const u32) as u64
            };

            let table_ptr = table_address as *const AcpiTable;
            let table = &*table_ptr;

            if table.is_valid() && table.verify_checksum() {
                let signature = table.signature_str();
                self.tables.insert(signature, table.clone());
            }
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Inicializar ACPI
    pub fn initialize(&mut self) -> Result<(), String> {
        self.state = AcpiState::SearchingRsdp;

        // Buscar el RSDP en memoria
        let rsdp = unsafe { Self::search_rsdp() };
        let rsdp = rsdp.ok_or_else(|| String::from("RSDP not found"))?;
        
        // Verificar checksum del RSDP
        if !rsdp.verify_checksum() {
            return Err(String::from("Invalid RSDP checksum"));
        }
        
        // Verificar checksum extendido si ACPI 2.0+
        if rsdp.revision >= 2 && !rsdp.verify_extended_checksum() {
            return Err(String::from("Invalid RSDP extended checksum"));
        }
        
        self.rsdp = Some(rsdp);
        self.state = AcpiState::ParsingRsdt;

        // Determinar si usar XSDT (ACPI 2.0+) o RSDT (ACPI 1.0)
        let use_xsdt = self.rsdp.as_ref().map(|r| r.revision >= 2).unwrap_or(false);
        
        if use_xsdt {
            // Parsear XSDT
            let xsdt_address = self.rsdp.as_ref().map(|r| r.xsdt_address).unwrap_or(0);
            if xsdt_address == 0 {
                return Err(String::from("XSDT address is zero"));
            }
            
            let xsdt = unsafe { Self::parse_xsdt(xsdt_address) };
            let xsdt = xsdt.ok_or_else(|| String::from("Failed to parse XSDT"))?;
            self.xsdt = Some(xsdt);
        } else {
            // Parsear RSDT
            let rsdt_address = self.rsdp.as_ref().map(|r| r.rsdt_address).unwrap_or(0);
            if rsdt_address == 0 {
                return Err(String::from("RSDT address is zero"));
            }
            
            let rsdt = unsafe { Self::parse_rsdt(rsdt_address) };
            let rsdt = rsdt.ok_or_else(|| String::from("Failed to parse RSDT"))?;
            self.rsdt = Some(rsdt);
        }

        self.state = AcpiState::ParsingFadt;

        // Enumerar todas las tablas ACPI
        unsafe { self.enumerate_tables(use_xsdt) };

        // Buscar FADT en las tablas
        if let Some(fadt) = self.tables.get("FACP") {
            // Parsear FADT (simplificado)
            self.fadt = Some(Fadt::new());
        } else {
            return Err(String::from("FADT not found in ACPI tables"));
        }

        // Buscar y parsear MADT para soporte SMP
        if let Some(madt_table) = self.tables.get("APIC") {
            let madt = unsafe { Madt::from_acpi_table(madt_table) };
            if let Some(madt) = madt {
                self.madt = Some(madt);
            }
        }

        // Buscar y parsear DSDT para intérprete AML
        if let Some(dsdt_table) = self.tables.get("DSDT") {
            let mut aml_interpreter = AmlInterpreter::new();
            aml_interpreter.set_dsdt(dsdt_table.clone());
            
            // Ejecutar el intérprete AML
            let _ = unsafe { aml_interpreter.execute() };
            
            self.aml_interpreter = Some(aml_interpreter);
        }

        self.state = AcpiState::Ready;
        Ok(())
    }

    /// Agregar un dispositivo ACPI
    pub fn add_device(&mut self, hid: String, uid: String) -> Result<(), String> {
        let mut device = AcpiDevice {
            hid,
            uid,
            status: 0,
            resources: Vec::new(),
            graph_node_id: None,
        };

        // Registrar el dispositivo como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::HardwareDevice(crate::graph_kernel::HardwareType::Acpi);
            let node_name = format!("acpi_device_{}", device.hid);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            device.graph_node_id = node_id;
        }

        self.devices.push(device);
        Ok(())
    }

    /// Obtener información de energía
    pub fn get_power_info(&self) -> &PowerInfo {
        &self.power_info
    }

    /// Actualizar información de energía
    pub fn update_power_info(&mut self) {
        // En un sistema real, aquí se leería la información de la batería
        // desde los registros ACPI o desde el controlador de batería
    }

    /// Obtener información térmica
    pub fn get_thermal_info(&self) -> &ThermalInfo {
        &self.thermal_info
    }

    /// Actualizar información térmica
    pub fn update_thermal_info(&mut self) {
        // En un sistema real, aquí se leería la temperatura de los sensores
        // térmicos a través de ACPI
    }

    /// Cambiar estado de energía
    pub fn set_power_state(&mut self, state: PowerState) -> Result<(), String> {
        match state {
            PowerState::Sleep => {
                // En un sistema real, aquí se:
                // 1. Prepararía el sistema para dormir
                // 2. Guardaría el estado de los dispositivos
                // 3. Enviaría el comando de suspensión al hardware
            }
            PowerState::Hibernate => {
                // En un sistema real, aquí se:
                // 1. Guardaría el contenido de RAM en disco
                // 2. Apagaría el sistema
            }
            PowerState::SoftOff => {
                // En un sistema real, aquí se:
                // 1. Apagaría el sistema suavemente
                // 2. Mantendría alimentación para wake-on-LAN
            }
            _ => {}
        }

        self.power_info.power_state = state;
        Ok(())
    }

    /// Obtener una tabla ACPI por firma
    pub fn get_table(&self, signature: &str) -> Option<&AcpiTable> {
        self.tables.get(signature)
    }

    /// Listar todas las tablas ACPI
    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Listar todos los dispositivos ACPI
    pub fn list_devices(&self) -> &[AcpiDevice] {
        &self.devices
    }

    /// Habilitar evento de energía
    pub fn enable_power_event(&mut self, event: u32) -> Result<(), String> {
        // En un sistema real, aquí se habilitaría un evento de energía
        // en los registros PM1x
        Ok(())
    }

    /// Deshabilitar evento de energía
    pub fn disable_power_event(&mut self, event: u32) -> Result<(), String> {
        // En un sistema real, aquí se deshabilitaría un evento de energía
        // en los registros PM1x
        Ok(())
    }

    /// Obtener estado ACPI
    pub fn state(&self) -> &AcpiState {
        &self.state
    }

    /// Inicializar C-states desde la información ACPI
    pub fn init_c_states(&mut self) {
        // C1 siempre está disponible (instrucción HLT)
        self.c_state_manager.add_c_state(CState::C1, 1, 1000);

        // En un sistema real, aquí se leería la información de C-states
        // desde las tablas ACPI (FADT, SSDT) para determinar qué C-states
        // están disponibles y sus características

        // Por ahora, agregamos C-states comunes como ejemplo
        self.c_state_manager.add_c_state(CState::C2, 10, 500);
        self.c_state_manager.add_c_state(CState::C3, 50, 200);
    }

    /// Entrar en un C-state específico
    pub fn enter_c_state(&mut self, c_state: CState) -> Result<(), String> {
        self.c_state_manager.enter_c_state(c_state)
    }

    /// Obtener el C-state actual
    pub fn current_c_state(&self) -> CState {
        self.c_state_manager.current_c_state()
    }

    /// Calcular el C-state óptimo basado en la latencia permitida
    pub fn calculate_optimal_c_state(&self, max_latency: u32) -> Option<CState> {
        self.c_state_manager.calculate_optimal_c_state(max_latency)
    }

    /// Enumerar dispositivos desde el namespace ACPI
    pub fn enumerate_acpi_devices(&mut self) -> Result<(), String> {
        let mut devices_to_add: Vec<(String, String)> = Vec::new();
        
        if let Some(ref aml_interpreter) = self.aml_interpreter {
            // Recorrer el namespace ACPI buscando dispositivos
            for (name, value) in aml_interpreter.namespace.iter() {
                // Buscar nombres que corresponden a dispositivos
                // Los dispositivos ACPI típicamente tienen el formato "_HID"
                if name.contains("_HID") {
                    if let AmlValue::String(hid) = value {
                        // Crear un dispositivo ACPI
                        let uid = name.replace("_HID", "_UID");
                        devices_to_add.push((hid.clone(), uid));
                    }
                }
            }
        }
        
        // Agregar dispositivos después de terminar el borrow
        for (hid, uid) in devices_to_add {
            let _ = self.add_device(hid, uid);
        }
        
        Ok(())
    }

    /// Buscar dispositivos por Hardware ID (HID)
    pub fn find_devices_by_hid(&self, hid: &str) -> Vec<&AcpiDevice> {
        self.devices.iter()
            .filter(|device| device.hid == hid)
            .collect()
    }

    /// Obtener todos los dispositivos ACPI
    pub fn get_all_acpi_devices(&self) -> &[AcpiDevice] {
        &self.devices
    }
}

impl Default for AcpiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de ACPI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpiError {
    RsdpNotFound,
    InvalidRsdp,
    RsdtNotFound,
    InvalidRsdt,
    FadtNotFound,
    InvalidFadt,
    TableNotFound,
    DeviceNotFound,
    PowerStateChangeFailed,
}

impl fmt::Display for AcpiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AcpiError::RsdpNotFound => write!(f, "RSDP not found"),
            AcpiError::InvalidRsdp => write!(f, "Invalid RSDP"),
            AcpiError::RsdtNotFound => write!(f, "RSDT not found"),
            AcpiError::InvalidRsdt => write!(f, "Invalid RSDT"),
            AcpiError::FadtNotFound => write!(f, "FADT not found"),
            AcpiError::InvalidFadt => write!(f, "Invalid FADT"),
            AcpiError::TableNotFound => write!(f, "Table not found"),
            AcpiError::DeviceNotFound => write!(f, "Device not found"),
            AcpiError::PowerStateChangeFailed => write!(f, "Power state change failed"),
        }
    }
}
