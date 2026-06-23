use core::ptr::copy_nonoverlapping;

use log::debug;

use crate::{
    mm::user_check::UserCheck,
    process::{
        resource::{CpuSet, RLimit},
        PROCESS_MANAGER,
    },
    processor::{current_task, SumGuard},
    stack_trace,
    utils::error::{SyscallErr, SyscallRet},
};

pub fn sys_prlimit64(
    _pid: usize,
    resource: u32,
    new_limit: *const RLimit,
    old_limit: *mut RLimit,
) -> SyscallRet {
    stack_trace!();
    debug!("[sys_prlimit64] resource: {}", resource);
    let _sum_guard = SumGuard::new();
    if !old_limit.is_null() {
        UserCheck::new()
            .check_writable_slice(old_limit as *mut u8, core::mem::size_of::<RLimit>())?;
        let _sum_guard = SumGuard::new();
        let old_rlimit = RLimit::get_rlimit(resource);
        debug!("[sys_prlimit64] old limit: {:?}", old_rlimit);
        unsafe {
            *old_limit = old_rlimit;
        }
    }
    if new_limit.is_null() {
        debug!("[sys_prlimit64] new limit is null");
        return Ok(0);
    }
    UserCheck::new()
        .check_readable_slice(new_limit as *const u8, core::mem::size_of::<RLimit>())?;
    let _sum_guard = SumGuard::new();
    let new_rlimit = unsafe { &*new_limit };
    if new_rlimit.rlim_cur > new_rlimit.rlim_max {
        return Err(SyscallErr::EINVAL);
    }
    RLimit::set_rlimit(resource, new_rlimit)
}

pub fn sys_sched_getaffinity(pid: usize, cpusetsize: usize, mask: usize) -> SyscallRet {
    stack_trace!();
    debug_assert_eq!(cpusetsize, core::mem::size_of::<CpuSet>());
    let _sum_guard = SumGuard::new();
    if cpusetsize == 0 {
        return Err(SyscallErr::EINVAL);
    }
    UserCheck::new().check_writable_slice(mask as *mut u8, cpusetsize)?;
    let tid = if pid == 0 { current_task().tid() } else { pid };
    if let Some(proc) = PROCESS_MANAGER.get(tid) {
        if let Some(thread) = proc.inner_handler(|proc| {
            if let Some(thread) = proc.threads.get(&tid) {
                thread.upgrade()
            } else {
                None
            }
        }) {
            unsafe {
                let set = (*(thread.inner.get())).cpu_set;
                copy_nonoverlapping(
                    &set as *const CpuSet as *const u8,
                    mask as *mut u8,
                    core::cmp::min(cpusetsize, core::mem::size_of::<CpuSet>()),
                );
            }
            Ok(0)
        } else {
            log::info!(
                "[sys_sched_getaffinity] No such tid {} in pid {}",
                tid,
                proc.pid()
            );
            Err(SyscallErr::ESRCH)
        }
    } else {
        log::info!("[sys_sched_getaffinity] No such process, tid {}", tid);
        Err(SyscallErr::ESRCH)
    }
}

pub fn sys_sched_setaffinity(pid: usize, cpusetsize: usize, mask: usize) -> SyscallRet {
    stack_trace!();
    debug_assert_eq!(cpusetsize, core::mem::size_of::<CpuSet>());
    let _sum_guard = SumGuard::new();
    if cpusetsize < core::mem::size_of::<usize>() {
        return Err(SyscallErr::EINVAL);
    }
    UserCheck::new().check_readable_slice(mask as *const u8, cpusetsize)?;
    let tid = if pid == 0 { current_task().tid() } else { pid };
    if let Some(proc) = PROCESS_MANAGER.get(tid) {
        if let Some(thread) = proc.inner_handler(|proc| {
            if let Some(thread) = proc.threads.get(&tid) {
                thread.upgrade()
            } else {
                None
            }
        }) {
            unsafe {
                (*(thread.inner.get())).cpu_set = CpuSet {
                    set: *(mask as *const usize),
                    dummy: [0; 15],
                };
            }
            Ok(0)
        } else {
            debug!(
                "[sys_sched_setaffinity] No such tid {} in pid {}",
                tid,
                proc.pid()
            );
            Err(SyscallErr::ESRCH)
        }
    } else {
        debug!("[sys_sched_setaffinity] No such process");
        Err(SyscallErr::ESRCH)
    }
}

pub fn sys_sched_setscheduler() -> SyscallRet {
    stack_trace!();
    Ok(0)
}

pub fn sys_sched_getscheduler() -> SyscallRet {
    stack_trace!();
    Ok(0)
}

pub fn sys_sched_getparam() -> SyscallRet {
    stack_trace!();
    Ok(0)
}
