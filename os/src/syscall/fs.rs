//! File and filesystem-related syscalls
use core::mem;

use alloc::vec::Vec;

use crate::fs::{open_file, OSInode, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!("kernel:pid[{}] sys_fstat", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }
    let file = inner.fd_table[fd].clone().unwrap();
    drop(inner);
    if let Some(os_inode) = file.as_any().downcast_ref::<OSInode>() {
        let stat = os_inode.fstat();

        // 将 `Stat` 写入用户空间的 `buffers`
        let len = core::mem::size_of::<Stat>();
        let mut buffers = translated_byte_buffer(current_user_token(), st as *const u8, len);
       write_stat_to_buffers(&stat, &mut buffers);
        0
    } else {
        -1   // 类型不匹配
    }
}
fn write_stat_to_buffers(stat: &Stat, buffers: &mut Vec<&mut [u8]>) {
    // 将 `Stat` 转换为字节数组
    let stat_bytes = unsafe {
        core::slice::from_raw_parts(
            (stat as *const Stat) as *const u8,
            mem::size_of::<Stat>(),
        )
    };

    // 将字节数组写入 `buffers`
    let mut offset = 0;
    for buffer in buffers.iter_mut() {
        let remaining = stat_bytes.len() - offset;
        if remaining == 0 {
            break;
        }
        let to_copy = remaining.min(buffer.len());
        buffer[..to_copy].copy_from_slice(&stat_bytes[offset..offset + to_copy]);
        offset += to_copy;
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_linkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let old_name = translated_str(token, old_name);
    let new_name = translated_str(token, new_name);
    if old_name == new_name {
        return -1;
    }
    if let Some(old_inode) = open_file(&old_name, OpenFlags::RDONLY) {
        old_inode.link(&new_name, &old_name);
        0
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_unlinkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let name = translated_str(token, name);
    if let Some(inode) = open_file(&name, OpenFlags::RDONLY) {
        inode.unlink(&name);
        0
    } else {
        -1
    }
}
