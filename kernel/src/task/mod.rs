mod context;

use core::arch::global_asm;

use context::TaskContext;
use crate::mm::MemorySet;
use crate::trap::TrapContext;

global_asm!(include_str!("switch.S"));

const MAX_TASKS: usize = crate::user::MAX_USER_TASKS;
const MMAP_BASE: usize = 0x4000_0000;
const NO_PARENT: usize = usize::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub trap_cx_addr: usize,
    pub task_cx: TaskContext,
    pub memory_set: Option<MemorySet>,
    pub satp_token: usize,
    pub heap_bottom: usize,
    pub heap_end: usize,
    pub mmap_end: usize,
    pub parent: usize,
    pub exit_code: i32,
    pub waited: bool,
}

impl TaskControlBlock {
    pub const fn zero_init() -> Self {
        Self {
            status: TaskStatus::Exited,
            trap_cx_addr: 0,
            task_cx: TaskContext::zero_init(),
            memory_set: None,
            satp_token: 0,
            heap_bottom: 0,
            heap_end: 0,
            mmap_end: MMAP_BASE,
            parent: NO_PARENT,
            exit_code: 0,
            waited: true,
        }
    }
}

static mut TASKS: [TaskControlBlock; MAX_TASKS] =
    [const { TaskControlBlock::zero_init() }; MAX_TASKS];

static mut CURRENT: usize = 0;

pub fn init() {
    let mut i = 0;
    let use_external_app = crate::loader::has_external_app();
    let boot_task_count = if use_external_app {
        1
    } else {
        crate::loader::APP_NUM
    };

    while i < MAX_TASKS {
        if i >= boot_task_count {
            unsafe {
                TASKS[i] = TaskControlBlock::zero_init();
            }

            i += 1;
            continue;
        }

        init_task(i, use_external_app);

        i += 1;
    }
}

fn init_task(task_id: usize, use_external_app: bool) {
    unsafe {
        if use_external_app && task_id == 0 {
            crate::fs::reset_for_external_program();
        }

        let memory_set = MemorySet::new_user(task_id);
        let satp_token = memory_set.satp_token();
        let heap_bottom = if use_external_app && task_id == 0 {
            crate::loader::external_app_heap_base()
        } else {
            crate::loader::USER_HEAP_BASE
        };

        TASKS[task_id] = TaskControlBlock {
            status: TaskStatus::Ready,
            trap_cx_addr: crate::user::init_user_context(task_id),
            task_cx: TaskContext::zero_init(),
            memory_set: Some(memory_set),
            satp_token,
            heap_bottom,
            heap_end: heap_bottom,
            mmap_end: MMAP_BASE,
            parent: NO_PARENT,
            exit_code: 0,
            waited: true,
        };

        if !use_external_app {
            crate::println!("task {} user space ready: satp={:#x}", task_id, satp_token);
        }
    }
}

pub fn run_first_task() -> ! {
   run_task(0)
}

pub fn current_task_id() -> usize {
    unsafe { CURRENT }
}

pub fn current_pid() -> usize {
    unsafe { task_pid(CURRENT) }
}

pub fn current_ppid() -> usize {
    unsafe {
        let parent = TASKS[CURRENT].parent;
        if parent == NO_PARENT {
            1
        } else {
            task_pid(parent)
        }
    }
}

pub fn current_brk() -> usize {
    unsafe { TASKS[CURRENT].heap_end }
}

pub fn set_current_brk(new_brk: usize) -> usize {
    unsafe {
        let current = CURRENT;
        let old_brk = TASKS[current].heap_end;
        let heap_bottom = TASKS[current].heap_bottom;
        let heap_top = heap_bottom + crate::loader::USER_HEAP_SIZE;

        if new_brk == 0 {
            return old_brk;
        }

        if new_brk < heap_bottom || new_brk > heap_top {
            return old_brk;
        }

        if new_brk > old_brk {
            let mapped = match TASKS[current].memory_set.as_ref() {
                Some(memory_set) => memory_set.map_user_zero_range(old_brk, new_brk),
                None => false,
            };

            if !mapped {
                return old_brk;
            }

            crate::mm::activate_satp(TASKS[current].satp_token);
        }

        TASKS[current].heap_end = new_brk;
        TASKS[current].heap_end
    }
}

