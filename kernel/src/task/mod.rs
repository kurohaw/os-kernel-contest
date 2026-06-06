mod context;

use core::arch::global_asm;

use context::TaskContext;
use crate::mm::MemorySet;

global_asm!(include_str!("switch.S"));

const MAX_TASKS: usize = crate::user::APP_NUM;
const MMAP_BASE: usize = 0x4000_0000;

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
    }
}
}

static mut TASKS: [TaskControlBlock; MAX_TASKS] =
    [const { TaskControlBlock::zero_init() }; MAX_TASKS];

static mut CURRENT: usize = 0;

pub fn init() {
    let mut i = 0;
    let use_external_app = crate::loader::has_external_app();

    while i < MAX_TASKS {
        if use_external_app && i > 0 {
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
            crate::fs::reset_cwd_from_loader();
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
        };

        if use_external_app {
            crate::println!(
                "task {} external user space ready: satp={:#x}, heap={:#x}",
                task_id,
                satp_token,
                heap_bottom,
            );
        } else {
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

fn run_task(task_id: usize) -> ! {
    unsafe {
        CURRENT = task_id;
        TASKS[task_id].status = TaskStatus::Running;

        let trap_cx_addr = TASKS[task_id].trap_cx_addr;
        let satp_token = TASKS[task_id].satp_token;

        crate::println!(
            "run task {}, switch_satp={:#x}",
            task_id,
            TASKS[task_id].satp_token,
        );

        crate::mm::activate_satp(satp_token);
        crate::trap::restore(trap_cx_addr);
    }
}

pub fn suspend_current_and_run_next(trap_cx_addr: usize) {
    let current = unsafe { CURRENT };

    crate::println!("task {} yield", current);

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

pub fn exit_current(code: i32) -> ! {
    let current = unsafe { CURRENT };

    crate::println!("task {} exited with code {}",current, code);

    unsafe {
        TASKS[current].status = TaskStatus::Exited;
    }

    if let Some(next) = find_next_ready() {
        run_task(next);
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

fn round_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
