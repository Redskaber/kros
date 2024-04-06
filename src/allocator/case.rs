use core::{
    alloc::{GlobalAlloc, Layout}, 
    mem,
    ptr,
};

use super::{align_up, AddrRegion, Allocator, Locked};

#[allow(dead_code)]
struct CaseNode {
    size: usize,
    next: Option<&'static mut CaseNode>,
}

impl CaseNode {
    /// 创建一个知道存储大小的链表节点, 使用const 是因为新建的CaseNode 节点是不能更改指向的
    const fn new(size: usize) -> Self {
        CaseNode{size, next: None}
    }
    /// 获取这个链表节点的起始地址, 分配器分配的链表节点地址就是开始地址，返回指向自身的指针解引用的地址即可
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    /// 获取这个链表节点的起始地址并通过运算加上size，获取到末尾地址
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

#[allow(dead_code)]
pub struct CaseAllocator {
    head: CaseNode,
}

impl CaseAllocator {
    /// 创建一个分配器链表，头节点为空节点的链表结构
    pub const fn new() -> Self {
        Self {head: CaseNode::new(0)}
    }
    /// 初始化内存分配器,通过使用调用unsafe 函数，创建一个大的分配器空间
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }
}

unsafe impl Allocator<CaseNode> for CaseAllocator {

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // 校验 地址对齐后的地址是否与给定的地址一致
        assert_eq!(addr, align_up(addr, mem::align_of::<CaseNode>()));
        // 校验 分配的地址内存大小是否能够容纳所需分配的数据大小
        assert!(size >= mem::size_of::<CaseNode>());

        let mut node = CaseNode::new(size);
        let head_next = self.head.next.take();
        node.next = head_next;
        
        // 获取 创建节点的起始地址
        let node_ptr = addr as *mut CaseNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    fn find_free_region(&mut self, size: usize, align: usize) -> Option<AddrRegion<CaseNode>> {
        let mut current = &mut self.head;

        while let Some(ref mut region) = current.next {
            if let Ok(alloc_addr) = Self::alloc_from_region(&region, size, align){
                let addr_region = current.next.take().unwrap();
                let region_next = addr_region.next.take();

                current.next = region_next;

                return Some(AddrRegion::new(addr_region, alloc_addr));
            }else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }

    fn alloc_from_region(region: &CaseNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<CaseNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<CaseNode>())
            .expect("adjusting alignment failed!")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<CaseNode>());

        (size, layout.align())
    }
}


/// impl GlobalAlloc for CaseAllocator
unsafe impl GlobalAlloc for Locked<CaseAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 获取分配的大小和对齐方式
        let (size, align) = CaseAllocator::size_align(layout);
        // 获取分配器的锁
        let mut allocator = self.lock();

        // 寻找到一个合适的空间，并分配
        if let Some(AddrRegion{addr_region, alloc_addr}) = allocator.find_free_region(size, align) {
            let alloc_end = alloc_addr.checked_add(size).expect("find_free_region overflow!");

            // 获取剩余空间, 如果存在，则将剩余空间添加到链表中
            let excess_size = addr_region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            };

            alloc_addr as *mut u8
        } else {
            ptr::null_mut()
        }   

    }
    /// 释放内存, 将释放的内存添加到链表中，
    /// 存在一个问题，就是过多的通过分配内存还会将heap 中的内存快逐渐变小变碎，无法分配较大的内存区域，缺少对内存的合并与管理。
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _align) = CaseAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size);
    }
}