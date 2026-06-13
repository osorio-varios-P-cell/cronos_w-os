//! Signals Module
//! 
//! This module implements Unix-like signals for process communication and control.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Tipos de señales POSIX
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// SIGHUP - Hangup detected on controlling terminal
    SIGHUP = 1,
    /// SIGINT - Interrupt from keyboard
    SIGINT = 2,
    /// SIGQUIT - Quit from keyboard
    SIGQUIT = 3,
    /// SIGILL - Illegal instruction
    SIGILL = 4,
    /// SIGTRAP - Trace/breakpoint trap
    SIGTRAP = 5,
    /// SIGABRT - Abort signal from abort(3)
    SIGABRT = 6,
    /// SIGBUS - Bus error (bad memory access)
    SIGBUS = 7,
    /// SIGFPE - Floating point exception
    SIGFPE = 8,
    /// SIGKILL - Kill signal
    SIGKILL = 9,
    /// SIGUSR1 - User-defined signal 1
    SIGUSR1 = 10,
    /// SIGSEGV - Invalid memory reference
    SIGSEGV = 11,
    /// SIGUSR2 - User-defined signal 2
    SIGUSR2 = 12,
    /// SIGPIPE - Broken pipe
    SIGPIPE = 13,
    /// SIGALRM - Timer signal from alarm(2)
    SIGALRM = 14,
    /// SIGTERM - Termination signal
    SIGTERM = 15,
    /// SIGCHLD - Child stopped or terminated
    SIGCHLD = 17,
    /// SIGCONT - Continue if stopped
    SIGCONT = 18,
    /// SIGSTOP - Stop process
    SIGSTOP = 19,
    /// SIGTSTP - Stop typed at terminal
    SIGTSTP = 20,
    /// SIGTTIN - Background read from tty
    SIGTTIN = 21,
    /// SIGTTOU - Background write to tty
    SIGTTOU = 22,
}

impl Signal {
    /// Crear señal desde número
    pub fn from_number(num: u8) -> Option<Self> {
        match num {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            5 => Some(Signal::SIGTRAP),
            6 => Some(Signal::SIGABRT),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            20 => Some(Signal::SIGTSTP),
            21 => Some(Signal::SIGTTIN),
            22 => Some(Signal::SIGTTOU),
            _ => None,
        }
    }

    /// Obtener número de señal
    pub fn number(self) -> u8 {
        self as u8
    }

    /// Obtener nombre de señal
    pub fn name(self) -> &'static str {
        match self {
            Signal::SIGHUP => "SIGHUP",
            Signal::SIGINT => "SIGINT",
            Signal::SIGQUIT => "SIGQUIT",
            Signal::SIGILL => "SIGILL",
            Signal::SIGTRAP => "SIGTRAP",
            Signal::SIGABRT => "SIGABRT",
            Signal::SIGBUS => "SIGBUS",
            Signal::SIGFPE => "SIGFPE",
            Signal::SIGKILL => "SIGKILL",
            Signal::SIGUSR1 => "SIGUSR1",
            Signal::SIGSEGV => "SIGSEGV",
            Signal::SIGUSR2 => "SIGUSR2",
            Signal::SIGPIPE => "SIGPIPE",
            Signal::SIGALRM => "SIGALRM",
            Signal::SIGTERM => "SIGTERM",
            Signal::SIGCHLD => "SIGCHLD",
            Signal::SIGCONT => "SIGCONT",
            Signal::SIGSTOP => "SIGSTOP",
            Signal::SIGTSTP => "SIGTSTP",
            Signal::SIGTTIN => "SIGTTIN",
            Signal::SIGTTOU => "SIGTTOU",
        }
    }

    /// Verificar si la señal puede ser ignorada
    pub fn can_ignore(self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// Verificar si la señal puede ser capturada
    pub fn can_catch(self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }
}

/// Acción de señal
#[derive(Debug, Clone, Copy)]
pub enum SignalAction {
    /// Acción por defecto
    Default,
    /// Ignorar señal
    Ignore,
    /// Handler personalizado (dirección del handler)
    Handler(u64),
}

