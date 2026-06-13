//! GDB Stub Module
//! 
//! This module implements a GDB stub for remote debugging of the kernel,
//! implementing the GDB remote serial protocol.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::BTreeMap;

/// Estado del GDB stub
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdbStubState {
    /// No inicializado
    Uninitialized,
    /// Esperando conexión
    WaitingForConnection,
    /// Conectado
    Connected,
    /// Ejecutando
    Running,
    /// Pausado en breakpoint
    Paused,
    /// Desconectado
    Disconnected,
}

/// Comando GDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdbCommand {
    /// No command
    None,
    /// Get last signal
    QuestionMark,
    /// Read registers
    G,
    /// Write registers
    P,
    /// Read memory
    M,
    /// Write memory
    X,
    /// Continue
    C,
    /// Step
    S,
    /// Set breakpoint
    Z,
    /// Remove breakpoint
    z,
    /// Kill
    K,
    /// Query
    q,
    /// Unknown command
    Unknown,
}

/// Breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Dirección del breakpoint
    pub address: u64,
    /// Tipo (software, hardware)
    pub breakpoint_type: BreakpointType,
    /// Si está habilitado
    pub enabled: bool,
}

/// Tipo de breakpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
    /// Software breakpoint
    Software,
    /// Hardware breakpoint
    Hardware,
}

/// Gestor de GDB stub
pub struct GdbStub {
    /// Estado actual
    state: GdbStubState,
    /// Breakpoints activos
    breakpoints: Vec<Breakpoint>,
    /// Puerto serial
    serial_port: u16,
    /// Buffer de entrada
    input_buffer: Vec<u8>,
    /// Buffer de salida
    output_buffer: Vec<u8>,
    /// Última señal
    last_signal: u8,
}

impl GdbStub {
    /// Crear nuevo GDB stub
    pub fn new(serial_port: u16) -> Self {
        Self {
            state: GdbStubState::Uninitialized,
            breakpoints: Vec::new(),
            serial_port,
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
            last_signal: 0,
        }
    }

    /// Inicializar el GDB stub
    pub fn initialize(&mut self) {
        self.state = GdbStubState::WaitingForConnection;
        // En un sistema real, esto inicializaría el puerto serial
    }

    /// Esperar conexión de GDB
    pub fn wait_for_connection(&mut self) -> bool {
        // En un sistema real, esto esperaría a que GDB se conecte
        self.state = GdbStubState::Connected;
        true
    }

    /// Procesar comando de GDB
    pub fn process_command(&mut self, command: &[u8]) -> Vec<u8> {
        let cmd = self.parse_command(command);
        let response = self.handle_command(cmd);
        response
    }

    /// Parsear comando
    fn parse_command(&self, command: &[u8]) -> GdbCommand {
        if command.is_empty() {
            return GdbCommand::None;
        }

        match command[0] {
            b'?' => GdbCommand::QuestionMark,
            b'g' => GdbCommand::G,
            b'p' => GdbCommand::P,
            b'm' => GdbCommand::M,
            b'x' => GdbCommand::X,
            b'c' => GdbCommand::C,
            b's' => GdbCommand::S,
            b'Z' => GdbCommand::Z,
            b'z' => GdbCommand::z,
            b'k' => GdbCommand::K,
            b'q' => GdbCommand::q,
            _ => GdbCommand::Unknown,
        }
    }

    /// Manejar comando
    fn handle_command(&mut self, command: GdbCommand) -> Vec<u8> {
        match command {
            GdbCommand::QuestionMark => self.handle_signal_query(),
            GdbCommand::G => self.handle_read_registers(),
            GdbCommand::P => self.handle_write_registers(),
            GdbCommand::M => self.handle_read_memory(),
            GdbCommand::X => self.handle_write_memory(),
            GdbCommand::C => self.handle_continue(),
            GdbCommand::S => self.handle_step(),
            GdbCommand::Z => self.handle_set_breakpoint(),
            GdbCommand::z => self.handle_remove_breakpoint(),
            GdbCommand::K => self.handle_kill(),
            GdbCommand::q => self.handle_query(),
            GdbCommand::None => vec![b'+'],
            GdbCommand::Unknown => vec![b'+'],
        }
    }

