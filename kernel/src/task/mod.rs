
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub trap_cx_addr: usize,
}

static mut INIT_TASK: Option<TaskControlBlock> = None;

pub fn init() {
    let trap_cx_addr = crate::user::init_user_context();

    unsafe {
        INIT_TASK = Some(TaskControlBlock {
            status: TaskStatus::Ready,
            trap_cx_addr,
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