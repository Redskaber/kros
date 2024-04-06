//! this module is kros memory allocator
//! 
//! this allocator use linked list
//! 
//! this allocator is simple
//! 

use super::{align_up, Locked, AddrRegion, Allocator};

use core::{mem, ptr};
use alloc::alloc::{GlobalAlloc, Layout};

#[allow(dead_code)]
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// create a new list node
    const fn new(size: usize) -> Self {
        ListNode{size, next: None}
    }

    /// get the start address of the list node
    fn start_addr(&self) -> usize {
        self as *const Self as usize 
    }

    // get the end address of the list node
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }    
}


#[allow(dead_code)]
pub struct LinkListAllocator {
    head: ListNode,
}
impl LinkListAllocator {   
    
    /// Creates an empty LinkedListAllocator
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    /// 
    /// This function is unsafe because the caller must guarantee that the given heap bounds are valid and that the heap is unused. This method must be called only once to avoid dangling pointers.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }
}

unsafe impl Allocator<ListNode> for LinkListAllocator {
    /// Adds the given memory region to the front of the list.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // create a new list node and append it at the start of the list
        let mut node = ListNode::new(size);
        let head_next = self.head.next.take();
        node.next = head_next;
        
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);

        self.head.next = Some(&mut *node_ptr);
    }
    
    /// Locks for a free region with the given size and alignment and removes it from the list.
    /// 
    /// Returns a tuple of the list node and the start address of the allocation.
    fn find_free_region(&mut self, size: usize, align: usize) -> Option<AddrRegion<ListNode>> {

        // reference to current list node, updated for each iteration
        let mut current = &mut self.head;

        // look for a large enough free region in list
        while let Some(ref mut region) =  current.next {
            // filter: region area don't allocator memory, if region align after.
            if let Ok(alloc_addr) = Self::alloc_from_region(&region, size, align){                
                // region suitable for allocation -> remove node from list
                let addr_region = current.next.take().unwrap();
                let region_next = addr_region.next.take();
                current.next = region_next;
                // return address and allocated size               
                return Some(AddrRegion::new(addr_region, alloc_addr));
            }else {
                //region not suitable -> continue with next region
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }
    
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start
            .checked_add(size)
            .ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        };

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // rest of region too small to hold a ListNode (required because the allocation 
            //splits the region in a used and a free part)
            return Err(());
        }
        
        Ok(alloc_start)
    }
    
    /// Adjust the given layout so that the resulting allocated memory
    /// region is also capable of storing a `ListNode`.
    ///
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}



unsafe impl GlobalAlloc for Locked<LinkListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //perform layout adjustments
        let (size, algin) = LinkListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some(AddrRegion{addr_region, alloc_addr}) = allocator.find_free_region(size, algin) {
            let alloc_end = alloc_addr.checked_add(size).expect("find_free_region overflow!");
            let excess_size = addr_region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_addr as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    /// 释放内存, 将释放的内存添加到链表中，
    /// 存在一个问题，就是过多的通过分配内存还会将heap 中的内存快逐渐变小变碎，无法分配较大的内存区域，缺少对内存的合并与管理。
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // perform layout adjustments
        let (size, _algin) = LinkListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}


