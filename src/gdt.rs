//! Global Descriptor Table: [u64; 8]  u64 -> VirtAddr pointer
//!     - load TSS
//!         - privilege_stack_table [VirtAddr; 3],
//!         - interrupt_stack_table [VirtAddr; 7],
//!         - iomap_base u16

use core::ptr::addr_of; // static addr

/// Task Status Segment (TSS)
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

/// Global Descriptor Table (GDT)
use x86_64::structures::gdt::{
    Descriptor, 
    GlobalDescriptorTable, 
    SegmentSelector
};

/// buidler once TSS 
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static!{
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // build stack base addr.
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // stack start -> end
            let stack_start = VirtAddr::from_ptr(unsafe {addr_of!(STACK)});
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        // init interrupt stack table finish
        tss
    };
}


// load TSS
// 我们已经创建了一个TSS，现在的问题就是怎么让CPU使用它。不幸的是这事有点繁琐，因为TSS用到了分段系统（历史原因）。
// 但我们可以不直接加载，而是在[全局描述符表]
struct GdtSelector {
    gdt: GlobalDescriptorTable,
    selector: Selector,
}

impl GdtSelector {
    fn new(gdt: GlobalDescriptorTable, selector: Selector) -> Self {
        GdtSelector {
            gdt, 
            selector,
        }
    }
}

struct Selector {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

impl Selector {
    fn new(code_selector: SegmentSelector, tss_selector: SegmentSelector) -> Self {
        Selector {
            code_selector,
            tss_selector,
        }
    }
}


lazy_static!{
    static ref GDT_SELECTOR: GdtSelector = {
        // build GDT 
        let mut gdt = GlobalDescriptorTable::new();
        // segment
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        // combi
        let selector = Selector::new(code_selector, tss_selector);
        GdtSelector::new(gdt, selector)
    };
}

/// load gdt
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT_SELECTOR.gdt.load(); // load gdt
    unsafe {
        load_tss(GDT_SELECTOR.selector.tss_selector); // load tss
        CS::set_reg(GDT_SELECTOR.selector.code_selector); // load code
    }
}
