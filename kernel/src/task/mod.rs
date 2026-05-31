mod context;

use core::arch::global_asm;

use context::TaskContext;

global_asm!(include_str!("switch.S"));

const MAX_TASKS: usize = crate::user::APP_NUM;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub trap_cx_addr: usize,
    pub task_cx: TaskContext,
}

impl TaskControlBlock {
    pub const fn zero_init() -> Self {
        Self {
            status: TaskStatus::Exited,
            trap_cx_addr: 0,
            task_cx: TaskContext::zero_init(),
        }
    }
}

static mut TASKS: [TaskControlBlock; MAX_TASKS] = 
    [TaskControlBlock::zero_init(); MAX_TASKS];

static mut CURRENT: usize = 0;

pub fn init() {
    let mut i = 0;

    while i < MAX_TASKS {
        unsafe {
            TASKS[i] = TaskControlBlock {
                status: TaskStatus::Ready,
                trap_cx_addr: crate::user::init_user_context(i),
                task_cx: TaskContext::zero_init(),
            };
        }

        i += 1;
    }
}

pub fn run_first_task() -> ! {
   run_task(0)
}

fn run_task(task_id: usize) -> ! {
    unsafe {
        CURRENT = task_id;
        TASKS[task_id].status = TaskStatus::Running;

        crate::println!("run task {}", task_id);
        crate::trap::restore(TASKS[task_id].trap_cx_addr);
    }
}

pub fn suspend_current_and_run_next() {
    let current = unsafe { CURRENT };

    crate::println!("task {} yield", current);

    unsafe {
        TASKS[current].status = TaskStatus::Ready;
    }

    if let Some(next) = find_next_ready() {
        run_task(next);
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

    crate::println!("all tasks exited");
    loop{}
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