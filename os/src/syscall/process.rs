//! Process management syscalls
use crate::{config::PAGE_SIZE, mm::{translated_byte_buffer, MapPermission, PTEFlags}, 
task::{change_program_brk, current_task, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER}, timer::get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;

    let len = core::mem::size_of::<TimeVal>();
    let mut buffers = translated_byte_buffer(current_user_token(), ts as *const u8, len, PTEFlags::empty()).unwrap();

    match buffers.len() {
        1 => {
            let buffer = buffers[0].as_mut_ptr() as *mut TimeVal;
            unsafe {
                *buffer = TimeVal {
                    sec,
                    usec,
                };
            }
            0
        }
        2 => {
            let buffer1 = buffers[0].as_mut_ptr() as *mut usize;
            let buffer2 = buffers[1].as_mut_ptr() as *mut usize;
            unsafe {
                // core::ptr::copy_nonoverlapping(&sec as *const _ as *const u8, buffer1, core::mem::size_of::<usize>());
                // core::ptr::copy_nonoverlapping(&usec as *const _ as *const u8, buffer2, core::mem::size_of::<usize>());
                core::ptr::write_volatile(buffer1, sec);
                core::ptr::write_volatile(buffer2, usec);
            }
            0
        }
        _ => {
            -1
        }
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let buffers = translated_byte_buffer(current_user_token(), id as *const u8, core::mem::size_of::<u8>(), PTEFlags::R);
            if buffers.is_none() {
                return  -1;
            }
            let id = buffers.unwrap()[0].as_ptr() as *const u8;
            let val = unsafe {
                core::ptr::read_volatile(id)
            };
            val as isize
        }
        1 => {
            let buffers = translated_byte_buffer(current_user_token(), id as *const u8, core::mem::size_of::<u8>(), PTEFlags::W);
            if buffers.is_none() {
                return -1;
            }
            let id = buffers.unwrap()[0].as_mut_ptr() as *mut u8;
            unsafe {
                core::ptr::write_volatile(id, data as u8);
            };
            0
        }
        2 => {
            let syscall_count = TASK_MANAGER.get_syscall_count(id);
            syscall_count as isize
        }
        _ => {
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");

    // 检查 start 是否按页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    // 检查 prot 是否有效
    if prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    }

    // 检查 len 是否为 0
    if len == 0 {
        return -1;
    }

    // 计算页数，向上取整
    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;

    // 检查虚存区间是否已经被映射
    
    let task = current_task().unwrap();
    let memory_set = &mut task.memory_set;
    for i in 0..page_count {
        let va = start + i * PAGE_SIZE;
        if memory_set.is_mapped(va / PAGE_SIZE) {
            return -1;
        }
    }

    // 设置权限
    let mut permission = MapPermission::empty();
    if prot & 0x1 != 0 {
        permission |= MapPermission::R; // 可读
    }
    if prot & 0x2 != 0 {
        permission |= MapPermission::W; // 可写
    }
    if prot & 0x4 != 0 {
        permission |= MapPermission::X; // 可执行
    }
    permission |= MapPermission::U;

    // 计算映射的结束地址
    let end = start + (page_count * PAGE_SIZE);
    // 使用 insert_framed_area 映射虚存区间
    memory_set.insert_framed_area(start.into(), end.into(), permission);

    0 // 成功返回 0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");

    // 检查 start 是否按页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    // 检查 len 是否为 0
    if len == 0 {
        return -1;
    }

    // 计算页数，向上取整
    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;

    // 检查虚存区间是否已经被映射
    let task = crate::task::current_task().unwrap();
    let memory_set = &mut task.memory_set;
    for i in 0..page_count {
        let va = start + i * PAGE_SIZE;
        if !memory_set.is_mapped(va / PAGE_SIZE) {
            return -1;
        }
    }

    // 计算映射的结束地址
    let end = start + (page_count * PAGE_SIZE);
    // 取消映射并释放物理页
    memory_set.remove_area_with_start(start.into(), end.into());

    0 // 成功返回 0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
