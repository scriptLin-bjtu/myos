#![no_std] // 不使用标准库
#![no_main] // 不使用标准的main函数入口
#![feature(abi_x86_interrupt)] // 启用x86中断ABI

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

mod vga_buffer;
mod serial;
mod gdt;
mod interrupts;
mod memory;
mod simple_batch; // 使用简化版批处理系统
mod context;      // 上下文切换
mod simple_syscall; // 简化版系统调用
mod user_programs; // 用户程序

// 定义内核入口点
entry_point!(kernel_main);



/// 内核入口点函数
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 清屏并打印欢迎信息
    vga_buffer::WRITER.lock().clear_screen();

    // 只使用串口输出，因为VGA输出可能有问题
    serial_println!("欢迎使用批处理操作系统！");

    // 简化测试，只进行基本初始化
    serial_println!("正在初始化GDT...");
    gdt::init();
    serial_println!("GDT初始化完成");

    // 暂停一下，让输出有时间显示
    for _ in 0..10000000 {
        // 空循环，用于延迟
    }

    serial_println!("正在初始化IDT...");
    interrupts::init();
    serial_println!("IDT初始化完成");

    // 初始化系统调用
    serial_println!("正在初始化系统调用...");
    simple_syscall::init_syscall();
    serial_println!("系统调用初始化完成");

    // 暂停一下，让输出有时间显示
    for _ in 0..10000000 {
        // 空循环，用于延迟
    }

    serial_println!("正在初始化内存管理...");

    // 使用固定的物理内存偏移量
    let phys_mem_offset = VirtAddr::new(0xffff_8000_0000_0000);
    serial_println!("物理内存偏移量: 0x{:x}", phys_mem_offset.as_u64());

    // 初始化内存管理
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    serial_println!("页表初始化完成");

    // 暂停一下，让输出有时间显示
    for _ in 0..10000000 {
        // 空循环，用于延迟
    }

    // 初始化帧分配器
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    serial_println!("帧分配器初始化完成");

    // 暂停一下，让输出有时间显示
    for _ in 0..10000000 {
        // 空循环，用于延迟
    }

    // 跳过堆初始化，直接使用简化版批处理系统
    serial_println!("正在初始化简化版批处理系统...");
    let mut batch_system = simple_batch::init();

    // 添加示例任务
    let hello_task = simple_batch::Task::new(
        "hello_world",
        simple_batch::TaskPriority::Normal,
        &user_programs::HELLO_WORLD_PROGRAM
    );

    let fibonacci_task = simple_batch::Task::new(
        "fibonacci",
        simple_batch::TaskPriority::High,
        &user_programs::FIBONACCI_PROGRAM
    );

    // 添加任务到批处理系统
    match batch_system.add_task(hello_task) {
        Ok(_) => serial_println!("Hello任务添加成功"),
        Err(e) => serial_println!("Hello任务添加失败: {}", e),
    }

    match batch_system.add_task(fibonacci_task) {
        Ok(_) => serial_println!("Fibonacci任务添加成功"),
        Err(e) => serial_println!("Fibonacci任务添加失败: {}", e),
    }

    // 运行批处理系统
    serial_println!("正在运行批处理系统...");
    batch_system.run();

    serial_println!("批处理系统执行完毕，系统将进入空闲状态");

    // 进入无限循环，防止CPU执行到未定义的内存区域
    loop {
        x86_64::instructions::hlt(); // 使用HLT指令让CPU休眠，直到下一个中断
    }
}

/// 当panic发生时调用此函数
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 同时输出到VGA缓冲区和串口
    serial_println!("内核崩溃: {}", info);

    loop {
        x86_64::instructions::hlt();
    }
}
