pub const SYS_TEST: usize = 0;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYS_TEST => sys_test(args[0]),
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
