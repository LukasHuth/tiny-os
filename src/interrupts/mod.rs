use spin::Once;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use crate::memory::MemoryController;

mod gdt;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        let double_fault_entry_options = idt.double_fault.set_handler_fn(double_fault_handler);
        unsafe { double_fault_entry_options.set_stack_index(DOUBLE_FAULT_IST_INDEX as u16) };
        idt
    };
}

const DOUBLE_FAULT_IST_INDEX: usize = 0;

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

pub fn init(memory_controller: &mut MemoryController) {
    let double_fault_stack = memory_controller
        .alloc_stack(1)
        .expect("Could not allocate a double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            VirtAddr::new(double_fault_stack.top() as u64);
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();

    unsafe {
        x86_64::instructions::segmentation::CS::set_reg(code_selector);
        x86_64::instructions::tables::load_tss(tss_selector);
    }

    IDT.load();
}
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}
