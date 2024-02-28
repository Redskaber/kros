#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // don't used 'cargo test', selfdefine test framework
#![test_runner(kros::test_runner)] // pointer test inner function. -> used lib 
#![reexport_test_harness_main = "test_main"] // handle test find main and no_main 

use core::panic::PanicInfo;
use kros::println;


// entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World!");

    // init
    kros::init();

    // create break point used soft interrupt `int3`
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}


/// This function is called on panic.
// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kros::test_panic_handler(info)      // used lib.rs 
}


#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
