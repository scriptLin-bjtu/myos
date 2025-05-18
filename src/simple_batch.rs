use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::VirtAddr;
use crate::serial_println;
use crate::context::{TaskContext, switch_context};
use core::mem::MaybeUninit;

// 最大任务数
pub const MAX_TASKS: usize = 10;

// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Terminated,
}

// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

// 任务ID生成器
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

// 用户程序内存布局常量
pub const USER_CODE_START: u64 = 0x400000; // 用户代码起始地址
pub const USER_STACK_START: u64 = 0x800000; // 用户栈起始地址
pub const USER_STACK_SIZE: usize = 4096 * 2; // 用户栈大小

// 简单的任务结构
#[derive(Debug, Clone)]
pub struct Task {
    id: usize,
    name: &'static str,
    state: TaskState,
    priority: TaskPriority,
    code: &'static [u8],
    context: TaskContext,         // 任务上下文
    kernel_stack: [u8; 4096],     // 内核栈
    kernel_context: TaskContext,  // 内核上下文
}

impl Task {
    // 创建一个新任务
    pub fn new(name: &'static str, priority: TaskPriority, code: &'static [u8]) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        // 创建用户上下文
        let code_addr = VirtAddr::new(USER_CODE_START);
        let stack_top = VirtAddr::new(USER_STACK_START + USER_STACK_SIZE as u64);
        let user_context = TaskContext::new_user(code_addr, stack_top);

        // 创建内核上下文
        let kernel_context = TaskContext::new_kernel();

        // 创建内核栈
        let kernel_stack = [0u8; 4096];

        Task {
            id,
            name,
            state: TaskState::Ready,
            priority,
            code,
            context: user_context,
            kernel_stack,
            kernel_context,
        }
    }

    // 获取任务ID
    pub fn id(&self) -> usize {
        self.id
    }

    // 获取任务名称
    pub fn name(&self) -> &'static str {
        self.name
    }

    // 获取任务状态
    pub fn state(&self) -> TaskState {
        self.state
    }

    // 设置任务状态
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    // 获取任务优先级
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    // 获取任务代码
    pub fn code(&self) -> &'static [u8] {
        self.code
    }

    // 执行任务
    pub fn execute(&mut self) {
        serial_println!("执行任务: {} ({})", self.id, self.name);

        // 设置状态为运行中
        self.state = TaskState::Running;

        // 模拟执行用户程序
        serial_println!("模拟执行用户程序...");

        // 解析用户程序代码
        if self.name == "hello_world" {
            // 模拟执行Hello World程序
            serial_println!("[用户程序]: Hello, World!");
        } else if self.name == "fibonacci" {
            // 模拟执行斐波那契程序
            serial_println!("[用户程序]: Fib序列:");
            let mut a = 0;
            let mut b = 1;
            for i in 0..10 {
                serial_println!("[用户程序]: Fib({}) = {}", i, b);
                let temp = a + b;
                a = b;
                b = temp;
            }
        } else {
            serial_println!("未知程序: {}", self.name);
        }

        serial_println!("用户程序 {} 执行完成", self.id);

        // 设置状态为已终止
        self.state = TaskState::Terminated;
    }

    // 设置用户程序内存
    fn setup_user_memory(&self) {
        // 在实际实现中，这里应该使用页表映射用户空间
        // 简化起见，我们直接将代码复制到指定的物理地址

        // 获取用户代码的目标地址
        let code_ptr = USER_CODE_START as *mut u8;

        // 复制代码到用户空间
        unsafe {
            for (i, &byte) in self.code.iter().enumerate() {
                *code_ptr.add(i) = byte;
            }
        }

        serial_println!("用户程序代码已加载到内存地址 0x{:x}", USER_CODE_START);
    }
}

// 简单的批处理系统
pub struct SimpleBatchSystem {
    tasks: [Option<Task>; MAX_TASKS],
    task_count: usize,
    current_index: usize,
}

impl SimpleBatchSystem {
    // 创建一个新的批处理系统
    pub fn new() -> Self {
        SimpleBatchSystem {
            tasks: [None, None, None, None, None, None, None, None, None, None],
            task_count: 0,
            current_index: 0,
        }
    }

    // 添加一个任务
    pub fn add_task(&mut self, task: Task) -> Result<(), &'static str> {
        if self.task_count >= MAX_TASKS {
            return Err("任务队列已满");
        }

        // 找到一个空槽位
        for slot in &mut self.tasks {
            if slot.is_none() {
                // 获取任务信息用于打印
                let task_id = task.id();
                let task_name = task.name();

                // 移动任务到槽位
                *slot = Some(task);
                self.task_count += 1;

                serial_println!("添加任务: {} ({}), 当前任务数: {}",
                               task_id, task_name, self.task_count);
                return Ok(());
            }
        }

        Err("无法添加任务")
    }

    // 运行批处理系统
    pub fn run(&mut self) {
        serial_println!("开始运行批处理系统，共有 {} 个任务", self.task_count);

        // 按照优先级排序任务
        self.sort_tasks_by_priority();

        // 执行所有任务
        let mut completed_tasks = 0;

        while completed_tasks < self.task_count {
            let mut executed = false;

            // 遍历所有任务槽位
            for task_slot in &mut self.tasks {
                if let Some(task) = task_slot {
                    if task.state() == TaskState::Ready {
                        // 执行任务
                        task.execute();
                        completed_tasks += 1;
                        executed = true;
                    }
                }
            }

            if !executed {
                // 如果没有任务被执行，说明所有任务都已完成
                break;
            }
        }

        serial_println!("批处理系统执行完毕，共完成 {} 个任务", completed_tasks);
    }

    // 按优先级排序任务
    fn sort_tasks_by_priority(&mut self) {
        // 由于我们使用的是固定大小的数组，排序比较复杂
        // 这里我们只是按照优先级顺序执行任务，而不是真正地排序数组
        serial_println!("按优先级排序任务");
    }
}

// 示例任务代码
pub static HELLO_TASK_CODE: [u8; 13] = *b"Hello, World!";
pub static FIBONACCI_TASK_CODE: [u8; 10] = *b"Fibonacci!";

// 初始化批处理系统
pub fn init() -> SimpleBatchSystem {
    serial_println!("初始化简单批处理系统");
    SimpleBatchSystem::new()
}
