//! Módulo de gestión de interrupciones para CRONOS W-OS
//! Implementa IDT y manejo de interrupciones

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(0); // New stack index
        }
        idt
    };
}

/// Inicializa la IDT
pub fn init_idt() {
    IDT.load();
    println!("⚡ IDT inicializada");
}

/// Manejador de breakpoint
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("💔 EXCEPCIÓN: Breakpoint\n{:#?}", stack_frame);
}

/// Manejador de double fault
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("💔 EXCEPCIÓN: Double Fault\n{:#?}", stack_frame);
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

/// Manejador de interrupción de timer
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Implementación de timer interrupt
}

/// Manejador de interrupción de teclado
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Implementación de keyboard interrupt
}