    /// Manejar query de señal
    fn handle_signal_query(&mut self) -> Vec<u8> {
        // Retornar la última señal que causó el stop
        let mut response = String::from("S");
        response.push_str(&format!("{:02x}", self.last_signal));
        self.encode_packet(response.as_bytes())
    }

    /// Manejar lectura de registros
    fn handle_read_registers(&self) -> Vec<u8> {
        // En un sistema real, esto leería los registros del CPU
        // Para este ejemplo, retornamos datos simulados
        let registers = "0000000000000000000000000000000000000000000000000000000000000000";
        self.encode_packet(registers.as_bytes())
    }

    /// Manejar escritura de registros
    fn handle_write_registers(&self) -> Vec<u8> {
        // En un sistema real, esto escribiría los registros del CPU
        self.encode_packet(b"OK")
    }

    /// Manejar lectura de memoria
    fn handle_read_memory(&self) -> Vec<u8> {
        // En un sistema real, esto leería memoria
        self.encode_packet(b"OK")
    }

    /// Manejar escritura de memoria
    fn handle_write_memory(&self) -> Vec<u8> {
        // En un sistema real, esto escribiría memoria
        self.encode_packet(b"OK")
    }

    /// Manejar continue
    fn handle_continue(&mut self) -> Vec<u8> {
        self.state = GdbStubState::Running;
        self.encode_packet(b"OK")
    }

    /// Manejar step
    fn handle_step(&mut self) -> Vec<u8> {
        self.state = GdbStubState::Running;
        self.encode_packet(b"OK")
    }

    /// Manejar set breakpoint
    fn handle_set_breakpoint(&mut self) -> Vec<u8> {
        // En un sistema real, esto establecería un breakpoint
        self.encode_packet(b"OK")
    }

    /// Manejar remove breakpoint
    fn handle_remove_breakpoint(&mut self) -> Vec<u8> {
        // En un sistema real, esto removería un breakpoint
        self.encode_packet(b"OK")
    }

    /// Manejar kill
    fn handle_kill(&mut self) -> Vec<u8> {
        self.state = GdbStubState::Disconnected;
        self.encode_packet(b"OK")
    }

    /// Manejar query
    fn handle_query(&self) -> Vec<u8> {
        // En un sistema real, esto manejaría queries específicos
        self.encode_packet(b"OK")
    }

    /// Codificar paquete GDB
    fn encode_packet(&self, data: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push(b'$');
        packet.extend_from_slice(data);
        
        // Calcular checksum
        let checksum = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        packet.push(b'#');
        packet.push(hex_nibble(checksum >> 4));
        packet.push(hex_nibble(checksum & 0xF));
        
        packet
    }

    /// Decodificar paquete GDB
    fn decode_packet(&self, packet: &[u8]) -> Option<Vec<u8>> {
        if packet.is_empty() || packet[0] != b'$' {
            return None;
        }
        
        // Encontrar el final del paquete
        let end = packet.iter().position(|&b| b == b'#')?;
        if end + 3 > packet.len() {
            return None;
        }
        
        // Verificar checksum
        let data = &packet[1..end];
        let checksum_high = hex_to_nibble(packet[end + 1])?;
        let checksum_low = hex_to_nibble(packet[end + 2])?;
        let checksum = (checksum_high << 4) | checksum_low;
        
        let calculated_checksum = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        
        if checksum != calculated_checksum {
            return None;
        }
        
        Some(data.to_vec())
    }

    /// Verificar si hay breakpoint en dirección
    pub fn has_breakpoint_at(&self, address: u64) -> bool {
        self.breakpoints.iter()
            .any(|bp| bp.address == address && bp.enabled)
    }

    /// Agregar breakpoint
    pub fn add_breakpoint(&mut self, address: u64, breakpoint_type: BreakpointType) {
        self.breakpoints.push(Breakpoint {
            address,
            breakpoint_type,
            enabled: true,
        });
    }

    /// Remover breakpoint
    pub fn remove_breakpoint(&mut self, address: u64) {
        self.breakpoints.retain(|bp| bp.address != address);
    }

