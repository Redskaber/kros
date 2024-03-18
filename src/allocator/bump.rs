//! this module is impl simple allocator `Bump`.


use core::ptr;
use alloc::alloc::{GlobalAlloc, Layout};
use super::{Locked, align_up};

#[allow(dead_code)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Create a new empty bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0, // counter
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    /// 
    /// This method is unsafe because the caller must ensure that the given 
    /// memory range is unused, Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        // check verify  assert!()
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get lock

        // align up the start of the heap
        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.check_add(layout.size()) { 
            Some(end) => end,
            None => return ptr::null_mut(),
        };
            
        // check if the heap is big enough
        if alloc_end > bump.heap_end {
            return ptr::null_mut();
        }else{
            bump.next = alloc_end;
            bump.allocations +=1;
            alloc_start as *mut u8
        }
    }

    /// dealloc impl `dealloc` method
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -=1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
