mod context;

use core::arch::global_asm;

use context::TaskContext;

global_asm!(include_str!("switch.S"));

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
}

static mut INIT_TASK: Option<TaskControlBlock> = None;

pub fn init() {
    let trap_cx_addr = crate::user::init_user_context();

    unsafe {
        INIT_TASK = Some(TaskControlBlock {
            status: TaskStatus::Ready,
            trap_cx_addr,
            task_cx: TaskContext::zero_init(),
        });
    }
}

pub fn run_first_task() -> ! {
    let task = unsafe {
        INIT_TASK
            .as_mut()
            .expect("init task must be initialized before run")
    };

    task.status = TaskStatus::Running;
    crate::println!("run first task");

    unsafe {
        crate::trap::restore(task.trap_cx_addr);
    }
}

pub fn suspend_current_and_run_next() {
    crate::println!("user yield");

    unsafe {
        if let Some(task) = INIT_TASK.as_mut() {
            task.status = TaskStatus::Ready;
            task.status = TaskStatus::Running;
        }
    }
}

pub fn exit_current(code: i32) -> ! {
    crate::println!("user exited with code {}", code);

    unsafe {
        if let Some(task) = INIT_TASK.as_mut() {
            task.status = TaskStatus::Exited;
        }
    }

    crate::println!("all tasks exited");
    loop{}
}