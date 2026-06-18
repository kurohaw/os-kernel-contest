#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use user_lib::{chdir, close, execve, exit, fork, openat, read, wait, waitpid, OpenFlags};

#[macro_use]
extern crate user_lib;

const QUEUE_PATH: &str = "/oscomp-queue\0";
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
    path.extend_from_slice(b"./");
    path.extend_from_slice(name);
    path.push(0);

    let argv = [path.as_ptr(), core::ptr::null()];

    let pid = fork();
    if pid == 0 {
        let path = core::str::from_utf8(&path).unwrap();
        let result = execve(path, &argv, &[core::ptr::null::<u8>()]);
        if result != 0 {
            println!("oscomp: execve {} failed: {}", path, result);
        }
        exit(-1);
    } else if pid > 0 {
        let mut exit_code: i32 = 0;
        waitpid(pid as usize, &mut exit_code);
    } else {
        println!("oscomp: fork failed");
    }
}

fn enter_group(record: &[u8]) {
    let Some(separator) = record.iter().position(|byte| *byte == b'\t') else {
        println!("oscomp: malformed group record");
        return;
    };
    let (path, marker) = record.split_at(separator);
    let marker = &marker[1..];

    let mut path_with_nul = path.to_vec();
    path_with_nul.push(0);
    let Ok(path) = core::str::from_utf8(&path_with_nul) else {
        println!("oscomp: invalid group path");
        return;
    };
    if chdir(path) != 0 {
        println!("oscomp: cannot enter group {}", path);
        return;
    }
    if !marker.is_empty() {
        let Ok(marker) = core::str::from_utf8(marker) else {
            return;
        };
        println!("{}", marker);
    }
}

fn run_basic_queue() -> bool {
    let queue = match read_metadata(QUEUE_PATH) {
        Some(data) => data,
        None => return false,
    };

    let mut ran = false;
    for record in queue
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        match record[0] {
            b'G' => enter_group(&record[1..]),
            b'X' => {
                ran = true;
                run_test(&record[1..]);
            }
            b'E' => {
                if let Ok(marker) = core::str::from_utf8(&record[1..]) {
                    println!("{}", marker);
                }
            }
            _ => println!("oscomp: unknown queue record"),
        }
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
