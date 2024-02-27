#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kros::{QemuExitCode, exit_qemu, serial_println, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();      // single test
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop{}
}

fn should_fail() {
    serial_print!("should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}


// #![no_std]
// #![no_main]
// #![feature(custom_test_frameworks)]
// #![test_runner(test_runner)]
// #![reexport_test_harness_main = "test_main"]


// use core::panic::PanicInfo;
// use kros::{QemuExitCode, exit_qemu, serial_print, serial_println};


// // entry
// #[no_mangle]
// pub extern "C" fn _start() -> ! {
//     test_main();
//     loop {}
// }


// // panic
// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     serial_println!("[ok]");
//     exit_qemu(QemuExitCode::Success);
//     loop {}
// }

// // tests
// pub fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         test();     // cant test once..
//         serial_println!("[test did not panic]");
//         exit_qemu(QemuExitCode::Failed);
//     }
//     exit_qemu(QemuExitCode::Success);
// }

// #[test_case]
// fn should_fail() {
//     serial_print!("should_fail... ");
//     assert_eq!(0, 1);  
// }
