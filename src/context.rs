//! 上下文切换相关代码

use core::arch::asm;
use x86_64::{VirtAddr, registers::rflags::RFlags};

/// 任务上下文结构体，保存所有通用寄存器和特殊寄存器
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    // 通用寄存器
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // 特殊寄存器
    pub rip: u64,  // 程序计数器
    pub rsp: u64,  // 栈指针
    pub rflags: u64, // 标志寄存器

    // 段寄存器
    pub cs: u64,
    pub ss: u64,
}

impl TaskContext {
    /// 创建一个新的内核上下文
    pub fn new_kernel() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rsp: 0,
            rflags: RFlags::INTERRUPT_FLAG.bits(), // 启用中断
            cs: 0x8, // 内核代码段
            ss: 0x10, // 内核数据段
        }
    }

    /// 创建一个新的用户上下文
    pub fn new_user(entry_point: VirtAddr, stack_top: VirtAddr) -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: entry_point.as_u64(),
            rsp: stack_top.as_u64(),
            rflags: RFlags::INTERRUPT_FLAG.bits(), // 启用中断
            cs: 0x1b, // 用户代码段 (0x18 | 3)
            ss: 0x23, // 用户数据段 (0x20 | 3)
        }
    }
}

/// 保存当前上下文到指定的TaskContext结构体中
#[inline(never)]
pub unsafe fn save_context(context: &mut TaskContext) {
    asm!(
        "mov [{0} + 0x00], rax",
        "mov [{0} + 0x08], rbx",
        "mov [{0} + 0x10], rcx",
        "mov [{0} + 0x18], rdx",
        "mov [{0} + 0x20], rsi",
        "mov [{0} + 0x28], rdi",
        "mov [{0} + 0x30], rbp",
        "mov [{0} + 0x38], r8",
        "mov [{0} + 0x40], r9",
        "mov [{0} + 0x48], r10",
        "mov [{0} + 0x50], r11",
        "mov [{0} + 0x58], r12",
        "mov [{0} + 0x60], r13",
        "mov [{0} + 0x68], r14",
        "mov [{0} + 0x70], r15",
        // RIP, RSP和RFLAGS需要特殊处理
        in(reg) context as *mut TaskContext,
        options(nostack),
    );

    // 保存RSP (当前栈指针)
    let rsp: u64;
    asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack));
    context.rsp = rsp;

    // 保存RFLAGS
    let rflags: u64;
    asm!("pushfq; pop {}", out(reg) rflags, options(nomem, nostack));
    context.rflags = rflags;

    // RIP无法直接保存，将在switch_context中处理
}

/// 从指定的TaskContext结构体恢复上下文
#[inline(never)]
pub unsafe fn restore_context(context: &TaskContext) -> ! {
    // 先恢复通用寄存器
    asm!(
        "mov rax, [{0} + 0x00]",
        "mov rbx, [{0} + 0x08]",
        "mov rcx, [{0} + 0x10]",
        "mov rdx, [{0} + 0x18]",
        "mov rsi, [{0} + 0x20]",
        "mov rdi, [{0} + 0x28]",
        "mov rbp, [{0} + 0x30]",
        "mov r8, [{0} + 0x38]",
        "mov r9, [{0} + 0x40]",
        "mov r10, [{0} + 0x48]",
        "mov r11, [{0} + 0x50]",
        "mov r12, [{0} + 0x58]",
        "mov r13, [{0} + 0x60]",
        "mov r14, [{0} + 0x68]",
        "mov r15, [{0} + 0x70]",
        in(reg) context as *const TaskContext,
        options(nostack),
    );

    // 准备IRET所需的数据
    let rip = context.rip;
    let cs = context.cs;
    let rflags = context.rflags;
    let rsp = context.rsp;
    let ss = context.ss;

    // 使用IRET指令切换到目标上下文
    // IRET会从栈上弹出RIP, CS, RFLAGS, RSP, SS
    asm!(
        // 构建IRET栈帧
        "push {ss}",    // SS
        "push {rsp}",   // RSP
        "push {rflags}", // RFLAGS
        "push {cs}",    // CS
        "push {rip}",   // RIP
        "iretq",        // 执行IRET指令
        ss = in(reg) ss,
        rsp = in(reg) rsp,
        rflags = in(reg) rflags,
        cs = in(reg) cs,
        rip = in(reg) rip,
        options(noreturn),
    );
}

/// 切换上下文，从当前上下文切换到新上下文
pub unsafe fn switch_context(old_context: &mut TaskContext, new_context: &TaskContext) -> ! {
    // 保存当前RIP到old_context
    // 我们使用标签和LEA指令来获取返回地址
    asm!(
        "lea rax, [rip + 3f]", // 获取标签3处的地址
        "mov [{0} + 0x78], rax", // 保存到old_context.rip
        "jmp 2f", // 跳转到标签2
        "3:", // 标签3 - 这是恢复old_context后会返回的位置
        "ret", // 返回到调用者
        "2:", // 标签2
        in(reg) old_context as *mut TaskContext,
        options(nostack),
    );

    // 保存当前上下文
    save_context(old_context);

    // 恢复新上下文
    restore_context(new_context);
}
