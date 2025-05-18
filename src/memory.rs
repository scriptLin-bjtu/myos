use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
        PageTableFlags,
    },
    PhysAddr, VirtAddr,
};
use bootloader::BootInfo;

/// 初始化一个新的OffsetPageTable
///
/// 这个函数是不安全的，因为调用者必须保证整个物理内存都被映射到了传入的
/// `physical_memory_offset`偏移处。此外，这个函数必须只被调用一次，以避免别名问题。
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// 返回一个对活动的第4级页表的可变引用
///
/// 这个函数是不安全的，因为调用者必须保证整个物理内存都被映射到了传入的
/// `physical_memory_offset`偏移处。此外，这个函数必须只被调用一次，以避免别名问题。
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // 不安全
}

/// 一个FrameAllocator，它从bootloader的内存映射中返回可用的帧
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// 从传入的内存映射创建一个FrameAllocator
    ///
    /// 这个函数是不安全的，因为调用者必须保证传入的内存映射是有效的。
    /// 主要的要求是，所有被标记为`USABLE`的帧都是真正可用的。
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// 返回内存映射中可用帧的迭代器
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // 获取内存映射中的可用区域
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // 将每个区域映射到它包含的地址范围
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // 将地址范围转换为帧起始地址的迭代器
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 创建`PhysFrame`对象
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// 为用户程序创建一个新的页表
pub fn create_user_page_table(
    boot_info: &'static BootInfo,
    kernel_page_table: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) -> OffsetPageTable<'static> {
    // 分配一个新的页表
    let frame = frame_allocator.allocate_frame().expect("no frames available");

    // 获取物理内存偏移
    // 在 bootloader 0.9.23 中，physical_memory_offset 可能不存在
    // 我们暂时使用一个固定的偏移量
    let phys_offset = VirtAddr::new(0xffff_8000_0000_0000);

    // 将新页表映射到虚拟地址空间
    let virt = phys_offset + frame.start_address().as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // 清空新页表
    unsafe {
        page_table_ptr.write_bytes(0, 1);
    }

    // 创建新的页表
    unsafe {
        OffsetPageTable::new(&mut *page_table_ptr, phys_offset)
    }
}

/// 为用户程序分配内存
pub fn allocate_user_memory(
    page_table: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
    start_address: VirtAddr,
    size: usize,
) -> Result<(), &'static str> {
    // 计算需要多少页
    let page_count = (size + 4095) / 4096; // 向上取整到页大小

    // 分配页
    for i in 0..page_count {
        let page = Page::<Size4KiB>::containing_address(start_address + (i * 4096) as u64);
        let frame = frame_allocator.allocate_frame().ok_or("no frames available")?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        unsafe {
            let result = page_table
                .map_to(page, frame, flags, frame_allocator);

            match result {
                Ok(mapper) => mapper.flush(),
                Err(_) => return Err("映射页面失败"),
            }
        }
    }

    Ok(())
}
