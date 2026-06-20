pub struct Ps2Keyboard;

impl Ps2Keyboard {
    pub fn initialize() {
        unsafe {
            Self::write_command(0xAD);
            Self::write_command(0xA7);

            // Timeout de seguridad para evitar cuelgues en CI
            let mut timeout = 0x10000;
            while (Self::read_status() & 0x01) != 0 && timeout > 0 {
                let _ = Self::read_data();
                timeout -= 1;
            }

            Self::write_command(0x20);
            let mut ccb = Self::read_data();
            ccb |= 0x01;
            ccb &= !0x10;
            Self::write_command(0x60);
            Self::write_data(ccb);
            Self::write_command(0xAE);
            crate::serial_println!("[PS2] Keyboard initialized with IRQ1 active");
        }
    }

    #[inline]
    unsafe fn read_status() -> u8 {
        let mut val: u8 = 0;
        core::arch::asm!("in al, dx", in("dx") 0x64u16, out("al") val);
        val
    }

    #[inline]
    unsafe fn read_data() -> u8 {
        let mut val: u8 = 0;
        core::arch::asm!("in al, dx", in("dx") 0x60u16, out("al") val);
        val
    }

    #[inline]
    unsafe fn write_data(val: u8) {
        core::arch::asm!("out dx, al", in("dx") 0x60u16, in("al") val);
    }

    #[inline]
    unsafe fn write_command(cmd: u8) {
        core::arch::asm!("out dx, al", in("dx") 0x64u16, in("al") cmd);
    }
}

/// Translate PS/2 scancode (Set 1, press only) to ASCII
pub fn scancode_to_char(scancode: u8) -> Option<char> {
    match scancode {
        0x01 => Some(0x1b as char), // Escape
        0x02 => Some('1'), 0x03 => Some('2'), 0x04 => Some('3'),
        0x05 => Some('4'), 0x06 => Some('5'), 0x07 => Some('6'),
        0x08 => Some('7'), 0x09 => Some('8'), 0x0a => Some('9'),
        0x0b => Some('0'), 0x0c => Some('-'), 0x0d => Some('='),
        0x0e => Some(0x08 as char), // Backspace
        0x0f => Some('\t'),
        0x10 => Some('q'), 0x11 => Some('w'), 0x12 => Some('e'),
        0x13 => Some('r'), 0x14 => Some('t'), 0x15 => Some('y'),
        0x16 => Some('u'), 0x17 => Some('i'), 0x18 => Some('o'),
        0x19 => Some('p'), 0x1a => Some('['), 0x1b => Some(']'),
        0x1c => Some('\n'), // Enter
        0x1d => None, // LCtrl
        0x1e => Some('a'), 0x1f => Some('s'),
        0x20 => Some('d'), 0x21 => Some('f'), 0x22 => Some('g'),
        0x23 => Some('h'), 0x24 => Some('j'), 0x25 => Some('k'),
        0x26 => Some('l'), 0x27 => Some(';'), 0x28 => Some('\''),
        0x29 => Some('`'),
        0x2a => None, // LShift
        0x2b => Some('\\'),
        0x2c => Some('z'), 0x2d => Some('x'), 0x2e => Some('c'),
        0x2f => Some('v'), 0x30 => Some('b'), 0x31 => Some('n'),
        0x32 => Some('m'), 0x33 => Some(','), 0x34 => Some('.'),
        0x35 => Some('/'),
        0x36 => None, // RShift
        0x37 => Some('*'), // Keypad *
        0x38 => None, // LAlt
        0x39 => Some(' '), // Space
        0x3a => None, // CapsLock
        0x3b..=0x44 => {
            // F1-F10
            let n = (scancode - 0x3b + 1) as u8;
            Some(if n <= 9 { (b'0' + n) as char } else { '0' })
        }
        0x47 => Some('7'), 0x48 => Some('8'), 0x49 => Some('9'),
        0x4a => Some('-'), 0x4b => Some('4'), 0x4c => Some('5'),
        0x4d => Some('6'), 0x4e => Some('+'), 0x4f => Some('1'),
        0x50 => Some('2'), 0x51 => Some('3'), 0x52 => Some('0'),
        0x53 => Some('.'),
        _ => None,
    }
}
