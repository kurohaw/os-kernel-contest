#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use user_lib::{close, execve, fork, openat, read, wait, waitpid, OpenFlags};

#[macro_use]
extern crate user_lib;

const EXECUTABLE_PATH: &str = "/oscomp-first\0";
const ARGV_PATH: &str = "/oscomp-argv\0";
const END_MARKER_PATH: &str = "/oscomp-end\0";
const METADATA_LIMIT: usize = 1024;

fn read_metadata(path: &str) -> Option<Vec<u8>> {
    let fd = openat(path, OpenFlags::O_RDONLY);
    if fd < 0 {
        return None;
    }
    let mut data = Vec::new();
    data.resize(METADATA_LIMIT, 0);
    let count = read(fd as usize, &mut data);
    close(fd as usize);
    if count < 0 {
        return None;
    }
    data.truncate(count as usize);
    Some(data)
}

fn run_first_test() -> bool {
    let argv_data = match read_metadata(ARGV_PATH) {
        Some(data) => data,
        None => return false,
    };
    let end_marker = read_metadata(END_MARKER_PATH).unwrap_or_default();

    let mut arg_storage: Vec<Vec<u8>> = argv_data
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .map(|arg| {
            let mut value = arg.to_vec();
            value.push(0);
            value
        })
        .collect();
    if arg_storage.is_empty() {
        arg_storage.push(b"./oscomp-first\0".to_vec());
    }
    let mut argv: Vec<*const u8> = arg_storage.iter().map(|arg| arg.as_ptr()).collect();
    argv.push(core::ptr::null());

    let pid = fork();
    if pid == 0 {
        if execve(EXECUTABLE_PATH, &argv, &[core::ptr::null::<u8>()]) != 0 {
            println!("oscomp: execve first basic ELF failed");
            return true;
        }
    } else {
        let mut exit_code: i32 = 0;
        waitpid(pid as usize, &mut exit_code);
        if let Ok(marker) = core::str::from_utf8(&end_marker) {
            println!("{}", marker);
        }
    }
    true
}

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        if !run_first_test() {
            println!("oscomp: no staged basic ELF");
        }
        println!(" !TEST FINISH! ");
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let _pid = wait(&mut exit_code);
            // println!(
            //     "[initproc] Released a zombie process, pid={}, exit_code={}",
            //     pid, exit_code,
            // );
        }
    }
    0
}
