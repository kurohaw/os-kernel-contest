pub const SYS_TEST: usize = 0;
pub const SYS_EXIT: usize = 1;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYS_TEST => sys_test(args[0]),
        SYS_EXIT => sys_exit(args[0] as i32),
        _ => {
            crate::println!("unsupported syscall: id={}", id);
            -1
        }
    }
}

fn sys_test(value: usize) -> isize {
    crate::println!("sys_test called, arg0={}", value);
    (value + 1) as isize
}

fn sys_exit(code: i32) -> isize {
    crate::println!("user exited with code {}", code);
    loop{}
}