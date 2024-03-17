//! this nodule impl kros memory allocator
//! 


pub mod test_space {
    use bootloader::BootInfo;
    use alloc::alloc::{GlobalAlloc, Layout};
    use core::ptr::null_mut;
    use x86_64::{
        structures::paging::{
            mapper::MapToError, 
            FrameAllocator, 
            Mapper, 
            Page, 
            PageTableFlags, 
            Size4KiB, 
        },
        VirtAddr,
    };    

    pub struct Dummy;
    unsafe impl GlobalAlloc for Dummy {
        unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
            null_mut() // addr: 0
        }
        unsafe fn dealloc(&self, 
            _ptr: *mut u8, _layout: Layout) {
            panic!("dealloc should be never called");
        }
    }
    
    // error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait
    // #[global_allocator]
    // static ALLOCATOR_DUMMY: Dummy = Dummy;

    // define heap start and size
    pub const HEAP_START: usize = 0x_4444_4444_0000;
    pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

    // kros memory allocator     
    pub fn init_heap(
        mapper: &mut impl Mapper<Size4KiB>,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> Result<(), MapToError<Size4KiB>> {
        let page_range = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            unsafe {
                mapper.map_to(page, frame, flags, frame_allocator)?.flush() // flush cache
            };
        }
        Ok(())
    }

    pub fn heap_memory_mapper_allocator(boot_info: &'static BootInfo) {
        // inner offset mapper table 
        let mut mapper = unsafe {
            crate::memory::OffsetPageTableWarper::init(
                VirtAddr::new(
                    boot_info.physical_memory_offset
                )
            )
        };
        let mut frame_allocator = unsafe {
            crate::memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
        };
        crate::allocator::test_space::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");
    }

    pub fn create_null_box() {
        use alloc::boxed::Box;
        crate::println!("create null start!");
        let _null = Box::new(1000_000);
        crate::println!("create null end!");
    }    
}



// used lib allocator   
use linked_list_allocator::LockedHeap;
use x86_64::{structures::paging::{
        mapper::MapToError,
        FrameAllocator, 
        Mapper, 
        Size4KiB,
        Page,
        PageTableFlags,
    },  
    VirtAddr,
};

// define heap start and size
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush() // flush cache
        };
    }

    // init allocator
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); // version 0.10 to u8
    }

    Ok(())
}

// test used allocator
pub mod test_lib_space {

    pub fn create_null_box() {
        use alloc::boxed::Box;
        crate::println!("create Box start!");
        let heap_value = Box::new(1000_000);
        crate::println!("heap addr: {:p}", heap_value);
        crate::println!("create Box end!");
    } 
    pub fn create_vec_box() {
        use alloc::vec::Vec;
        crate::println!("create Vec start!");
        let mut vec = Vec::new();
        for i in 0..500 {
            vec.push(i);
        }
        crate::println!("vec addr: {:p}", vec.as_ptr());
        crate::println!("create Vec end!");
    }
    pub fn create_rc_box() {
        use alloc::vec;
        use alloc::rc::Rc;

        crate::println!("create Rc start!");
        let reference_counted = Rc::new(vec![1, 2, 3]);
        let cloned_reference = reference_counted.clone();
        
        crate::println!("current reference count is {}", Rc::strong_count(&cloned_reference));
        core::mem::drop(reference_counted);
        crate::println!("reference count is {} now", Rc::strong_count(&cloned_reference));

        crate::println!("create Rc end!");
    }
}

