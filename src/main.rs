#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // don't used 'cargo test', selfdefine test framework
#![test_runner(kros::test_runner)] // pointer test inner function. -> used lib 
#![reexport_test_harness_main = "test_main"] // handle test find main and no_main 

use core::panic::PanicInfo;
use kros::{memory::{print_level_4_table, translate_some_addr, translate_some_addr_from_lib}, println};
use bootloader::{entry_point, BootInfo}; // point real low inner and param info


// entry point
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World!");

    // init
    kros::init();

    // print_level_4_table(boot_info);
    // translate_some_addr(boot_info);  // huge err
    translate_some_addr_from_lib(boot_info);
    
    #[cfg(test)]
    test_main();

    kros::hlt_loop();
}


/// This function is called on panic.
// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    kros::hlt_loop();
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
