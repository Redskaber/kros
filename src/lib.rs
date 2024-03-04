#![no_std] // don't link the Rust standard library
#![cfg_attr(test, no_main)] // cargo test -> #![no_main]
#![feature(custom_test_frameworks)] // used custom framework
#![test_runner(crate::test_runner)] // set test runner function
#![reexport_test_harness_main = "test_main"] // rename test entry function name
#![feature(abi_x86_interrupt)]  // interrupt used unstable feature

use core::panic::PanicInfo;

pub mod vga_buffer; // export
pub mod serial; // export
pub mod interrupts; // export
pub mod gdt; // export

/// init area:
/// - interrupt
/// - gdt
pub fn init() {
    gdt::init();
    interrupts::init_idt();
}


/// This define custom test framework.
/// handle test print statement complex output
#[allow(dead_code)]
pub trait Testable {
    fn run(&self) -> ();
}

impl <T> Testable for T 
    where
        T: Fn(),
{
    fn run(&self) -> () {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// default find main function execute tests. pub -> Union(main, test)
pub fn test_runner(tests: &[&dyn Testable]) {  
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

// our panic handler in test mode: pub -> Union(main, test)
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Entry ponit `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init(); 
    test_main();
    loop{}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

/// this impl send msg to qemu virtual shutdown os.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32); 
    }
}
