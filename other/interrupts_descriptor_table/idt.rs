//! self define kernel interrupt.
//! - Exception: https://wiki.osdev.org/Exceptions

use core::{
    fmt,
    ops::Deref,
    marker::PhantomData,
};
use x86_64::{
    addr::VirtAddr,
    structures::gdt::SegmentSelector,
};
use bitflags::bitflags;
use bit_field::BitField;
use volatile::Volatile;



/// Handle Error Function type.
#[cfg(feature = "abi_x86_interrupt")]   // used feature abi_x86_interrupt
type HandlerFunc = 
    extern "x86-interrupt" fn (stack_frame: InterruptStackFrame); // normal handler function.
#[cfg(feature = "abi_x86_interrupt")]
type HandlerFuncWithErrCode = 
    extern "x86-interrupt" fn(stack_frame: InterruptStackFrame, error_code: u64); // normal handler has error code function.
#[cfg(feature = "abi_x86_interrupt")]
pub type DivergingHandlerFunc = 
    extern "x86-interrupt" fn(stack_frame: InterruptStackFrame) -> !; // Hardware exception
#[cfg(feature = "abi_x86_interrupt")]
pub type DivergingHandlerFuncWithErrCode = 
    extern "x86-interrupt" fn(stack_frame: InterruptStackFrame, error_code: u64) -> !;  // double error handler.
#[cfg(feature = "abi_x86_interrupt")]
pub type PageFaultHandlerFunc =
    extern "x86-interrupt" fn(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode);


#[cfg(not(feature = "abi_x86_interrupt"))] // not used feature abi_x86_interrupt
#[derive(Copy, Clone, Debug)]   // Easy to handle and debug
pub struct HandlerFunc(());
#[cfg(not(feature = "abi_x86_interrupt"))]
#[derive(Copy, Clone, Debug)]
pub struct HandlerFuncWithErrCode(());
#[cfg(not(feature = "abi_x86_interrupt"))]
#[derive(Copy, Clone, Debug)]
pub struct DivergingHandlerFunc(());
#[cfg(not(feature = "abi_x86_interrupt"))]
#[derive(Copy, Clone, Debug)]
pub struct DivergingHandlerFuncWithErrCode(());
#[cfg(not(feature = "abi_x86_interrupt"))]
#[derive(Copy, Clone, Debug)]
pub struct PageFaultHandlerFunc(());

bitflags! { // Convenient bit operation
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
    pub struct PageFaultErrorCode: u64 { // U32 -> U64      
        const PROTECTION_VIOLATION = 1;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
        const PROTECTION_KEY = 1 << 5;
        const SHADOW_STACK = 1 << 6;
        const SGX = 1 << 15;
        const RMP = 1 << 31; // AMD-only
    }
}



