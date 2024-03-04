//! test kros double fault -> triple fault

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use kros::{
    serial_print,
    serial_println,
    exit_qemu,
    QemuExitCode,
};

use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};

// 我们需要注册一个自定义的 double fault 处理函数，在被触发的时候调用 `exit_qemu(QemuExitCode::Success)` 函数，而非使用默认的逻辑。
lazy_static!{
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(kros::gdt::DOUBLE_FAULT_IST_INDEX); // mask -> err
        };
        idt
    };
}

pub fn init_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");
    // init
    kros::gdt::init();
    init_idt(); 

    // trigger a stack overflow
    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kros::test_panic_handler(info)
}

// test case
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read_only(); // prevent tail recursion optimizations
}