pub fn alloc_current_mmap(len: usize) -> usize {
    unsafe {
        let current = CURRENT;
        let start = round_up(TASKS[current].mmap_end, crate::mm::PAGE_SIZE);
        let end = round_up(start + len, crate::mm::PAGE_SIZE);
        TASKS[current].mmap_end = end;
        start
    }
}

pub fn map_current_user_range(start: usize, end: usize) -> bool {
    unsafe {
        let current = CURRENT;
        let mapped = match TASKS[current].memory_set.as_ref() {
            Some(memory_set) => memory_set.map_user_zero_range(start, end),
            None => false,
        };

        if mapped {
            crate::mm::activate_satp(TASKS[current].satp_token);
        }

        mapped
    }
}

pub fn clone_current(parent_cx: &TrapContext, _flags: usize, user_stack: usize) -> isize {
    unsafe {
        let parent = CURRENT;
        let child = match find_free_task() {
            Some(id) => id,
            None => return -1,
        };

        let child_cx_addr = crate::user::trap_context_addr(child);
        let child_cx = &mut *(child_cx_addr as *mut TrapContext);

        let clone_stack = if user_stack == 0 {
            copy_user_stack(parent, child);
            0
        } else {
            copy_clone_stack_args(user_stack, child_cx_addr)
        };

        *child_cx = *parent_cx;
        child_cx.x[10] = 0;
        if user_stack != 0 {
            child_cx.x[2] = clone_stack;
        } else {
            relocate_stack_registers(child_cx, parent, child);
        }

        TASKS[child] = TaskControlBlock {
            status: TaskStatus::Ready,
            trap_cx_addr: child_cx_addr,
            task_cx: TaskContext::zero_init(),
            memory_set: None,
            satp_token: TASKS[parent].satp_token,
            heap_bottom: TASKS[parent].heap_bottom,
            heap_end: TASKS[parent].heap_end,
            mmap_end: TASKS[parent].mmap_end,
            parent,
            exit_code: 0,
            waited: false,
        };

        task_pid(child) as isize
    }
}

pub fn wait_child(pid: usize, status_ptr: usize) -> isize {
    unsafe {
        let current = CURRENT;
        let wait_any = pid == usize::MAX;
        let mut id = 0usize;

        while id < MAX_TASKS {
            if TASKS[id].parent == current
                && TASKS[id].status == TaskStatus::Exited
                && !TASKS[id].waited
                && (wait_any || task_pid(id) == pid)
            {
                if status_ptr != 0 {
                    let wait_status = (TASKS[id].exit_code.max(0) as usize & 0xff) << 8;
                    (status_ptr as *mut i32).write(wait_status as i32);
                }
                TASKS[id].waited = true;
                return task_pid(id) as isize;
            }

            id += 1;
        }
    }

    -1
}

pub fn has_waitable_child(pid: usize) -> bool {
    unsafe {
        let current = CURRENT;
        let wait_any = pid == usize::MAX;
        let mut id = 0usize;

        while id < MAX_TASKS {
            if TASKS[id].parent == current
                && TASKS[id].status != TaskStatus::Exited
                && (wait_any || task_pid(id) == pid)
            {
                return true;
            }

            id += 1;
        }
    }

    false
}

pub fn exec_current() -> ! {
    let current = unsafe { CURRENT };
    init_task(current, true);
    run_task(current);
}

fn run_task(task_id: usize) -> ! {
    unsafe {
        CURRENT = task_id;
        TASKS[task_id].status = TaskStatus::Running;

        let trap_cx_addr = TASKS[task_id].trap_cx_addr;
        let satp_token = TASKS[task_id].satp_token;

        if !crate::loader::has_external_app() {
            crate::println!(
                "run task {}, switch_satp={:#x}",
                task_id,
                TASKS[task_id].satp_token,
            );
        }

        crate::mm::activate_satp(satp_token);
        crate::trap::restore(trap_cx_addr);
    }
}

