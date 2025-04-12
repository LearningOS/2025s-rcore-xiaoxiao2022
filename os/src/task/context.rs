//! Implementation of [`TaskContext`]
use crate::trap::trap_return;

#[repr(C)]
/// task context structure containing some registers
pub struct TaskContext {
    /// Ret position after task switching
    ra: usize,
    /// Stack pointer
    sp: usize,
    /// s0-11 register, callee saved
    s: [usize; 12],
}

// task_cx 保存的是 ‌内核线程切换时的上下文‌（即内核态的寄存器状态），而非用户态寄存器状态。它的核心字段包括：
// ra（返回地址）：指向内核函数 trap_return，用于子进程首次调度时返回到用户态。
// sp（内核栈指针）：设置为子进程的内核栈顶（kernel_stack_top），用于读取 TrapContext。
// s0-s11（被调用者保存寄存器）：初始化为 0，‌因为子进程的内核线程尚未执行过任何函数调用‌。
impl TaskContext {
    /// Create a new empty task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    /// Create a new task context with a trap return addr and a kernel stack pointer
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
