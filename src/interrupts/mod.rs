use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

use pic8259::ChainedPics;
use spin::Mutex;
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

        idt[0x20].set_handler_fn(timer_interrupt_handler);
        idt[0x21].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

const DOUBLE_FAULT_IST_INDEX: usize = 0;

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

pub fn init(memory_controller: &mut MemoryController) {
    unsafe { PICS.lock().initialize() };
    unsafe { PICS.lock().write_masks(0b11111100, 0xFF) };
    start_timer();
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
fn start_timer() {
    const PIT_FREQ: u32 = 1193182; // PIT input clock frequency
    const HZ: u32 = 1_000;
    const DIVISOR: u16 = (PIT_FREQ / HZ) as u16;
    use x86_64::instructions::port::Port;
    let mut command_port = Port::<u8>::new(0x43);
    let mut channel0 = Port::<u8>::new(0x40);
    unsafe {
        command_port.write(0x34);
        channel0.write((DIVISOR & 0xFF) as u8);
        channel0.write((DIVISOR >> 8) as u8);
    }
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
pub static TIMER: AtomicU64 = AtomicU64::new(0);
static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER.fetch_add(1, Ordering::Relaxed);
    unsafe { PICS.lock().notify_end_of_interrupt(0x20) };
}
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port = Port::<u8>::new(0x60);
    let _scancode: u8 = unsafe { port.read() }; // Read scancode
    unsafe {
        PICS.lock().notify_end_of_interrupt(0x21);
    }
}
