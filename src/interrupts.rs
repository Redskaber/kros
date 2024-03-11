//! this moudle impl Rust kernel cpu interrupt.
//! `
//!     - InterruptDescriptorTable
//!         - Entry
//!             - HandlerFuncType
//!             - EntryOption
//!         - InterruptStackFrame
//!             - InterruptStackFrameValue
//!         - Interruptindex
//!    - Prom-interruot-controller(PIC) -> hardware handler  -> IDT interrupts
//!         Timer
//!         Kryboard
//! `

use crate::{
    gdt, hlt_loop, print, println
};

use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};

// hardware interrupts agent.
use pic8259::ChainedPics; 
use spin::mutex::Mutex;

use lazy_static::lazy_static;


// 可编程中断控制器（PIC）
// used pic8259 define ChainedPis -> Pic(Master /Slave)
// Pic {offset:u8, command: u8, data: u8}
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // used ChainedPics::new_contiguous

pub static PICS: Mutex<ChainedPics> = {
    Mutex::new(unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    })
};

// load: went ChainedPics to Interrupt Descriptor Table
// add_handler_fun: set hardware interrupt handler fucntion.
//     - eoi: note CPU interrupt over
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex { 
    Timer = PIC_1_OFFSET,  // hardware: Timer interrupt
    Keyboard,   // handler: Keyboard interrupt
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}


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

        // hardware handler
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

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
extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE_FAULT\n{:#?}", _stack_frame);
    println!("ERROR CODE: {:#?}", _error_code);

    // C2 register： page fault -> cpu auto write to exception virtual addr.
    use x86_64::registers::control::Cr2;
    println!("Accessed Address: {:?}", Cr2::read()); // error address 6
    hlt_loop();
}

// create func used handle double fault.
extern "x86-interrupt" fn double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", _stack_frame);
}

/// create func used handler hardware.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    // used EOI over handler -> unsafe
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// create func used handler keyboard.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {

    use spin::Mutex;
    use x86_64::instructions::port::Port;
    // used pc_keyboard: get scan_code end this interrupt
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    // create once keyboard globel.
    lazy_static!{
        static ref KEYBOARD: Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> = 
            Mutex::new(
                Keyboard::new(
                    ScancodeSet1::new(), 
                    layouts::Uk105Key, 
                    HandleControl::Ignore,
                )
            );
    }
    
    // ps/2 port addr; USB comp PS/2 .so used ps/2 can normal used. => USB
    let mut keyboard = KEYBOARD.lock();
    let mut ps2_port = Port::new(0x60);
    let scan_code: u8 = unsafe {ps2_port.read()};

    // pc_keyboard handler scan code.
    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) { // Result<Option<KeyEvent>, Error>
        if let Some(dec_key) = keyboard.process_keyevent(key_event) {
            match dec_key {
                DecodedKey::RawKey(dec_key) => print!("{:?}", dec_key),
                DecodedKey::Unicode(character) => print!("{}", character),
            }
        }
    }

    // EOI end keyboard interrupt.
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }

}

// test breakpoint
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

// // test breakpoint
// #[test_case]
// fn test_pagefault_exception() {
//     crate::serial_println!();
//     // invoke a breakpoint exception
//     unsafe {
//         *(0xdeadbeef as *mut u8) = 42;
//     }
// }

// // test overflow
// #[test_case]
// #[allow(unconditional_recursion)]
// fn stack_overflow() {
//     fn _stack_overflow(){
//         _stack_overflow();
//     }
//     _stack_overflow();
// }

