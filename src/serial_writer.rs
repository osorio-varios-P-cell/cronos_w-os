// Macros que usan acceso crudo al puerto UART 16550
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        $crate::serial_writer::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

use crate::spinlock::SpinMutex;
use x86_64::instructions::port::Port;

// Implementación de SerialWriter que usa acceso crudo al puerto
pub struct SerialWriter;

impl SerialWriter {
    // Definición explícita de la dirección del Registro de Estado de Línea (LSR)
    const LSR_OFFSET: u16 = 5;
    const DATA_PORT: u16 = 0x3F8;

    fn wait_for_empty_transmitter() {
        // Uso de turbofish para especificar el tipo u8
        let mut lsr = Port::<u8>::new(Self::DATA_PORT + Self::LSR_OFFSET);
        
        // Bit 5 (0x20): Transmit Holding Register Empty
        while (unsafe { lsr.read() } & 0x20) == 0 {
            core::hint::spin_loop();
        }
    }

    pub fn write_byte(byte: u8) {
        use x86_64::instructions::port::Port;
        let mut lsr = Port::<u8>::new(Self::DATA_PORT + Self::LSR_OFFSET);
        
        // Polling constante y forzoso antes de CADA byte
        while (unsafe { lsr.read() } & 0x20) == 0 {
            core::hint::spin_loop();
        }
        
        let mut data = Port::<u8>::new(Self::DATA_PORT);
        unsafe { data.write(byte); }
    }
}

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            Self::write_byte(byte);
        }
        Ok(())
    }
}

// Variable global protegida con SpinMutex (usa AtomicBool con compare_exchange)
pub static SERIAL_WRITER: SpinMutex<SerialWriter> = SpinMutex::new(SerialWriter);

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL_WRITER.lock().write_fmt(args).unwrap();
}

// Stubs para funciones exportadas (mantenidos por compatibilidad)
pub fn serial_print(s: &str) {
    let _serial = SERIAL_WRITER.lock();
    for byte in s.bytes() {
        SerialWriter::write_byte(byte);
    }
}

pub fn serial_println(s: &str) {
    let _serial = SERIAL_WRITER.lock();
    for byte in s.bytes() {
        SerialWriter::write_byte(byte);
    }
    SerialWriter::write_byte(b'\r');
    SerialWriter::write_byte(b'\n');
}

pub fn serial_print_hex(val: u64) {
    let _serial = SERIAL_WRITER.lock();
    let hex_chars = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = (val >> (i * 4)) & 0xf;
        SerialWriter::write_byte(hex_chars[nibble as usize]);
    }
}

pub fn serial_print_dec(val: u64) {
    let _serial = SERIAL_WRITER.lock();
    if val == 0 {
        SerialWriter::write_byte(b'0');
        return;
    }
    
    let mut buffer = [0u8; 20];
    let mut i = 0;
    let mut n = val;
    
    while n > 0 {
        buffer[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }
    
    for j in (0..i).rev() {
        SerialWriter::write_byte(buffer[j]);
    }
}

// Stub para serial_panic (requerido por el import)
pub fn serial_panic(_info: &core::panic::PanicInfo) -> ! {
    const COM1_PORT: u16 = 0x3F8;
    unsafe {
        let panic_msg = b"PANIC: Kernel panic occurred\r\n";
        for &byte in panic_msg {
            core::arch::asm!("out dx, al", in("dx") COM1_PORT, in("al") byte, options(nomem, nostack));
        }
    }
    loop {
        core::hint::spin_loop();
    }
}

// Función de emergencia sin Mutex para panic_handler (evita deadlock)
pub fn panic_print(s: &str) {
    const COM1_PORT: u16 = 0x3F8;
    for byte in s.bytes() {
        unsafe {
            core::arch::asm!("out dx, al", in("dx") COM1_PORT, in("al") byte, options(nomem, nostack));
        }
    }
}
