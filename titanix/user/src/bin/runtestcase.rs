#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use user_lib::{close, execve, exit, fork, openat, read, wait, waitpid, OpenFlags};

#[macro_use]
extern crate user_lib;

const QUEUE_PATH: &str = "/oscomp-queue\0";
const END_MARKER_PATH: &str = "/oscomp-end\0";
const METADATA_LIMIT: usize = 4096;

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

fn run_test(name: &[u8]) {
    let mut path = Vec::new();
    path.extend_from_slice(b"/");
    path.extend_from_slice(name);
    path.push(0);

    let mut arg = Vec::new();
    arg.extend_from_slice(b"./");
    arg.extend_from_slice(name);
    arg.push(0);
    let argv = [arg.as_ptr(), core::ptr::null()];

    let pid = fork();
    if pid == 0 {
        let path = core::str::from_utf8(&path).unwrap();
        if execve(path, &argv, &[core::ptr::null::<u8>()]) != 0 {
            println!("oscomp: execve {} failed", path);
        }
        exit(-1);
    } else if pid > 0 {
        let mut exit_code: i32 = 0;
        waitpid(pid as usize, &mut exit_code);
    } else {
        println!("oscomp: fork failed");
    }
}

fn run_basic_queue() -> bool {
    let queue = match read_metadata(QUEUE_PATH) {
        Some(data) => data,
        None => return false,
    };
    let end_marker = read_metadata(END_MARKER_PATH).unwrap_or_default();

    let mut ran = false;
    for name in queue
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        ran = true;
        run_test(name);
    }
    if let Ok(marker) = core::str::from_utf8(&end_marker) {
        println!("{}", marker);
    }
    ran
}

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        if !run_basic_queue() {
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
