use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use crate::println;

// 双重故障处理栈的索引
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
// 系统调用栈的索引
pub const SYSCALL_IST_INDEX: u16 = 1;

// 内核栈大小
const STACK_SIZE: usize = 4096 * 5;

// 定义TSS结构
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // 为双重故障设置栈
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // 为系统调用设置栈
        tss.interrupt_stack_table[SYSCALL_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // 设置特权级0的栈指针（内核栈）
        tss.privilege_stack_table[0] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };
}

// 定义段选择子结构
pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

// 定义GDT结构
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // 添加内核代码段（Ring 0）
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // 添加内核数据段（Ring 0）
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        // 添加TSS段
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        // 添加用户代码段（Ring 3）
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        // 添加用户数据段（Ring 3）
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());

        (gdt, Selectors {
            code_selector,
            data_selector,
            tss_selector,
            user_code_selector,
            user_data_selector,
        })
    };
}

// 初始化GDT
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;

    // 加载GDT
    GDT.0.load();

    // 更新段寄存器
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        FS::set_reg(GDT.1.data_selector);
        GS::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }

    println!("GDT initialized");
}
