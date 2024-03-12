//! this crate is impl Kros Memory Manage.
//!     - Understanding Page Table
//!     - create some example
//!     - used Recurse Page Table
//!     ...


mod test_space {
    use crate::println;
    use bootloader::BootInfo;   // addr info
    use x86_64::{
            VirtAddr,
            PhysAddr,
            structures::paging::{
            Page, 
            Mapper,             // 多级页表之间的映射关系与偏移
            PageTable,
            RecursivePageTable, // 递归页表
            OffsetPageTable,    // 使用lib中 `OffsetPageTable` => 巨大的页面
            Translate, 
        }
    };


    #[allow(dead_code)]
    fn base_case() {
        let addr: usize = 0xabcd;
        
        // page index range(0..=511) 2^9 0b1 1111 1111
        let r: usize = 0o777; 
        // sign 
        let sign: usize = 0o177777 << 48;    // 65535 => 16个1 （64-12-9*4） -> left moved 48 0b1111 1111 1111 1111
        println!("addr: {addr:#b},\nr: {r},\nsign: {sign:#b}");

        // 检索我们要翻译的地址的页表索引
        let l4_idx = (addr >> 39) & 0o777; // level 4  2^9 索引  0
        let l3_idx = (addr >> 30) & 0o777; // level 3  2^9 索引  0 
        let l2_idx = (addr >> 21) & 0o777; // level 2  2^9 索引  0 
        let l1_idx = (addr >> 12) & 0o777; // level 1  2^9 索引 10
        let page_offset = addr & 0o7777;   // page_of  2^12  3021

        println!("l4_idx: {l4_idx}, l3_idx: {l3_idx}, l2_idx: {l2_idx}, l1_idx: {l1_idx}, page_offset: {page_offset}");

        // 计算页表的地址
        // r: 0b1 1111 1111 9位
        // sign: 0b1111 1111 1111 1111 16位
        //
        // [ , )
        //             (64)      sgin      (48)
        // 16+48= 64 => |0b11111111 11111111| 00000000 00000000 00000000 00000000 00000000 00000000  (r<< 48)
        //             (64)                (48)  l4   (39)
        // 9+39 = 48 => 0b00000000 00000000 |11111111 1|0000000 00000000 00000000 00000000 00000000  (r<< 39)
        //             (64)                          (39)  l3   (30)
        // 9+30 = 39 => 0b00000000 00000000 00000000 0|1111111 11|000000 00000000 00000000 00000000  (r<< 30)
        //             (64)                                    (30)   l2  (21)
        // 9+21 = 30 => 0b00000000 00000000 00000000 00000000 00|111111 111|00000 00000000 00000000  (r<< 21)
        //             (64)                                              (21)   l1  (12)
        // 9+12 = 21 => 0b00000000 00000000 00000000 00000000 00000000 000|11111 1111|0000 00000000  (r<< 12)
        //             (64)                                                         (21)  offset (12)
        // 12+0 = 12 => 0b00000000 00000000 00000000 00000000 00000000 00000000 0000|1111 11111111|  (offset)


        let level_4_table_addr =
            sign | (r << 39) | (r << 30) | (r << 21) | (r << 12);  
        let level_3_table_addr =
            sign | (r << 39) | (r << 30) | (r << 21) | (l4_idx << 12);
        let level_2_table_addr =
            sign | (r << 39) | (r << 30) | (l4_idx << 21) | (l3_idx << 12);
        let level_1_table_addr =
            sign | (r << 39) | (l4_idx << 30) | (l3_idx << 21) | (l2_idx << 12);

        println!("level_4_table_addr: {level_4_table_addr},\nlevel_3_table_addr: {level_3_table_addr},\nlevel_2_table_addr: {level_2_table_addr},\nlevel_1_table_addr: {level_1_table_addr}");
    }


    #[allow(dead_code)]
    fn table_case(){
        // 从第4级地址创建一个RecursivePageTable实例。
        // let level_4_table_addr: u64 = 0x0000_0000_1000; // level 1 address -> err: NotResursice
        let level_4_table_addr: u64 = 0x0000_0000_0fff; // used 2^12-> [0, 4096)

        let level_4_table_ptr = level_4_table_addr as *mut PageTable;
        let recursive_page_table = unsafe {
        let level_4_table = &mut *level_4_table_ptr;
        RecursivePageTable::new(level_4_table).unwrap()
        };

        // 检索给定虚拟地址的物理地址
        let addr: u64 = 0xabcd;
        let addr = x86_64::VirtAddr::new(addr);
        let page: Page = Page::containing_address(addr);

        // 进行翻译
        let frame = recursive_page_table.translate_page(page);
        let re_phy = frame.map(|frame| frame.start_address() + u64::from(addr.page_offset()));
        println!("re_phy: {:#?}", re_phy.unwrap());
    }


    #[allow(dead_code)]
    /// ###################### 返回一个对活动的4级表的可变引用。#########################
    ///
    /// 这个函数是不安全的，因为调用者必须保证完整的物理内存在传递的 
    /// `physical_memory_offset`处被映射到虚拟内存。另外，这个函数
    /// 必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
    pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
        use x86_64::registers::control::Cr3;

        let (level_4_table_frame, _) = Cr3::read();

