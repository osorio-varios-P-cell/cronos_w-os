use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use pic8259::ChainedPics;
use spin::Mutex;
use lazy_static::lazy_static;

const PIC_1_OFFSET: u8 = 0x20;
const PIC_2_OFFSET: u8 = 0x28;

pub static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

// PIT I/O ports
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

/// Inicializa el PIT (Programmable Interval Timer) a ~100 Hz
pub fn init_pit() {
    let divisor: u16 = 11932; // 1.193182 MHz / 100 Hz ≈ 11932
    unsafe {
        // Command: counter 0, lobyte/hibyte, mode 3 (square wave), binary
        core::arch::asm!("out dx, al", in("dx") PIT_COMMAND, in("al") 0x36u8);
        core::arch::asm!("out dx, al", in("dx") PIT_CHANNEL0, in("al") (divisor & 0xFF) as u8);
        core::arch::asm!("out dx, al", in("dx") PIT_CHANNEL0, in("al") (divisor >> 8) as u8);
    }
}

// 256 entries * 16 bytes = 4096 bytes for IDT
const IDT_SIZE: usize = 4096;
#[repr(C, align(16))]
struct AlignedIdt([u8; IDT_SIZE]);

static mut IDT_BYTES: AlignedIdt = AlignedIdt([0; IDT_SIZE]);

pub fn init_idt() {
    unsafe {
        let idt = &mut *(IDT_BYTES.0.as_mut_ptr() as *mut InterruptDescriptorTable);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[PIC_1_OFFSET as usize].set_handler_fn(timer_handler);
        idt.load();
    }
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
        // Unmask IRQ0 (master PIC data port), keep all others masked
        core::arch::asm!("out dx, al", in("dx") 0x21u16, in("al") 0xFEu8);
        core::arch::asm!("out dx, al", in("dx") 0xA1u16, in("al") 0xFFu8);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("BREAKPOINT");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    crate::serial_println!("SP: {:#x}", stack_frame.stack_pointer.as_u64());
    crate::serial_println!("FLAGS: {:#x}", unsafe { stack_frame.cpu_flags });
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    // BUG #13 corregido: mostrar IP, SP, FLAGS en double_fault
    crate::serial_println!("DOUBLE FAULT");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    crate::serial_println!("SP: {:#x}", stack_frame.stack_pointer.as_u64());
    crate::serial_println!("FLAGS: {:#x}", unsafe { stack_frame.cpu_flags });
    loop {}
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    // BUG #13 corregido: mostrar IP, SP, FLAGS en page_fault
    crate::serial_println!("PAGE FAULT");
    crate::serial_println!("Error Code: {:?}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    crate::serial_println!("SP: {:#x}", stack_frame.stack_pointer.as_u64());
    crate::serial_println!("FLAGS: {:#x}", unsafe { stack_frame.cpu_flags });
    loop {}
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    TICK_COUNT.fetch_add(1, Ordering::SeqCst);
    unsafe {
        PICS.lock().notify_end_of_interrupt(0);
    }
}

lazy_static! {
    static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
}