impl SignalAction {
    /// Crear acción por defecto
    pub fn default() -> Self {
        Self::Default
    }

    /// Crear acción de ignorar
    pub fn ignore() -> Self {
        Self::Ignore
    }

    /// Crear acción con handler
    pub fn handler(addr: u64) -> Self {
        Self::Handler(addr)
    }
}

impl Default for SignalAction {
    fn default() -> Self {
        Self::Default
    }
}

/// Máscara de señales
#[derive(Debug, Clone)]
pub struct SignalMask {
    /// Bits de señales bloqueadas
    pub blocked: u64,
}

impl SignalMask {
    /// Crear nueva máscara vacía
    pub fn new() -> Self {
        Self { blocked: 0 }
    }

    /// Bloquear señal
    pub fn block(&mut self, signal: Signal) {
        let bit = 1u64 << signal.number();
        self.blocked |= bit;
    }

    /// Desbloquear señal
    pub fn unblock(&mut self, signal: Signal) {
        let bit = 1u64 << signal.number();
        self.blocked &= !bit;
    }

    /// Verificar si señal está bloqueada
    pub fn is_blocked(&self, signal: Signal) -> bool {
        let bit = 1u64 << signal.number();
        (self.blocked & bit) != 0
    }

    /// Limpiar todas las señales
    pub fn clear(&mut self) {
        self.blocked = 0;
    }

    /// Establecer todas las señales
    pub fn set_all(&mut self) {
        self.blocked = 0xFFFFFFFFFFFFFFFF;
    }
}

impl Default for SignalMask {
    fn default() -> Self {
        Self::new()
    }
}

/// Información de señal pendiente
#[derive(Debug, Clone)]
pub struct PendingSignal {
    /// Tipo de señal
    pub signal: Signal,
    /// PID del proceso que envió la señal
    pub sender_pid: u32,
    /// Información adicional
    pub info: u64,
}

/// Gestor de señales
pub struct SignalManager {
    /// Acciones de señal para cada tipo
    pub actions: [SignalAction; 32],
    /// Máscara de señales actual
    pub mask: SignalMask,
    /// Señales pendientes
    pub pending: Vec<PendingSignal>,
}

impl SignalManager {
    /// Crear nuevo gestor de señales
    pub fn new() -> Self {
        Self {
            actions: [SignalAction::default(); 32],
            mask: SignalMask::default(),
            pending: Vec::new(),
        }
    }

    /// Establecer acción para una señal
    pub fn set_action(&mut self, signal: Signal, action: SignalAction) -> Result<(), String> {
        if !signal.can_catch() && matches!(action, SignalAction::Handler(_)) {
            return Err(format!("Signal {} cannot be caught", signal.name()));
        }
        
        if !signal.can_ignore() && matches!(action, SignalAction::Ignore) {
            return Err(format!("Signal {} cannot be ignored", signal.name()));
        }
        
        let index = signal.number() as usize;
        if index < self.actions.len() {
            self.actions[index] = action;
            Ok(())
        } else {
            Err(String::from("Invalid signal number"))
        }
    }

    /// Obtener acción para una señal
    pub fn get_action(&self, signal: Signal) -> SignalAction {
        let index = signal.number() as usize;
        if index < self.actions.len() {
            self.actions[index]
        } else {
            SignalAction::default()
        }
    }

    /// Enviar señal
    pub fn send(&mut self, signal: Signal, sender_pid: u32, info: u64) -> Result<(), String> {
        // Verificar si la señal está bloqueada
        if self.mask.is_blocked(signal) {
            // Agregar a pendientes
            self.pending.push(PendingSignal {
                signal,
                sender_pid,
                info,
            });
            return Ok(());
        }

        // Procesar la señal según su acción
        let action = self.get_action(signal);
        match action {
            SignalAction::Ignore => {
                // Ignorar señal
                Ok(())
            }
            SignalAction::Default => {
                // Acción por defecto
                self.default_action(signal)
            }
            SignalAction::Handler(_addr) => {
                // Llamar al handler (en un sistema real)
                Ok(())
            }
        }
    }

