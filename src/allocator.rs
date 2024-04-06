//! this nodule impl kros memory allocator
//! allocator used:
//!  - small used(8, 2048): fixed size allocator
//!  - big used(2048, ~): linked list allocator 

// pub mod dummy;
/// myself define allocator simple `bump`
// pub mod bump; 
pub mod linked_list;
// pub mod case;
pub mod fixed_size_block;

// used lib allocator   
use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator, 
        Mapper, 
        Size4KiB,
        Page,
        PageTableFlags,
    },  
    VirtAddr,
};

use alloc::alloc::Layout;
use spin::{mutex::Mutex, MutexGuard};


// 由于GlobalAlloc 参数是&self,而我们需要对&mut self进行操作，=> Warper(Allocator) => 足够通用可以放置在allocator 父模块中
pub struct Locked<T> {
    inner: Mutex<T>
}

impl <T> Locked<T> {
    
    /// Lock the underlying mutex.
    pub const fn new(value: T) -> Self {
        Locked {
            inner: Mutex::new(value),
        }
    }
    
    /// Lock the underlying mutex.
    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock()
    }
}

#[allow(dead_code)]
pub struct AddrRegion<N: 'static>{
    addr_region: &'static mut N,
    alloc_addr: usize,
}

impl <N>AddrRegion<N> {
    fn new(addr_region: &'static mut N, alloc_addr: usize) -> Self {
        AddrRegion {
            addr_region,
            alloc_addr,
        }
    }
}

pub unsafe trait Allocator<N: 'static> {
    /// allocator one address from heap is not used.
    unsafe fn add_free_region(&mut self, heap_start: usize, heap_size: usize);

    /// find one not used address region from heap area.
    fn find_free_region(&mut self, size: usize, align: usize) -> Option<AddrRegion<N>>;

    /// tool function: used allocator memory from memory region.
    fn alloc_from_region(region: &N, size: usize, align: usize) -> Result<usize, ()>;

    /// tool function: used align memory address, return alloc address and align size tuple.
    fn size_align(layout: Layout) -> (usize, usize);
}

/// align_up: 对齐地址，向上对齐，align必须是2的倍数，!(align -1) -> bit_mask(用于对齐区段)
/// ```Rust
/// let remainder = addr % align;
/// if remainder == 0 {
///     addr
/// } else {
///     addr - remainder + align    //  range: [ |(addr-remainder+align)  ] |(remainder)
/// }
/// ```
/// addr = 1_0010    align = 8
/// addr + align -1 
///  1_0010           
///+ 0_0111
///  1_1001      0_0111 -> !(0_0111) -> 1...11000
///  ||
///  v
/// (addr + align - 1) & !(align - 1) (addr + align -1) 向上取整
///   0...01_1000   24  
/// & 1...11_1000   -8 (8bit[0..7]) => 2^3 -1 => 3 个 0
///   0...01_1000   24
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// define heap start and size
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// static ALLOCATOR: Locked<linked_list::LinkListAllocator> = Locked::new(linked_list::LinkListAllocator::new());
// static ALLOCATOR: Locked<case::CaseAllocator> = Locked::new(case::CaseAllocator::new());
static ALLOCATOR: Locked<fixed_size_block::FixedSizeBlockAllocator> = Locked::new(fixed_size_block::FixedSizeBlockAllocator::new());

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
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); // used self define `dump allocator`
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

