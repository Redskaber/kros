//! this moudle impl Rust kernel cpu interrupt.

use crate::println;

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};
use lazy_static::lazy_static;

// nice func used heap memory -> 'static life, but None. 
// used lazy load handle: error[E0597]: `idt` does not live long enough
lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

// used x86_64 moudle create IDE object
pub fn init_idt() {
    IDT.load(); // need lidt(Load Interrupt Descriptor Table Register)
}

// create func used handle breakpoint.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}


// test breakpoint
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

}

