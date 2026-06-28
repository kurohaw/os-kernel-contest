use core::time::Duration;

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use log::{debug, info, trace};

use crate::{
    fs::{FdInfo, File, FileMeta, InodeMode, OpenFlags},
    mm::user_check::UserCheck,
    process::{thread::spawn_kernel_thread, PROCESS_MANAGER},
    processor::{current_process, current_task, SumGuard},
    signal::SIGALRM,
    stack_trace,
    sync::mutex::SpinNoIrqLock,
    sync::Event,
    timer::{
        current_time_duration, current_time_ms,
        ffi::current_time_spec,
        ffi::TimeVal,
        ffi::Tms,
        ffi::{ITimerval, TimeSpec},
        realtime_offset, set_realtime_offset,
        timed_task::TimedTaskFuture,
        timeout_task::ksleep,
        CLOCK_MANAGER, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, TIMER_ABSTIME,
    },
    utils::{
        async_utils::{Select2Futures, SelectOutput},
        error::{AsyscallRet, GeneralRet, SyscallErr, SyscallRet},
    },
};

const EFD_SEMAPHORE: u32 = 1;
const EFD_NONBLOCK: u32 = 1 << 11;
const EFD_CLOEXEC: u32 = 1 << 19;
const TFD_NONBLOCK: u32 = 1 << 11;
const TFD_CLOEXEC: u32 = 1 << 19;
const TFD_TIMER_ABSTIME: u32 = 1;
const TFD_TIMER_CANCEL_ON_SET: u32 = 2;

static TIMERFD_TABLE: SpinNoIrqLock<BTreeMap<usize, Arc<TimerFd>>> =
    SpinNoIrqLock::new(BTreeMap::new());

struct EventFd {
    counter: SpinNoIrqLock<u64>,
    semaphore: bool,
    meta: FileMeta,
}

impl EventFd {
    fn new(initval: u64, semaphore: bool) -> Self {
        Self {
            counter: SpinNoIrqLock::new(initval),
            semaphore,
            meta: FileMeta::new(InodeMode::FileREG),
        }
    }
}

impl File for EventFd {
    fn metadata(&self) -> &FileMeta {
        &self.meta
    }

    fn read<'a>(&'a self, buf: &'a mut [u8], flags: OpenFlags) -> AsyscallRet {
        Box::pin(async move {
            if buf.len() < core::mem::size_of::<u64>() {
                return Err(SyscallErr::EINVAL);
            }
            loop {
                let value = {
                    let mut counter = self.counter.lock();
                    if *counter == 0 {
                        None
                    } else if self.semaphore {
                        *counter -= 1;
                        Some(1)
                    } else {
                        let value = *counter;
                        *counter = 0;
                        Some(value)
                    }
                };
                if let Some(value) = value {
                    buf[..8].copy_from_slice(&value.to_ne_bytes());
                    return Ok(8);
                }
                if flags.contains(OpenFlags::NONBLOCK) {
                    return Err(SyscallErr::EAGAIN);
                }
                ksleep(Duration::from_millis(1)).await;
            }
        })
    }

    fn write<'a>(&'a self, buf: &'a [u8], _flags: OpenFlags) -> AsyscallRet {
        Box::pin(async move {
            if buf.len() < core::mem::size_of::<u64>() {
                return Err(SyscallErr::EINVAL);
            }
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(&buf[..8]);
            let value = u64::from_ne_bytes(bytes);
            if value == u64::MAX {
                return Err(SyscallErr::EINVAL);
            }
            let mut counter = self.counter.lock();
            *counter = counter.saturating_add(value);
            Ok(8)
        })
    }

    fn pollin(&self, _waker: Option<core::task::Waker>) -> GeneralRet<bool> {
        Ok(*self.counter.lock() > 0)
    }
}