    /// Acción por defecto para una señal
    fn default_action(&self, signal: Signal) -> Result<(), String> {
        match signal {
            Signal::SIGTERM | Signal::SIGHUP | Signal::SIGINT => {
                // Terminar proceso
                Ok(())
            }
            Signal::SIGKILL => {
                // Terminar proceso inmediatamente
                Ok(())
            }
            Signal::SIGSTOP => {
                // Detener proceso
                Ok(())
            }
            Signal::SIGCONT => {
                // Continuar proceso
                Ok(())
            }
            Signal::SIGCHLD => {
                // Ignorar por defecto
                Ok(())
            }
            Signal::SIGSEGV | Signal::SIGBUS | Signal::SIGILL | Signal::SIGFPE => {
                // Terminar proceso con core dump
                Ok(())
            }
            _ => {
                // Ignorar otras señales por defecto
                Ok(())
            }
        }
    }

    /// Procesar señales pendientes
    pub fn process_pending(&mut self) -> Result<(), String> {
        let mut i = 0;
        while i < self.pending.len() {
            let pending = self.pending[i].clone();
            
            if !self.mask.is_blocked(pending.signal) {
                let action = self.get_action(pending.signal);
                match action {
                    SignalAction::Ignore => {
                        self.pending.remove(i);
                        continue;
                    }
                    SignalAction::Default => {
                        self.default_action(pending.signal)?;
                        self.pending.remove(i);
                        continue;
                    }
                    SignalAction::Handler(_addr) => {
                        // Llamar al handler (en un sistema real)
                        self.pending.remove(i);
                        continue;
                    }
                }
            }
            
            i += 1;
        }
        
        Ok(())
    }

    /// Establecer máscara de señales
    pub fn set_mask(&mut self, new_mask: SignalMask) {
        self.mask = new_mask;
    }

    /// Obtener máscara de señales
    pub fn get_mask(&self) -> SignalMask {
        self.mask.clone()
    }

    /// Obtener número de señales pendientes
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Verificar si hay señales pendientes
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Signal Manager Status\n");
        report.push_str("=====================\n\n");
        
        report.push_str(&format!("Pending Signals: {}\n", self.pending_count()));
        if self.has_pending() {
            report.push_str("Pending: ");
            for (i, pending) in self.pending.iter().enumerate() {
                if i > 0 {
                    report.push_str(", ");
                }
                report.push_str(pending.signal.name());
            }
            report.push_str("\n");
        }
        
        report.push_str("\nSignal Actions:\n");
        for i in 1..=22 {
            if let Some(signal) = Signal::from_number(i) {
                let action_str = match self.get_action(signal) {
                    SignalAction::Default => "Default",
                    SignalAction::Ignore => "Ignore",
                    SignalAction::Handler(addr) => &format!("Handler(0x{:X})", addr),
                };
                report.push_str(&format!("  {}: {}\n", signal.name(), action_str));
            }
        }
        
        report
    }
}

impl Default for SignalManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades para señales
pub struct SignalUtils;

impl SignalUtils {
    /// Verificar si un número de señal es válido
    pub fn is_valid_signal(num: u8) -> bool {
        Signal::from_number(num).is_some()
    }

    /// Obtener todas las señales estándar
    pub fn get_standard_signals() -> Vec<Signal> {
        vec![
            Signal::SIGHUP,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGILL,
            Signal::SIGTRAP,
            Signal::SIGABRT,
            Signal::SIGBUS,
            Signal::SIGFPE,
            Signal::SIGKILL,
            Signal::SIGUSR1,
            Signal::SIGSEGV,
            Signal::SIGUSR2,
            Signal::SIGPIPE,
            Signal::SIGALRM,
            Signal::SIGTERM,
            Signal::SIGCHLD,
            Signal::SIGCONT,
            Signal::SIGSTOP,
            Signal::SIGTSTP,
            Signal::SIGTTIN,
            Signal::SIGTTOU,
        ]
    }
}
