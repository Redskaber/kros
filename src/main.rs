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

    // Cr3 read
    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    // Cr2 test
    unsafe {
        let p = 0x2031b2 as *mut u8;
        println!("Read 0x2031b2: {:?}", *p);
        *p = 24; 
        println!("Write 0x2031b2: {:?}", *p);
    }

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