pub fn suspend_current_and_run_next(trap_cx_addr: usize) {
    let current = unsafe { CURRENT };

    unsafe {
        TASKS[current].trap_cx_addr = trap_cx_addr;
        TASKS[current].status = TaskStatus::Ready;
    }

    if let Some(next) = find_next_ready() {
        run_task(next);
    }

    unsafe {
        if TASKS[current].status == TaskStatus::Ready {
            run_task(current);
        }
    }

    panic!("no ready task after yield");
}

pub fn run_next_ready_after_syscall(trap_cx_addr: usize) {
    let current = unsafe { CURRENT };

    unsafe {
        TASKS[current].trap_cx_addr = trap_cx_addr;
        TASKS[current].status = TaskStatus::Ready;
    }

    if let Some(next) = find_next_ready() {
        run_task(next);
    }

    unsafe {
        if TASKS[current].status == TaskStatus::Ready {
            run_task(current);
        }
    }

    panic!("no ready task after syscall schedule");
}

pub fn exit_current(code: i32) -> ! {
    let current = unsafe { CURRENT };

    if !crate::loader::has_external_app() {
        crate::println!("task {} exited with code {}", current, code);
    }

    unsafe {
        TASKS[current].status = TaskStatus::Exited;
        TASKS[current].exit_code = code;
    }

    if let Some(next) = find_next_ready() {
        run_task(next);
    }

    if crate::loader::has_external_app() {
        crate::drivers::ext4::report_current_external_result(code);
    }

    if crate::loader::has_external_app() && crate::drivers::ext4::load_next_queued_external() {
        init_task(0, true);
        run_task(0);
    }

    if crate::loader::has_external_app() {
        crate::loader::print_external_group_end();
    }

    crate::println!("all tasks exited");
    crate::sbi::shutdown();
}

fn find_next_ready() -> Option<usize> {
    let current = unsafe { CURRENT };

    let mut offset = 1;
    while offset <= MAX_TASKS {
        let id = (current + offset) % MAX_TASKS;

        unsafe {
            if TASKS[id].status == TaskStatus::Ready {
                return Some(id);
            }
        }

        offset += 1;
    }

    None
}

fn find_free_task() -> Option<usize> {
    let mut id = 0usize;

    while id < MAX_TASKS {
        unsafe {
            if TASKS[id].status == TaskStatus::Exited && TASKS[id].waited {
                return Some(id);
            }
        }

        id += 1;
    }

    None
}

fn task_pid(task_id: usize) -> usize {
    task_id + 1
}

fn copy_user_stack(parent: usize, child: usize) {
    let (parent_bottom, parent_top) = crate::user::user_stack_range(parent);
    let (child_bottom, _) = crate::user::user_stack_range(child);
    let len = parent_top - parent_bottom;

    unsafe {
        core::ptr::copy_nonoverlapping(parent_bottom as *const u8, child_bottom as *mut u8, len);
    }
}

fn relocate_stack_registers(cx: &mut TrapContext, parent: usize, child: usize) {
    let (parent_bottom, parent_top) = crate::user::user_stack_range(parent);
    let (child_bottom, _) = crate::user::user_stack_range(child);
    let mut index = 1usize;

    while index < cx.x.len() {
        let value = cx.x[index];
        if value >= parent_bottom && value < parent_top {
            cx.x[index] = child_bottom + (value - parent_bottom);
        }
        index += 1;
    }
}

fn copy_clone_stack_args(user_stack: usize, child_cx_addr: usize) -> usize {
    let child_sp = child_cx_addr - 16;

    unsafe {
        core::ptr::copy_nonoverlapping(user_stack as *const u8, child_sp as *mut u8, 16);
    }

    child_sp
}

fn round_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
