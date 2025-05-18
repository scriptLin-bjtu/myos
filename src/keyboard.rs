use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicU8, Ordering};
use crate::{println, serial_println};

/// 键盘控制器数据端口
const KEYBOARD_DATA_PORT: u16 = 0x60;

// 记录上一次处理的扫描码，避免重复处理
static LAST_SCANCODE: AtomicU8 = AtomicU8::new(0);
/// 键盘控制器状态/命令端口
const KEYBOARD_STATUS_PORT: u16 = 0x64;

/// 初始化键盘驱动
pub fn init() {
    println!("keyboard driver initialized");
    serial_println!("keyboard driver initialized");

    // 读取并打印键盘控制器的状态
    let mut status_port = Port::<u8>::new(KEYBOARD_STATUS_PORT);
    let status = unsafe { status_port.read() };
    println!("keyboard controller status: 0x{:02x}", status);
    serial_println!("keyboard controller status: 0x{:02x}", status);
}

/// 处理键盘输入
pub fn handle_keyboard() {
    // 从键盘状态端口读取状态
    let mut status_port = Port::<u8>::new(0x64);
    let status = unsafe { status_port.read() };
    // 只有当输出缓冲区有数据时才读取（状态寄存器的第0位为1）
    if status & 1 != 0 {
        // 从键盘数据端口读取
        let mut data_port = Port::<u8>::new(KEYBOARD_DATA_PORT);
        let scancode = unsafe { data_port.read() };
        // 忽略特定的扫描码（0xa0, 0xaa, 0xee, 0xfa 等是一些键盘控制器的状态码，不是实际按键）
        if scancode != 0 && scancode != 0xa0 && scancode != 0xaa && scancode != 0xee && scancode != 0xfa {
            // 检查是否与上一次处理的扫描码相同
            let last = LAST_SCANCODE.load(Ordering::Relaxed);
            // 只处理不同的扫描码，避免重复处理
            if scancode != last {
                // 更新上一次处理的扫描码
                LAST_SCANCODE.store(scancode, Ordering::Relaxed);
                // 打印原始扫描码
                println!("read scancode: 0x{:02x}", scancode);
                serial_println!("read scancode: 0x{:02x}", scancode);
                // 将扫描码转换为字符
                if let Some(key) = scancode_to_char(scancode) {
                    println!("key: {}", key);
                    serial_println!("key: {}", key);
                }
            }
        }
    }
}

/// 将扫描码转换为字符
///
/// 这是一个简单的扫描码到ASCII字符的映射
/// 只处理基本的按键按下事件（不处理释放事件）
/// 扫描码集合1（Set 1）映射表
pub fn scancode_to_char(scancode: u8) -> Option<char> {
    // 只处理按键按下事件（不处理释放事件，释放事件扫描码通常是0x80+按下扫描码）
    if scancode & 0x80 != 0 {
        return None;
    }

    // 扫描码到ASCII字符的映射
    match scancode {
        0x01 => Some(27 as char),  // ESC
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),
        0x0C => Some('-'),
        0x0D => Some('='),
        0x0E => Some('\u{0008}'),  // Backspace
        0x0F => Some('\t'),        // Tab
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        0x1C => Some('\n'),        // Enter
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),
        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x39 => Some(' '),         // Space
        _ => None,
    }
}
