//! Serial writer para debug en entorno no_std
//!
//! Implementa escritura por UART 16550 (COM1) para output de debug,
//! reemplazando el uso de `println!` del std que no existe en bare-metal.
//!
//! Todos los módulos del kernel deben usar `serial_println!` / `kprintln!`
//! en lugar de `println!`.

use core::fmt;
use spin::Mutex;
use uart_16550::SerialPort;

/// Puerto serie COM1 (I/O base 0x3F8)
pub static SERIAL1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });

/// Inicializa el puerto serial. Debe llamarse una sola vez en el boot.
pub fn init_serial() {
    SERIAL1.lock().init();
}

/// Escribe un carácter al puerto serie.
pub fn serial_putchar(c: char) {
    let mut port = SERIAL1.lock();
    port.send(c as u8);
}

/// Implementación interna de format para escritura serial.
pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut port = SERIAL1.lock();
        for byte in s.bytes() {
            port.send(byte);
        }
        Ok(())
    }
}

/// Escribe al puerto serie sin newline.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!($crate::serial_writer::SerialWriter, $($arg)*);
        }
    };
}

/// Escribe al puerto serie con newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}

/// Alias principal del kernel: `kprintln!` → serial en debug, silencioso en release.
#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::serial_println!($($arg)*);
    };
}

/// Alias para errores: siempre imprime aunque sea release.
#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        $crate::serial_println!("[ERROR] {}", format_args!($($arg)*));
    };
}

/// Imprime un panic formateado al serial antes de hacer loop.
pub fn serial_panic(info: &core::panic::PanicInfo) {
    use core::fmt::Write;
    let mut writer = SerialWriter;
    let _ = writeln!(writer, "\n\n─────────────────────────────────────────");
    let _ = writeln!(writer, "  CRONOS W-OS KERNEL PANIC");
    let _ = writeln!(writer, "─────────────────────────────────────────");
    if let Some(location) = info.location() {
        let _ = writeln!(writer, "  Archivo: {}:{}", location.file(), location.line());
    }
    let _ = writeln!(writer, "  Mensaje: {}", info.message());
    let _ = writeln!(writer, "─────────────────────────────────────────");
    let _ = writeln!(writer, "  Sistema detenido. Reinicia el equipo.");
    let _ = writeln!(writer, "─────────────────────────────────────────\n");
}