pub fn sys_eventfd2(initval: u64, flags: u32) -> SyscallRet {
    stack_trace!();
    let allowed = EFD_SEMAPHORE | EFD_NONBLOCK | EFD_CLOEXEC;
    if flags & !allowed != 0 {
        return Err(SyscallErr::EINVAL);
    }
    let mut open_flags = OpenFlags::RDWR;
    if flags & EFD_NONBLOCK != 0 {
        open_flags |= OpenFlags::NONBLOCK;
    }
    if flags & EFD_CLOEXEC != 0 {
        open_flags |= OpenFlags::CLOEXEC;
    }
    let file = Arc::new(EventFd::new(initval, flags & EFD_SEMAPHORE != 0));
    current_process().inner_handler(move |proc| {
        let fd = proc.fd_table.alloc_fd()?;
        proc.fd_table.put(fd, FdInfo::new(file, open_flags));
        Ok(fd)
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ITimerSpec {
    it_interval: TimeSpec,
    it_value: TimeSpec,
}

struct TimerFdInner {
    clock_id: usize,
    interval: Duration,
    next_expiration: Option<Duration>,
}

struct TimerFd {
    inner: SpinNoIrqLock<TimerFdInner>,
    meta: FileMeta,
}

impl TimerFd {
    fn new(clock_id: usize) -> Self {
        Self {
            inner: SpinNoIrqLock::new(TimerFdInner {
                clock_id,
                interval: Duration::ZERO,
                next_expiration: None,
            }),
            meta: FileMeta::new(InodeMode::FileREG),
        }
    }

    fn now_for_clock(clock_id: usize) -> Duration {
        if clock_id == CLOCK_REALTIME {
            current_time_duration() + realtime_offset()
        } else {
            current_time_duration()
        }
    }

    fn validate_timespec(time: TimeSpec) -> GeneralRet<Duration> {
        if time.sec > isize::MAX as usize || time.nsec >= 1_000_000_000 {
            return Err(SyscallErr::EINVAL);
        }
        Ok(time.into())
    }

    fn snapshot(&self) -> ITimerSpec {
        let inner = self.inner.lock();
        let remaining = inner
            .next_expiration
            .map(|deadline| deadline.saturating_sub(Self::now_for_clock(inner.clock_id)))
            .unwrap_or(Duration::ZERO);
        ITimerSpec {
            it_interval: inner.interval.into(),
            it_value: remaining.into(),
        }
    }

    fn expired_count(inner: &mut TimerFdInner) -> u64 {
        let Some(deadline) = inner.next_expiration else {
            return 0;
        };
        let now = Self::now_for_clock(inner.clock_id);
        if now < deadline {
            return 0;
        }
        if inner.interval.is_zero() {
            inner.next_expiration = None;
            return 1;
        }
        let elapsed = now.saturating_sub(deadline).as_nanos();
        let step = inner.interval.as_nanos();
        let count = 1 + (elapsed / step) as u64;
        let advance = step.saturating_mul(count as u128).min(u64::MAX as u128);
        inner.next_expiration = Some(deadline + Duration::from_nanos(advance as u64));
        count
    }

    fn next_wait(&self) -> Option<Duration> {
        let inner = self.inner.lock();
        inner
            .next_expiration
            .map(|deadline| deadline.saturating_sub(Self::now_for_clock(inner.clock_id)))
    }

    fn has_expired(&self) -> bool {
        let inner = self.inner.lock();
        inner
            .next_expiration
            .map(|deadline| Self::now_for_clock(inner.clock_id) >= deadline)
            .unwrap_or(false)
    }
}

impl File for TimerFd {
    fn metadata(&self) -> &FileMeta {
        &self.meta
    }

    fn read<'a>(&'a self, buf: &'a mut [u8], flags: OpenFlags) -> AsyscallRet {
        Box::pin(async move {
            if buf.len() < core::mem::size_of::<u64>() {
                return Err(SyscallErr::EINVAL);
            }
            loop {
                let count = {
                    let mut inner = self.inner.lock();
                    Self::expired_count(&mut inner)
                };
                if count > 0 {
                    buf[..8].copy_from_slice(&count.to_ne_bytes());
                    return Ok(8);
                }
                if flags.contains(OpenFlags::NONBLOCK) {
                    return Err(SyscallErr::EAGAIN);
                }
                let Some(wait) = self.next_wait() else {
                    return Err(SyscallErr::EAGAIN);
                };
                if wait.is_zero() {
                    continue;
                }
                ksleep(wait).await;
            }
        })
    }

    fn write<'a>(&'a self, _buf: &'a [u8], _flags: OpenFlags) -> AsyscallRet {
        Box::pin(async move { Err(SyscallErr::EINVAL) })
    }

    fn pollin(&self, _waker: Option<core::task::Waker>) -> GeneralRet<bool> {
        Ok(self.has_expired())
    }
}

pub fn sys_timerfd_create(clock_id: usize, flags: u32) -> SyscallRet {
    stack_trace!();
    let allowed = TFD_NONBLOCK | TFD_CLOEXEC;
    if flags & !allowed != 0 {
        return Err(SyscallErr::EINVAL);
    }
    if !matches!(clock_id, CLOCK_REALTIME | CLOCK_MONOTONIC) {
        return Err(SyscallErr::EINVAL);
    }
    let mut open_flags = OpenFlags::RDONLY;
    if flags & TFD_NONBLOCK != 0 {
        open_flags |= OpenFlags::NONBLOCK;
    }
    if flags & TFD_CLOEXEC != 0 {
        open_flags |= OpenFlags::CLOEXEC;
    }
    let timer = Arc::new(TimerFd::new(clock_id));
    let file: Arc<dyn File> = timer.clone();
    current_process().inner_handler(move |proc| {
        let fd = proc.fd_table.alloc_fd()?;
        proc.fd_table.put(fd, FdInfo::new(file, open_flags));
        TIMERFD_TABLE.lock().insert(fd, timer);
        Ok(fd)
    })
}

pub fn sys_timerfd_gettime(fd: usize, curr_value: usize) -> SyscallRet {
    stack_trace!();
    if curr_value == 0 {
        return Err(SyscallErr::EFAULT);
    }
    UserCheck::new()
        .check_writable_slice(curr_value as *mut u8, core::mem::size_of::<ITimerSpec>())?;
    let _sum_guard = SumGuard::new();
    let spec = current_process().inner_handler(|proc| {
        proc.fd_table.get_ref(fd).ok_or(SyscallErr::EBADF)?;
        let timer = TIMERFD_TABLE
            .lock()
            .get(&fd)
            .cloned()
            .ok_or(SyscallErr::EINVAL)?;
        Ok(timer.snapshot())
    })?;
    unsafe {
        (curr_value as *mut ITimerSpec).write_volatile(spec);
    }
    Ok(0)
}

pub fn sys_timerfd_settime(
    fd: usize,
    flags: u32,
    new_value: usize,
    old_value: usize,
) -> SyscallRet {
    stack_trace!();
    let allowed = TFD_TIMER_ABSTIME | TFD_TIMER_CANCEL_ON_SET;
    if flags & !allowed != 0 {
        return Err(SyscallErr::EINVAL);
    }
    if new_value == 0 {
        return Err(SyscallErr::EFAULT);
    }
    UserCheck::new()
        .check_readable_slice(new_value as *const u8, core::mem::size_of::<ITimerSpec>())?;
    if old_value != 0 {
        UserCheck::new()
            .check_writable_slice(old_value as *mut u8, core::mem::size_of::<ITimerSpec>())?;
    }
    let _sum_guard = SumGuard::new();
    let new_spec = unsafe { *(new_value as *const ITimerSpec) };
    let interval = TimerFd::validate_timespec(new_spec.it_interval)?;
    let value = TimerFd::validate_timespec(new_spec.it_value)?;
    let old_spec = current_process().inner_handler(|proc| {
        proc.fd_table.get_ref(fd).ok_or(SyscallErr::EBADF)?;
        let timer = TIMERFD_TABLE
            .lock()
            .get(&fd)
            .cloned()
            .ok_or(SyscallErr::EINVAL)?;
        let old_spec = timer.snapshot();
        let mut inner = timer.inner.lock();
        inner.interval = interval;
        inner.next_expiration = if value.is_zero() {
            None
        } else if flags & TFD_TIMER_ABSTIME != 0 {
            Some(value)
        } else {
            Some(TimerFd::now_for_clock(inner.clock_id) + value)
        };
        Ok(old_spec)
    })?;
    if old_value != 0 {
        unsafe {
            (old_value as *mut ITimerSpec).write_volatile(old_spec);
        }
    }
    Ok(0)
}

pub fn sys_get_time(time_val_ptr: *mut TimeVal, timezone_ptr: *mut u8) -> SyscallRet {
    stack_trace!();
    let user_check = UserCheck::new();
    if !time_val_ptr.is_null() {
        user_check
            .check_writable_slice(time_val_ptr as *mut u8, core::mem::size_of::<TimeVal>())?;
    }
    if !timezone_ptr.is_null() {
        user_check.check_writable_slice(timezone_ptr, core::mem::size_of::<[i32; 2]>())?;
    }
    drop(user_check);
    let _sum_guard = SumGuard::new();
    let current_time = current_time_ms();
    let time_val = TimeVal {
        sec: current_time / 1000,
        usec: current_time % 1000 * 1000,
    };
    // debug!("get time of day, time(ms): {}", current_time);
    unsafe {
        if !time_val_ptr.is_null() {
            time_val_ptr.write_volatile(time_val);
        }
        if !timezone_ptr.is_null() {
            timezone_ptr.write_bytes(0, core::mem::size_of::<[i32; 2]>());
        }
    }
    Ok(0)
}

pub fn sys_clock_settime(clock_id: usize, time_spec_ptr: *const TimeSpec) -> SyscallRet {
    stack_trace!();
    UserCheck::new()
        .check_readable_slice(time_spec_ptr as *const u8, core::mem::size_of::<TimeSpec>())?;
    let _sum_guard = SumGuard::new();
    let time_spec = unsafe { &*time_spec_ptr };
    if (time_spec.sec as isize) < 0 {
        debug!("Cannot set time. sec is negative");
        return Err(SyscallErr::EINVAL);
    } else if (time_spec.nsec as isize) < 0 || time_spec.nsec > 999999999 {
        debug!("Cannot set time. nsec is invalid");
        return Err(SyscallErr::EINVAL);
    } else if clock_id == CLOCK_REALTIME && time_spec.sec < current_time_ms() / 1000 {
        debug!("set the time to a value less than the current value of the CLOCK_MONOTONIC clock.");
        return Err(SyscallErr::EINVAL);
    } else if clock_id == CLOCK_PROCESS_CPUTIME_ID {
        debug!("Cannot set this clock");
        return Err(SyscallErr::EPERM);
    }

    // calculate the diff
    // arg_timespec - device_timespec = diff
    let dev_spec = current_time_spec();
    let diff_time = Duration::from(*time_spec) - current_time_duration();
    // let diff_spec = TimeDiff {
    //     sec: time_spec.sec   - dev_spec.sec  ,
    //     nsec: time_spec.nsec   - dev_spec.nsec  ,
    // };
    log::info!(
        "[sys_clock_settime] arg time spec {:?}, dev curr time spec {:?}",
        Duration::from(*time_spec),
        Duration::from(dev_spec)
    );

    let mut manager_unlock = CLOCK_MANAGER.lock();
    manager_unlock.0.insert(clock_id, diff_time);
    if clock_id == CLOCK_REALTIME {
        set_realtime_offset(diff_time);
    }

    Ok(0)
}

pub fn sys_clock_gettime(clock_id: usize, time_spec_ptr: *mut TimeSpec) -> SyscallRet {
    stack_trace!();
    UserCheck::new()
        .check_writable_slice(time_spec_ptr as *mut u8, core::mem::size_of::<TimeSpec>())?;
    let _sum_guard = SumGuard::new();
    if clock_id == CLOCK_PROCESS_CPUTIME_ID {
        let cpu_time = current_process().inner_handler(|proc| {
            let mut user_time = Duration::ZERO;
            let mut sys_time = Duration::ZERO;
            for (_, thread) in proc.threads.iter() {
                if let Some(thread) = thread.upgrade() {
                    // TODO: is it ok to just read the other thread's unsafe cell data?
                    user_time += unsafe { (*thread.inner.get()).time_info.user_time };
                    sys_time += unsafe { (*thread.inner.get()).time_info.sys_time };
                }
            }
            user_time + sys_time
        });
        debug!("[sys_clock_gettime] get process cpu time: {:?}", cpu_time);
        unsafe {
            time_spec_ptr.write_volatile(cpu_time.into());
        }
        return Ok(0);
    }
    if clock_id == CLOCK_MONOTONIC {
        unsafe {
            time_spec_ptr.write_volatile(current_time_duration().into());
        }
        return Ok(0);
    }
    if clock_id == CLOCK_REALTIME {
        unsafe {
            time_spec_ptr.write_volatile((current_time_duration() + realtime_offset()).into());
        }
        return Ok(0);
    }
    let manager_locked = CLOCK_MANAGER.lock();
    let clock = manager_locked.0.get(&clock_id);
    match clock {
        Some(clock) => {
            trace!("[sys_clock_gettime] find the clock, clock id {}", clock_id);
            let dev_time = current_time_duration();
            let clock_time = dev_time + *clock;
            log::debug!("[sys_clock_gettime] get time {:?}", clock_time);
            unsafe {
                time_spec_ptr.write_volatile(clock_time.into());
            }
            Ok(0)
        }
        None => {
            trace!("[sys_clock_gettime] Cannot find the clock: {}", clock_id);
            Err(SyscallErr::EINVAL)
        }
    }
}

pub fn sys_clock_getres(clock_id: usize, res: *mut TimeSpec) -> SyscallRet {
    stack_trace!();
    let _sum_guard = SumGuard::new();
    UserCheck::new().check_writable_slice(res as *mut u8, core::mem::size_of::<TimeSpec>())?;
    if matches!(
        clock_id,
        CLOCK_REALTIME | CLOCK_MONOTONIC | CLOCK_PROCESS_CPUTIME_ID
    ) {
        unsafe {
            res.write_volatile(Duration::from_nanos(1).into());
        }
        return Ok(0);
    }
    let manager_locked = CLOCK_MANAGER.lock();
    let clock = manager_locked.0.get(&clock_id);
    match clock {
        Some(_clock) => {
            trace!("[sys_clock_getres] find the clock, clock id {}", clock_id);
            let resolution = Duration::from_nanos(1);
            info!("[sys_clock_getres] get time {:?}", resolution);
            unsafe {
                res.write_volatile(resolution.into());
            }
            Ok(0)
        }
        None => {
            trace!("[sys_clock_getres] Cannot find the clock: {}", clock_id);
            Err(SyscallErr::EINVAL)
        }
    }
}

pub async fn sys_clock_nanosleep(
    _clock_id: usize,
    flags: u32,
    request: usize,
    remain: usize,
) -> SyscallRet {
    stack_trace!();
    let _sum_guard = SumGuard::new();
    let size = core::mem::size_of::<TimeSpec>();
    UserCheck::new().check_readable_slice(request as *const u8, size)?;
    let request_time = unsafe { *(request as *const TimeSpec) };
    if (request_time.sec as isize) < 0 || request_time.nsec >= 1_000_000_000 {
        return Err(SyscallErr::EINVAL);
    }
    let request: Duration = request_time.into();
    let has_remain = if (remain as *mut TimeSpec).is_null() {
        false
    } else {
        true
    };
    let current = current_time_duration();
    if flags as usize == TIMER_ABSTIME {
        // request time is absolutely
        if request.le(&current) {
            return Ok(0);
        }
        let sleep = request - current;
        ksleep(sleep).await;
        return Ok(0);
    } else {
        // request time is relative
        match Select2Futures::new(
            ksleep(request),
            current_task().wait_for_events(Event::all()),
        )
        .await
        {
            SelectOutput::Output1(_) => {}
            SelectOutput::Output2(intr) => {
                log::warn!("[sys_nanosleep] interrupt by event {:?}", intr);
                return Err(SyscallErr::EINTR);
            }
        };
        // ksleep(request).await;
        if has_remain {
            UserCheck::new().check_writable_slice(remain as *mut u8, size)?;
            unsafe {
                *(remain as *mut TimeSpec) = Duration::ZERO.into();
            }
        }
        return Ok(0);
    }
}

pub fn sys_times(buf: *mut Tms) -> SyscallRet {
    stack_trace!();
    const CLK_TCK: usize = 100;
    let ticks = (current_time_duration().as_micros() as usize * CLK_TCK) / 1_000_000;
    if buf.is_null() {
        return Ok(ticks);
    }
    UserCheck::new().check_writable_slice(buf as *mut u8, core::mem::size_of::<Tms>())?;
    let _sum_guard = SumGuard::new();
    let tms = unsafe { &mut *buf };
    let cpu_time = current_process().inner_handler(|proc| {
        let mut user_time = Duration::ZERO;
        let mut sys_time = Duration::ZERO;
        for (_, thread) in proc.threads.iter() {
            if let Some(thread) = thread.upgrade() {
                user_time += unsafe { (*thread.inner.get()).time_info.user_time };
                sys_time += unsafe { (*thread.inner.get()).time_info.sys_time };
            }
        }
        (user_time, sys_time)
    });
    tms.utime = (cpu_time.0.as_micros() as usize * CLK_TCK) / 1_000_000;
    tms.stime = (cpu_time.1.as_micros() as usize * CLK_TCK) / 1_000_000;
    tms.cutime = 0;
    tms.cstime = 0;
    Ok(ticks)
}

pub async fn sys_nanosleep(request_ptr: usize, remain_ptr: usize) -> SyscallRet {
    stack_trace!();
    let sleep_time = {
        UserCheck::new()
            .check_readable_slice(request_ptr as *const u8, core::mem::size_of::<TimeSpec>())?;
        let _sum_guard = SumGuard::new();

        let request_ptr = request_ptr as *const TimeSpec;
        unsafe { *request_ptr }
    };
    if (sleep_time.sec as isize) < 0 || sleep_time.nsec >= 1_000_000_000 {
        return Err(SyscallErr::EINVAL);
    }
    let sleep_duration = Duration::from(sleep_time);
    match Select2Futures::new(
        ksleep(sleep_duration),
        current_task().wait_for_events(Event::all()),
    )
    .await
    {
        SelectOutput::Output1(_) => Ok(0),
        SelectOutput::Output2(intr) => {
            log::info!("[sys_nanosleep] interrupt by event {:?}", intr);
            if remain_ptr != 0 {
                UserCheck::new().check_writable_slice(
                    remain_ptr as *mut u8,
                    core::mem::size_of::<TimeSpec>(),
                )?;
                let _sum_guard = SumGuard::new();
                unsafe {
                    *(remain_ptr as *mut TimeSpec) = Duration::ZERO.into();
                }
            }
            Err(SyscallErr::EINTR)
        }
    }
    // ksleep().await;
    // Ok(0)
}

const ITIMER_REAL: i32 = 0;
const ITIMER_VIRTUAL: i32 = 1;
const ITIMER_PROF: i32 = 2;

pub fn sys_setitimer(
    which: i32,
    new_value: *const ITimerval,
    old_value: *mut ITimerval,
) -> SyscallRet {
    stack_trace!();

    let current_pid = current_process().pid();

    UserCheck::new()
        .check_readable_slice(new_value as *const u8, core::mem::size_of::<ITimerval>())?;

    let _sum_guard = SumGuard::new();

    let new_value = unsafe { &*new_value };
    let interval = Duration::from(new_value.it_interval);
    let next_timeout = Duration::from(new_value.it_value);
    info!(
        "[sys_settimer]: which {}, new_value{{ interval:{:?}, value:{:?} }}",
        which, interval, next_timeout
    );

    let idx = match which {
        ITIMER_REAL => which,
        _ => todo!(),
    };
    if old_value as usize != 0 {
        UserCheck::new()
            .check_writable_slice(old_value as *mut u8, core::mem::size_of::<ITimerval>())?;
        let old_value = unsafe { &mut *old_value };
        *old_value = current_process().inner_handler(|proc| {
            let next_trigger_ts = Duration::from(proc.timers[idx as usize].it_value);
            let mut value = next_trigger_ts;
            if !value.is_zero() {
                let current_ts = current_time_duration();
                if value > current_ts {
                    value -= current_time_duration();
                } else {
                    value = Duration::ZERO;
                }
            }
            proc.timers[idx as usize].it_value = value.into();
            proc.timers[idx as usize]
        });
        log::debug!("[sys_settimer] old timer {:?}", old_value);
    }

    match which {
        ITIMER_REAL => {
            let callback = move || {
                stack_trace!();
                if let Some(process) = PROCESS_MANAGER.get(current_pid) {
                    if process.inner_handler(|proc| {
                        let mut timer = proc.timers[ITIMER_REAL as usize];
                        if Duration::from(timer.it_value).is_zero() {
                            timer.it_value = Duration::ZERO.into();
                            false
                        } else {
                            let expired_time = current_time_duration() + interval;
                            timer.it_value = expired_time.into();
                            true
                        }
                    }) {
                        process.recv_signal(SIGALRM).unwrap()
                    } else {
                        return false;
                    }
                    if interval.is_zero() {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            };
            if next_timeout.is_zero() {
                // Disarm the old timer
                current_process().inner_handler(|proc| {
                    proc.timers[ITIMER_REAL as usize].it_value = Duration::ZERO.into();
                });
            } else {
                current_process().inner_handler(|proc| {
                    proc.timers[ITIMER_REAL as usize].it_value =
                        (current_time_duration() + next_timeout).into();
                });
                spawn_kernel_thread(async move {
                    TimedTaskFuture::new(interval, callback, next_timeout + current_time_duration())
                        .await
                });
            }
            which
        }
        ITIMER_VIRTUAL => {
            todo!()
        }
        ITIMER_PROF => {
            todo!()
        }
        _ => {
            return Err(SyscallErr::EINVAL);
        }
    };

    Ok(0)
}