#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    /// vector nr. 0
    pub division_error: Entry<HandlerFunc>,
    /// vector nr. 1
    pub debug: Entry<HandlerFunc>,
    /// vector nr. 2
    pub non_maskable_interrupt: Entry<HandlerFunc>,    
    /// vector nr. 3
    pub breakpoint: Entry<HandlerFunc>,
    /// vector nr. 4
    pub overflow: Entry<HandlerFunc>,
    /// vector nr. 5
    pub bound_range_exceeded: Entry<HandlerFunc>,
    /// vector nr. 6
    pub invalid_opcode: Entry<HandlerFunc>,
    /// vector nr. 7
    pub device_not_available: Entry<HandlerFunc>,
    /// vector nr. 8
    pub double_fault: Entry<DivergingHandlerFuncWithErrCode>, // daouble fault
    /// vector nr. 9
    coprocessor_segment_overrun: Entry<HandlerFunc>,
    /// vector nr. 10
    pub invaild_tss: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 11
    pub segment_not_present: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 12
    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 13
    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 14
    pub page_fault: Entry<PageFaultHandlerFunc>, // page fault
    /// vector nr. 15
    reserved_15: Entry<HandlerFunc>,
    /// vector nr. 16
    pub x87_floating_point_exception: Entry<HandlerFunc>,
    /// vector nr. 17
    pub alignment_check: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 18
    pub machine_check: Entry<DivergingHandlerFunc>, // machine check
    /// vector nr. 19
    pub simd_floating_point_exception: Entry<HandlerFunc>,
    /// vector nr. 20
    pub virtualization_exception: Entry<HandlerFunc>,
    /// vector nr. 21
    pub control_protection_exception: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 22-27
    reserved_22_27: [Entry<HandlerFunc>; 6],
    /// vector nr. 28
    pub hypervisor_injection_exception: Entry<HandlerFunc>,
    /// vector nr. 29
    pub vmm_communication_exception: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 30
    pub security_exception: Entry<HandlerFuncWithErrCode>,
    /// vector nr. 31
    reserved_31: Entry<HandlerFunc>,
    /// used define interrupts
    interrupts: [Entry<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    #[inline]
    pub fn new() -> Self {
        InterruptDescriptorTable {
            division_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            coprocessor_segment_overrun: Entry::missing(),
            invaild_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            reserved_15: Entry::missing(),
            x87_floating_point_exception: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point_exception: Entry::missing(),
            virtualization_exception: Entry::missing(),
            control_protection_exception: Entry::missing(),
            reserved_22_27: [Entry::missing(); 6],
            hypervisor_injection_exception: Entry::missing(),
            vmm_communication_exception: Entry::missing(),
            security_exception: Entry::missing(),
            reserved_31: Entry::missing(),
            interrupts: [Entry::missing(); 256 -32],
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    /// used `lidt` load IDT to CPU.
    #[cfg(feature = "instructions")]
    #[inline]
    pub fn load(&'static self) {
        unsafe { self.load_unsafe()};
    }
    #[cfg(feature = "instructions")]
    #[inline]
    pub unsafe fn load_unsafe(&self) {
        use x86_64::instructions::tables::lidt;
        unsafe {
            lidt(&self.pointer()); 
        }
    }
    // need 
    #[cfg(feature = "instructions")]
    #[inline]
    fn pointer(&self) -> x86_64::structures::DescriptorTablePointer {
        use core::mem::size_of;
        x86_64::structures::DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        }
    }
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry<F> {
    fpointer_low: u16,
    gdt_selector: SegmentSelector,  // u16
    options: EntryOptions,  // u16
    fpointer_middle: u16,
    fpointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>, // hold F
}

impl Entry<F> {
    #[inline]
    pub const fn missing() -> Self {
        Entry {
            fpointer_low: 0,
            gdt_selector: SegmentSelector::NULL,
            options: EntryOptions::minimal(),
            fpointer_middle: 0,
            fpointer_high: 0,
            reserved: 0,
            phantom: PhantomData,
        }
    }
    #[inline]
    pub unsafe fn set_handler_addr(&mut self, addr: VirtAddr) -> &mut EntryOptions {
        use x86_64::instructions::segmentation::{Segment,CS};

        let addr = addr.as_u64();

        self.fpointer_low = addr as u16;
        self.fpointer_middle = (addr >> 16) as u16;
        self.fpointer_high = (addr >> 32) as u32;

        self.gdt_selector = CS::get_reg().0;

        self.options.set_present(true);
        &mut self.options
    }
    #[inline]
    pub fn get_handler_addr(&self) -> VirtAddr {
        let addr = self.fpointer_low as u64 
            | (self.fpointer_middle as u64) << 16 
            | (self.fpointer_high as u64) << u32;

        VirtAddr::new_truncate(addr)
    }
}


pub trait HandlerFuncType {
    fn to_virt_addr(self) -> VirtAddr;
}

macro_rules! impl_handler_func_type {
    ($f: ty) => {
        #[cfg(feature = "abi_x86_interrupt")]
        impl HandlerFuncType for $f {
            #[inline]
            fn to_virt_addr(self) -> VirtAddr {
                VirtAddr::new(self as u64)
            }
        }
    };
}

impl_handler_func_type!(HandlerFunc);
impl_handler_func_type!(HandlerFuncWithErrCode);
impl_handler_func_type!(PageFaultErrorCode);
impl_handler_func_type!(DivergingHandlerFunc);
impl_handler_func_type!(DivergingHandlerFuncWithErrCode);


impl <F: HandlerFuncType>  Entry<F> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: F) -> &mut EntryOptions {
        unsafe { self.set_handler_addr(handler.to_virt_addr())}
    }
}



// interrupt EntryOptions https://os.phil-opp.com/catching-exceptions/#the-interrupt-descriptor-table
#[repr(transparent)]
pub struct EntryOptions(u16);

impl EntryOptions {
    #[inline]
    const fn minimal() -> Self {
        EntryOptions(0b1110_0000_0000)  // must(9-11: 1 && 12: 0)
    }

    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    #[inline]
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {  // disable interrupt?
        self.0.set_bit(8, !disable);
        self
    }

    #[inline]
    pub fn set_privilege_level(&mut self, dpl: x86_64::PrivilegeLevel) -> &mut Self { // set level
        self.0.set_bits(13..15, dpl as u16);
        self
    }
    
    #[inline]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self { // choice stack index
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.0.set_bits(0..3, index + 1);
        self
    }
}


#[repr(C)]
pub struct InterruptStackFrame{ // stack frame
    value: InterruptStackFrameValue,
}

impl InterruptStackFrame {
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Volatile<&mut InterruptStackFrameValue> {
        Volatile::new(&mut self.value)
    }
}

impl Deref for InterruptStackFrame {
    type Target = InterruptStackFrameValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    // error code -> handle function
    pub instruction_pointer: VirtAddr, // RIP
    pub code_segment: u64, // CS
    pub cpu_flags: u64,
    pub stack_pointer: VirtAddr, // RSP
    pub stack_segment: u64, // SS
}


impl InterruptStackFrameValue {
    /// Call the `iretq` (interrupt return) instruction.
    #[inline(always)]
    #[cfg(feature = "instructions")]
    pub unsafe fn iretq(&self) -> ! {
        unsafe {
            core::arch::asm!(
                "push {stack_segment}",
                "push {new_stack_pointer}",
                "push {rflags}",
                "push {code_segment}",
                "push {new_instruction_pointer}",
                "iretq",
                rflags = in(reg) self.cpu_flags,
                new_instruction_pointer = in(reg) self.instruction_pointer.as_u64(),
                new_stack_pointer = in(reg) self.stack_pointer.as_u64(),
                code_segment = in(reg) self.code_segment,
                stack_segment = in(reg) self.stack_segment,
                options(noreturn)
            )
        }
    }
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Hex(u64);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }

        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &Hex(self.cpu_flags));
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}

pub fn init_load_test() {
    let idt = InterruptDescriptorTable::new();
    idt.load();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
}

// create func used handle breakpoint.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}



