use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use pic8259::ChainedPics;
use spin::Mutex;
use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = 0x28;

pub static TICK_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

lazy_static! {
    static ref KEYBUF: Mutex<alloc::collections::VecDeque<u8>> = Mutex::new(alloc::collections::VecDeque::new());
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Excepciones de Intel (0-31)
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        let df_options = idt.double_fault.set_handler_fn(double_fault_handler);
        unsafe { df_options.set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX); }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_fpu_error_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.vmm_communication_exception.set_handler_fn(security_exception_handler);
        
        // IRQs del PIC
        idt[PIC_1_OFFSET].set_handler_fn(timer_handler);
        idt[PIC_1_OFFSET + 1].set_handler_fn(keyboard_handler);
        idt[PIC_1_OFFSET + 2].set_handler_fn(cascade_handler);
        idt[PIC_1_OFFSET + 3].set_handler_fn(com2_handler);
        idt[PIC_1_OFFSET + 4].set_handler_fn(com1_handler);
        idt[PIC_1_OFFSET + 5].set_handler_fn(lpt2_handler);
        idt[PIC_1_OFFSET + 6].set_handler_fn(floppy_handler);
        idt[PIC_1_OFFSET + 7].set_handler_fn(spurious_handler);
        idt[PIC_2_OFFSET].set_handler_fn(rtc_handler);
        idt[PIC_2_OFFSET + 1].set_handler_fn(acpi_handler);
        idt[PIC_2_OFFSET + 2].set_handler_fn(irq2_handler);
        idt[PIC_2_OFFSET + 3].set_handler_fn(mouse_handler);
        idt[PIC_2_OFFSET + 4].set_handler_fn(co_processor_handler);
        idt[PIC_2_OFFSET + 5].set_handler_fn(primary_ata_handler);
        idt[PIC_2_OFFSET + 6].set_handler_fn(secondary_ata_handler);
        idt[PIC_2_OFFSET + 7].set_handler_fn(pic2_spurious_handler);
        
        idt
    };
}

lazy_static! {
    static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
        // Unmask IRQ0 (timer) and IRQ1 (keyboard) on master PIC
        core::arch::asm!("out dx, al", in("dx") 0x21u16, in("al") 0xFCu8);
        // Keep slave PIC fully masked
        core::arch::asm!("out dx, al", in("dx") 0xA1u16, in("al") 0xFFu8);
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("DIVIDE ERROR");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("DEBUG");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("NMI");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("BREAKPOINT");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("OVERFLOW");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn into_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("INTO");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("BOUND RANGE");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("INVALID OPCODE");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("DEVICE NOT AVAILABLE");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    crate::serial_println!("DOUBLE FAULT");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    crate::serial_println!("INVALID TSS");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    crate::serial_println!("SEGMENT NOT PRESENT");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    crate::serial_println!("STACK SEGMENT FAULT");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    crate::serial_println!("GENERAL PROTECTION FAULT");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    let cr2: u64;
    unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
    crate::serial_println!("PAGE FAULT");
    crate::serial_println!("Error Code: {:?}", error_code);
    crate::serial_println!("CR2 (fault addr): {:#x}", cr2);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn x87_fpu_error_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("X87 FPU ERROR");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    crate::serial_println!("ALIGNMENT CHECK");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    crate::serial_println!("MACHINE CHECK");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("SIMD FLOATING POINT");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("VIRTUALIZATION");
    crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    // Comentado para evitar corrupción serial durante el bucle del mapa de memoria
    // crate::serial_println!("SECURITY EXCEPTION");
    // crate::serial_println!("Error Code: {:#x}", error_code);
    // crate::serial_println!("IP: {:#x}", stack_frame.instruction_pointer.as_u64());
    loop {}
}

// IRQ handlers
extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    TICK_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

    // Llamar al tick del scheduler
    crate::SCHEDULER.lock().tick();

    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let scancode = x86_64::instructions::port::PortReadOnly::<u8>::new(0x60).read();
        // Filter release scancodes (bit 7 = 1 means key released)
        if scancode & 0x80 == 0 {
            let mut buf = KEYBUF.lock();
            if buf.len() < 128 {
                buf.push_back(scancode);
            }
        }
        // EOI
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

pub fn pop_scancode() -> Option<u8> {
    KEYBUF.lock().pop_front()
}

extern "x86-interrupt" fn cascade_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn com2_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn com1_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn lpt2_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn floppy_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn spurious_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn rtc_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn acpi_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn irq2_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn mouse_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn co_processor_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn primary_ata_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn secondary_ata_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
        core::arch::asm!("out 0x20, al", in("al") 0x20u8, options(nomem, nostack));
    }
}

extern "x86-interrupt" fn pic2_spurious_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!("out 0xA0, al", in("al") 0x20u8, options(nomem, nostack));
    }
}
