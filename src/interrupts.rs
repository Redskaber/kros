//! this moudle impl Rust kernel cpu interrupt.
//! `
//!     - InterruptDescriptorTable
//!         - Entry
//!             - HandlerFuncType
//!             - EntryOption
//!         - InterruptStackFrame
//!             - InterruptStackFrameValue
//!         ...
//! `

use crate::{
    gdt,
    println,
};

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};
use lazy_static::lazy_static;

// nice func used heap memory -> 'static life, but None. 
// used lazy load handle: error[E0597]: `idt` does not live long enough
lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // handler double fault
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                // double fault change safe stack.
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

/// used macro impl anything exception set hadnler function.

// used x86_64 moudle create IDE object
pub fn init_idt() {
    IDT.load(); // need lidt(Load Interrupt Descriptor Table Register)
}

// create func used handle breakpoint.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// create func used handle page fault.
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE_FAULT\n{:#?}", stack_frame);
    println!("ERROR CODE: {:#?}", error_code);
    loop {}
}

// create func used handle double fault.
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


// test breakpoint
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

}

// test breakpoint
#[test_case]
fn test_pagefault_exception() {
    // invoke a breakpoint exception
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    }
}

// test overflow
#[test_case]
fn stack_overflow() {
    fn _stack_overflow(){
        _stack_overflow();
    }
    _stack_overflow();
}

