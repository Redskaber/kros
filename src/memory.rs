//! this crate is impl Kros Memory Manage.
//!     - Understanding Page Table  # 理解
//!     - used Recurse Page Table   # 使用
//!     - translate Some Addr       # 了解地址转化过程
//!     - FrameAllocator            # 尝试分配
//!     - BootInfoFrameAllocator    # 尝试分配

use bootloader::bootinfo::{
    BootInfo,         // 引导程序传递的内存映射
    MemoryMap,        //底层机器的物理内存区域Map。
    MemoryRegionType, // 内存区域类型
};
use x86_64::{
    structures::paging::{
        FrameAllocator,  // 帧分配器
        Mapper,          // 映射
        OffsetPageTable, // 偏移页表
        Page,            // 页
        PageTable,       // 页表
        PhysFrame,       // 物理帧
        Size4KiB,        // 4KiB
        Translate,       // 翻译
    },
    PhysAddr, VirtAddr,
};

#[allow(dead_code)]
/// 返回一个对活动的4级表的可变引用。
/// 这个函数是不安全的，因为调用者必须保证完整的物理内存在传递的
/// `physical_memory_offset`处被映射到虚拟内存。另外，这个函数
/// 必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
pub unsafe fn get_active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

#[allow(dead_code)]
#[repr(transparent)]
pub struct OffsetPageTableWarper(OffsetPageTable<'static>);
impl OffsetPageTableWarper {
    /// 使用提供的物理内存偏移量初始化一个新的OffsetPageTable。
    ///
    /// # 安全性
    ///
    /// 这是一个不安全的函数，因为它依赖于提供的物理内存偏移量来正确地执行映射。
    /// 函数调用者必须确保提供的物理内存偏移量正确地映射到了虚拟内存。
    /// 此外，这个函数必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
    ///
    /// # 参数
    ///
    /// * `physical_memory_offset` - 物理内存的偏移量。
    ///
    /// # Returns
    ///
    /// 一个新的OffsetPageTable实例。
    pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
        OffsetPageTable::new(
            get_active_level_4_table(physical_memory_offset),
            physical_memory_offset,
        )
    }
}

pub fn translate_some_addr(boot_info: &BootInfo) {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { OffsetPageTableWarper::init(phys_mem_offset) };

    // case example
    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt); //
        crate::println!("{:?} -> {:?}", virt, phys);
    }
}

// ####################### 分配页框 && FrameAllocator #########################
/// 创建一个新的映射
/// 创建一个给定页面的实例映射到框架 `0xb8000`。
///
/// # 参数
///
/// * `page` - 要映射的页面。
/// * `mapper` - 用于执行映射的页表。
/// * `frame_allocator` - 用于分配物理帧的帧分配器。
///
/// # 安全性
///
/// 这是一个不安全的函数，因为它依赖于提供的页表和帧分配器来正确地执行映射。
/// 此外，这个函数需要确保提供的页面对应的物理帧已经被正确地分配和初始化，否则会导致未定义行为。
///
/// # 错误处理
///
/// 如果映射过程失败，返回一个错误。
///
/// # Panics
///
/// 如果分配物理帧或执行映射过程失败，则函数会 panic。
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let vga_addr = PhysAddr::new(0xb8000);
    let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(vga_addr);
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };

    map_to_result.expect("map_tp failed!").flush();
}

/// 尝试创建一个FrameAllocator
/// 一个FrameAllocator, 从bootloader的内存中返回可用的frames(页帧).
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// 从传递的内存 map 中创建一个FrameAllocator
    ///
    /// 这个函数是不安全的，因为调用这必须保证传递的内存 map 是有效的
    /// 主要的要求是， 所有在被标记为 "可用" 的帧都是真正未被使用的
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// 在我们实现`FrameAllocator`特性之前，我们添加一个辅助方法，将内存映射转换为可用帧的迭代器。
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

#[allow(dead_code)]
/// 使用自定义好的BootInfoFrameAllocator 页帧分配器，实现对虚拟地址新建物理映射关系
///
/// # 参数
///
/// * `boot_info` - 引导信息指针
///
/// # 安全性
///
/// 这个函数并不安全，详情请参考函数内部的注释。
///
/// # Panics
///
/// 如果映射失败，该函数会 panic。
pub fn used_impl_frame_allocator(boot_info: &'static BootInfo) {
    // unused -> frame_allocator -> regions_range-> PhyFrame -> ok
    let used_addr = VirtAddr::new(0xabcdef);
    let page: Page<Size4KiB> = Page::containing_address(used_addr);

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { OffsetPageTableWarper::init(phys_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // f04e(`N`): VGA(background(u4)foreground(u4)character(u8))
    unsafe { page_ptr.offset(200).write_volatile(0x_f021_f077_f065_f04e) };
}

// 这个实现不是很理想，因为它在每次分配时都会重新创建`usable_frame`分配器。
// 最好的办法是直接将迭代器存储为一个结构域。
// 这样我们就不需要`nth`方法了，可以在每次分配时直接调用[`next`]。
// 这种方法的问题是，目前不可能将 “impl Trait “类型存储在一个结构字段中。当 [_named existential types_]完全实现时，它可能会在某一天发挥作用。
