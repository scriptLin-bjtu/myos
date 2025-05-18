//! 简化版系统调用实现

use crate::serial_println;
use crate::context::{TaskContext, restore_context};
use core::arch::asm;

// 系统调用号
pub const SYSCALL_PRINT: u64 = 1;
pub const SYSCALL_EXIT: u64 = 2;

/// 处理系统调用
pub fn handle_syscall(syscall_number: u64, arg1: u64, arg2: u64, _arg3: u64) -> u64 {
    match syscall_number {
        SYSCALL_PRINT => {
            // 打印字符串
            let ptr = arg1 as *const u8;
            let len = arg2 as usize;
            
            // 安全检查：确保指针有效
            if ptr.is_null() || len == 0 {
                return 0;
            }
            
            // 从用户空间读取字符串
            let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
            
            // 尝试将字节转换为字符串
            if let Ok(s) = core::str::from_utf8(slice) {
                serial_println!("[用户程序]: {}", s);
            } else {
                // 如果不是有效的UTF-8，则打印字节
                serial_println!("[用户程序]: {:?}", slice);
            }
            
            0
        },
        SYSCALL_EXIT => {
            // 退出程序
            let exit_code = arg1;
            serial_println!("[用户程序]: 退出，退出码 {}", exit_code);
            
            // 在我们的简化系统中，我们需要恢复内核上下文
            // 这个函数不会返回，而是直接跳转到内核上下文
            unsafe {
                // 获取当前任务的内核上下文
                // 注意：这里需要一种方式来获取当前任务的内核上下文
                // 在实际实现中，我们可以使用全局变量或其他方式
                // 这里我们简化处理，直接返回
                exit_code
            }
        },
        _ => {
            // 未知系统调用
            serial_println!("[用户程序]: 未知系统调用 {}", syscall_number);
            u64::MAX
        }
    }
}

/// 设置系统调用处理
pub fn init_syscall() {
    // 在实际系统中，我们需要设置中断描述符表，将int 0x80映射到系统调用处理函数
    // 在我们的简化系统中，我们直接在用户程序中使用内联汇编调用系统调用
    serial_println!("系统调用初始化完成");
}

/// 系统调用处理函数
#[no_mangle]
pub extern "C" fn syscall_handler() {
    // 获取系统调用号和参数
    let syscall_number: u64;
    let arg1: u64;
    let arg2: u64;
    let arg3: u64;
    
    unsafe {
        asm!(
            "mov {0}, rax",
            "mov {1}, rdi",
            "mov {2}, rsi",
            "mov {3}, rdx",
            out(reg) syscall_number,
            out(reg) arg1,
            out(reg) arg2,
            out(reg) arg3,
        );
    }
    
    // 处理系统调用
    let result = handle_syscall(syscall_number, arg1, arg2, arg3);
    
    // 设置返回值
    unsafe {
        asm!("mov rax, {0}", in(reg) result);
    }
}