    /// Obtener estado
    pub fn get_state(&self) -> GdbStubState {
        self.state
    }

    /// Establecer señal
    pub fn set_signal(&mut self, signal: u8) {
        self.last_signal = signal;
    }

    /// Trigger breakpoint
    pub fn trigger_breakpoint(&mut self, address: u64) {
        if self.has_breakpoint_at(address) {
            self.state = GdbStubState::Paused;
            self.last_signal = 5; // SIGTRAP
        }
    }

    /// Poll para comandos entrantes
    pub fn poll(&mut self) -> Option<Vec<u8>> {
        // En un sistema real, esto leería del puerto serial
        None
    }

    /// Enviar respuesta
    pub fn send_response(&mut self, response: &[u8]) {
        // En un sistema real, esto enviaría al puerto serial
        self.output_buffer.extend_from_slice(response);
    }
}

impl Default for GdbStub {
    fn default() -> Self {
        Self::new(1234)
    }
}

/// Convertir nibble a hex
fn hex_nibble(nibble: u8) -> u8 {
    if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + (nibble - 10)
    }
}

/// Convertir hex a nibble
fn hex_to_nibble(hex: u8) -> Option<u8> {
    match hex {
        b'0'..=b'9' => Some(hex - b'0'),
        b'a'..=b'f' => Some(hex - b'a' + 10),
        b'A'..=b'F' => Some(hex - b'A' + 10),
        _ => None,
    }
}

/// Utilidades para GDB stub
pub struct GdbStubUtils;

impl GdbStubUtils {
    /// Verificar si el stub está habilitado
    pub fn is_stub_enabled() -> bool {
        // En un sistema real, esto verificaría una configuración
        true
    }

    /// Obtener puerto serial configurado
    pub fn get_serial_port() -> u16 {
        // En un sistema real, esto leería una configuración
        1234
    }

    /// Habilitar stub en boot
    pub fn enable_on_boot() {
        // En un sistema real, esto configuraría el boot para habilitar el stub
    }

    /// Deshabilitar stub
    pub fn disable_stub() {
        // En un sistema real, esto deshabilitaría el stub
    }

    /// Crear breakpoint en dirección específica
    pub fn create_breakpoint(address: u64) -> Breakpoint {
        Breakpoint {
            address,
            breakpoint_type: BreakpointType::Software,
            enabled: true,
        }
    }

    /// Verificar si el sistema está en modo debug
    pub fn is_debug_mode() -> bool {
        // En un sistema real, esto verificaría flags de compilación
        true
    }
}

/// Gestor de breakpoints
pub struct BreakpointManager {
    /// Breakpoints activos
    breakpoints: Vec<Breakpoint>,
    /// Contador de hits
    hit_counts: alloc::collections::BTreeMap<u64, usize>,
}

impl BreakpointManager {
    /// Crear nuevo gestor
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            hit_counts: alloc::collections::BTreeMap::new(),
        }
    }

    /// Agregar breakpoint
    pub fn add(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.push(breakpoint);
    }

    /// Remover breakpoint
    pub fn remove(&mut self, address: u64) {
        self.breakpoints.retain(|bp| bp.address != address);
    }

    /// Verificar si hay breakpoint
    pub fn has_breakpoint(&self, address: u64) -> bool {
        self.breakpoints.iter()
            .any(|bp| bp.address == address && bp.enabled)
    }

    /// Trigger breakpoint
    pub fn trigger(&mut self, address: u64) -> bool {
        if self.has_breakpoint(address) {
            *self.hit_counts.entry(address).or_insert(0) += 1;
            true
        } else {
            false
        }
    }

    /// Obtener count de hits
    pub fn get_hit_count(&self, address: u64) -> usize {
        *self.hit_counts.get(&address).unwrap_or(&0)
    }

    /// Listar todos los breakpoints
    pub fn list_breakpoints(&self) -> &Vec<Breakpoint> {
        &self.breakpoints
    }

    /// Limpiar todos los breakpoints
    pub fn clear(&mut self) {
        self.breakpoints.clear();
        self.hit_counts.clear();
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}
