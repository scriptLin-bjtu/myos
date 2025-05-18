#![no_std] // 不使用标准库
#![no_main] // 不使用标准的main函数入口

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod vga_buffer;
mod serial;
mod keyboard;

// 定义内核入口点
entry_point!(kernel_main);

/// 内核入口点函数
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 清屏并打印欢迎信息
    vga_buffer::WRITER.lock().clear_screen();

    // 同时输出到VGA缓冲区和串口
    println!("Hello from x86_64 kernel!");
    serial_println!("Hello from x86_64 kernel!");

    println!("This is a bare metal OS running on QEMU");
    serial_println!("This is a bare metal OS running on QEMU");

    println!("Boot info: {:#?}", boot_info.memory_map.iter().count());
    serial_println!("Boot info: {:#?}", boot_info.memory_map.iter().count());

    // 初始化键盘驱动
    keyboard::init();

    println!("Pleace press any key to test keyboard input...");
    serial_println!("Pleace press any key to test keyboard input...");

    loop {
        // 处理键盘输入
        keyboard::handle_keyboard();

        // 使用非常短的延迟，不使用hlt指令
        for _ in 0..10 {
            core::hint::spin_loop();
        }
    }
}

/// 当panic发生时调用此函数
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 同时输出到VGA缓冲区和串口
    println!("{}", info);
    serial_println!("{}", info);

    loop {
        x86_64::instructions::hlt();
    }
}
