//! 用户程序代码

use crate::simple_syscall::{SYSCALL_PRINT, SYSCALL_EXIT};

/// Hello World 程序的机器码
///
/// 这段代码相当于以下汇编:
/// ```asm
/// mov rax, 1                  ; SYSCALL_PRINT
/// lea rdi, [rip + message]    ; 消息地址
/// mov rsi, 13                 ; 消息长度
/// syscall                     ; 调用系统调用
///
/// mov rax, 2                  ; SYSCALL_EXIT
/// xor rdi, rdi                ; 退出码 0
/// syscall                     ; 调用系统调用
///
/// message:
/// db "Hello, World!", 0
/// ```
pub static HELLO_WORLD_PROGRAM: [u8; 48] = [
    // mov rax, 1 (SYSCALL_PRINT)
    0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,
    // lea rdi, [rip + 22] (指向消息)
    0x48, 0x8d, 0x3d, 0x16, 0x00, 0x00, 0x00,
    // mov rsi, 13 (消息长度)
    0x48, 0xc7, 0xc6, 0x0d, 0x00, 0x00, 0x00,
    // syscall
    0x0f, 0x05,

    // mov rax, 2 (SYSCALL_EXIT)
    0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00,
    // xor rdi, rdi (退出码 0)
    0x48, 0x31, 0xff,
    // syscall
    0x0f, 0x05,

    // "Hello, World!" 字符串
    0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21
];

/// 斐波那契数列程序的机器码
///
/// 这段代码相当于以下汇编:
/// ```asm
/// mov r12, 0                  ; 第一个数 (a)
/// mov r13, 1                  ; 第二个数 (b)
/// mov r14, 10                 ; 计算10个数
///
/// loop:
/// mov rax, 1                  ; SYSCALL_PRINT
/// lea rdi, [rip + message]    ; 消息地址
/// mov rsi, 4                  ; 消息长度
/// syscall                     ; 调用系统调用
///
/// mov rax, r13                ; 打印当前数 (b)
/// add r12, r13                ; a = a + b
/// xchg r12, r13               ; 交换 a 和 b
/// dec r14                     ; 计数器减1
/// jnz loop                    ; 如果计数器不为0，继续循环
///
/// mov rax, 2                  ; SYSCALL_EXIT
/// xor rdi, rdi                ; 退出码 0
/// syscall                     ; 调用系统调用
///
/// message:
/// db "Fib:", 0
/// ```
pub static FIBONACCI_PROGRAM: [u8; 74] = [
    // mov r12, 0 (第一个数)
    0x49, 0xc7, 0xc4, 0x00, 0x00, 0x00, 0x00,
    // mov r13, 1 (第二个数)
    0x49, 0xc7, 0xc5, 0x01, 0x00, 0x00, 0x00,
    // mov r14, 10 (计算10个数)
    0x49, 0xc7, 0xc6, 0x0a, 0x00, 0x00, 0x00,

    // loop:
    // mov rax, 1 (SYSCALL_PRINT)
    0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,
    // lea rdi, [rip + 40] (指向消息)
    0x48, 0x8d, 0x3d, 0x28, 0x00, 0x00, 0x00,
    // mov rsi, 4 (消息长度)
    0x48, 0xc7, 0xc6, 0x04, 0x00, 0x00, 0x00,
    // syscall
    0x0f, 0x05,

    // mov rax, r13 (当前数)
    0x4c, 0x89, 0xe8,
    // add r12, r13 (a = a + b)
    0x4d, 0x01, 0xec,
    // xchg r12, r13 (交换 a 和 b)
    0x4d, 0x87, 0xe5,
    // dec r14 (计数器减1)
    0x49, 0xff, 0xce,
    // jnz loop (如果计数器不为0，继续循环)
    0x75, 0xe0,

    // mov rax, 2 (SYSCALL_EXIT)
    0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00,
    // xor rdi, rdi (退出码 0)
    0x48, 0x31, 0xff,
    // syscall
    0x0f, 0x05,

    // "Fib:" 字符串
    0x46, 0x69, 0x62, 0x3a
];
