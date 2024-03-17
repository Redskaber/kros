//! test heap allocator in lib.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kros::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(heap_main);

fn heap_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use kros::allocator;
    use kros::memory::{BootInfoFrameAllocator, OffsetPageTableWarper};

    // init (gdt, idt, interrupt)
    kros::init();

    // memory mapper
    let mut mapper = unsafe {
        OffsetPageTableWarper::init(
            VirtAddr::new(
                boot_info.physical_memory_offset as u64
            )
        )
    };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    // heap allocator
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    // test 
    test_main();
    kros::hlt_loop()
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kros::test_panic_handler(info)
}


// test lib.rs
use alloc::boxed::Box;
use alloc::vec::Vec;
use kros::allocator::HEAP_SIZE;


#[test_case]
fn simple_allocation() {
    let x = Box::new(5);
    assert_eq!(*x, 5);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
