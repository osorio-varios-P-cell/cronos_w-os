use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 1;

static mut TSS: TaskStateSegment = TaskStateSegment::new();

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = unsafe { gdt.append(Descriptor::tss_segment(&*core::ptr::addr_of!(TSS))) };
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        (gdt, Selectors { code_selector, data_selector, tss_selector, user_code_selector, user_data_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

pub fn init_gdt() {
    use x86_64::instructions::segmentation::Segment;
    use x86_64::instructions::tables::load_tss;
    use x86_64::registers::segmentation::CS;
    unsafe {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(&raw const STACK as *const u8);
        TSS.interrupt_stack_table[(DOUBLE_FAULT_IST_INDEX - 1) as usize] = stack_start + STACK_SIZE as u64;

        GDT.0.load();
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);

        let data_sel = GDT.1.data_selector.0;
        let code_sel = GDT.1.code_selector.0;
        core::arch::asm!(
            "push {ss_sel}",
            "push rsp",
            "pushfq",
            "push {cs_sel}",
            "lea rax, [2f + rip]",
            "push rax",
            "iretq",
            "2:",
            "mov ax, {ds_sel:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            ss_sel = in(reg) data_sel as u64,
            cs_sel = in(reg) code_sel as u64 as u64,
            ds_sel = in(reg) data_sel as u64,
            options(nostack, preserves_flags)
        );
    }
}

pub fn set_kernel_stack(stack_top: VirtAddr) {
    unsafe {
        TSS.privilege_stack_table[0] = stack_top;
    }
}
