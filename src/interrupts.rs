use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::println;
use crate::gdt;
use crate::serial_println;
use crate::simple_syscall;

// 系统调用中断号
pub const SYSCALL_INTERRUPT_INDEX: u8 = 0x80;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // 设置断点异常处理
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // 设置双重故障处理，使用专用栈
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // 设置页错误处理
        idt.page_fault.set_handler_fn(page_fault_handler);

        // 设置系统调用处理
        unsafe {
            idt[SYSCALL_INTERRUPT_INDEX as usize]
                .set_handler_fn(syscall_handler)
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3) // 允许用户态调用
                .set_stack_index(gdt::SYSCALL_IST_INDEX);
        }

        idt
    };
}

// 初始化IDT
pub fn init() {
    IDT.load();
    println!("IDT initialized");
}

// 断点异常处理函数
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// 双重故障处理函数
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// 页错误处理函数
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode)
{
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);

    loop {
        x86_64::instructions::hlt();
    }
}

// 系统调用处理函数
extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    // 从寄存器中获取系统调用号和参数
    let syscall_number: u64;
    let arg1: u64;
    let arg2: u64;
    let arg3: u64;

    unsafe {
        // 使用内联汇编读取寄存器值
        core::arch::asm!(
            "nop", // 空操作，只是为了读取寄存器
            out("rax") syscall_number,
            out("rdi") arg1,
            out("rsi") arg2,
            out("rdx") arg3,
        );
    }

    // 使用我们的简化系统调用处理函数
    let result = simple_syscall::handle_syscall(syscall_number, arg1, arg2, arg3);

    // 将结果存储在RAX中返回给用户程序
    unsafe {
        core::arch::asm!(
            "mov rax, {}",
            in(reg) result,
            options(nostack),
        );
    }

    serial_println!("系统调用 {} 处理完成，结果: {}", syscall_number, result);
}
