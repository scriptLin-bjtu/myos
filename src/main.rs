#![no_std] // 不使用标准库
#![no_main] // 不使用标准的main函数入口

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod vga_buffer;
mod serial;

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

    // 进入无限循环，防止CPU执行到未定义的内存区域
    loop {
        x86_64::instructions::hlt(); // 使用HLT指令让CPU休眠，直到下一个中断
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