        let phys = level_4_table_frame.start_address();
        let virt = physical_memory_offset + phys.as_u64();
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

        &mut *page_table_ptr // unsafe
    }


    #[allow(dead_code)]
    pub fn print_level_4_table(boot_info: &BootInfo){
        let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
        let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

        for (i, entry) in l4_table.iter().enumerate() {
            if !entry.is_unused() {
                crate::println!("L4 Entry {}: {:?}", i, entry);

                // get the physical address from the entry and convet it.
                let phys = entry.frame().unwrap()
                    .start_address();
                let vrit = VirtAddr::new(boot_info.physical_memory_offset + phys.as_u64());
                let ptr: *mut PageTable = vrit.as_mut_ptr();
                let l3_table: &PageTable = unsafe {
                    &*ptr
                };

                // print non-emtry entries of the level 3 table
                for (i, entry) in l3_table.iter().enumerate() {
                    if !entry.is_unused() {
                        crate::println!("  L3 Entry {}: {:?}", i, entry);
                    }
                }
            }
        }
    }


    #[allow(dead_code)]
    /// ######################### Transalte 翻译地址 ###############################
    /// 将给定的虚拟地址转换为映射的物理地址，如果地址没有被映射，则为`None'。
    ///
    /// 这个函数是不安全的，因为调用者必须保证完整的物理内存在传递的`physical_memory_offset`处被映射到虚拟内存。
    pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
        _translate_addr_inner(addr, physical_memory_offset)
    }


    /// 由 `translate_addr`调用的私有函数。
    ///
    /// 这个函数是安全的，可以限制`unsafe`的范围，
    /// 因为Rust将不安全函数的整个主体视为不安全块。
    /// 这个函数只能通过`unsafe fn`从这个模块的外部到达。
    fn _translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
        use x86_64::structures::paging::page_table::FrameError;     // error
        use x86_64::registers::control::Cr3;    // lv4

        // 从CR3 寄存器中读取活动的l4 frame
        let (level_4_table_frame, _) = Cr3::read();

        // 构造遍历对象: 每一级页表的索引
        let table_indexs = [
            addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
        ];
        // 创建遍历临时变量：存储页表虚拟地址
        let mut frame = level_4_table_frame;

        // 遍历多级页表（4，3，2，1）
        for &index in &table_indexs {
            // 将该框架转换为页表参考
            let virt = physical_memory_offset + frame.start_address().as_u64();
            let table_ptr: *const PageTable = virt.as_ptr();
            let table = unsafe {&*table_ptr};   // l4, l3, l2, l1

            // 读取页表条目并更新‘frame’
            let entry = &table[index];  //  l4_tra,...
            frame = match entry.frame() {   // l3, l2, l1
                Ok(frame) => frame,
                Err(FrameError::FrameNotPresent) => return None,
                Err(FrameError::HugeFrame) => panic!("huge pages not supported!"), // buge page panic          
            };
        }

        // 通过添加页面偏移量来计算物理地址 => phy: 遍历表得帧号 + 页内偏移
        Some(frame.start_address() + u64::from(addr.page_offset()))
    }


    // handler example: 
    // x86_64::structures::paging::mapper::mapped_page_table::MappedPageTable
    // fn translate(&self, addr: VirtAddr) -> TranslateResult  


    #[allow(dead_code)]
    pub fn translate_some_addr(boot_info: &BootInfo) {
        let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

        let addresses = [
            // the identity-mapped vga buffer page
            0xb8000,
            // some code page
            0x201008,
            // some stack page
            0x0100_0020_1a10,
            // virtual address mapped to physical address 0
            boot_info.physical_memory_offset,
        ];

        for &address in &addresses {
            let virt = VirtAddr::new(address);
            let phys = unsafe { translate_addr(virt, phys_mem_offset) };
            crate::println!("{:?} -> {:?}", virt, phys);
        }
        // HugeFrame handler error！ -> used x86_64 OffsetPageTable
    }


    #[allow(dead_code)]
    /// ################ 使用lib中 `OffsetPageTable` => 巨大的页面 ###################
    /// 初始化一个新的OffsetPageTable。
    ///
    /// 这个函数是不安全的，因为调用者必须保证完整的物理内存在
    /// 传递的`physical_memory_offset`处被映射到虚拟内存。另
    /// 外，这个函数必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
    pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }

    
    #[allow(dead_code)]
    /// used Translate::translate_addr
    pub fn translate_some_addr_from_lib(boot_info: &BootInfo) {
        let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

        // new: mapper.
        // A Mapper implementation that requires that the complete physically memory is mapped at some
        // offset in the virtual address space.
        let mapper = unsafe {
            crate::memory::test_space::init(phys_mem_offset)
        };

        // case example
        let addresses = [
            // the identity-mapped vga buffer page
            0xb8000,
            // some code page
            0x201008,
            // some stack page
            0x0100_0020_1a10,
            // virtual address mapped to physical address 0
            boot_info.physical_memory_offset,
        ];

        for &address in &addresses {
            let virt = VirtAddr::new(address);
            let phys = mapper.translate_addr(virt);  // 
            crate::println!("{:?} -> {:?}", virt, phys);
        }
    }

}




















