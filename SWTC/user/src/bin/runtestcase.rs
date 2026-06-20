#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use user_lib::{
    chdir, close, execve, exit, fork, kill, openat, read, sleep, wait, waitpid, waitpid_options,
    OpenFlags,
};

#[macro_use]
extern crate user_lib;

const QUEUE_PATH: &str = "/oscomp-queue\0";
const METADATA_LIMIT: usize = 64 * 1024;
const METADATA_CHUNK: usize = 1024;
const WNOHANG: i32 = 1;
const SIGKILL: i32 = 9;
const WAIT_POLL_MS: usize = 10;

fn read_metadata(path: &str) -> Option<Vec<u8>> {
    let fd = openat(path, OpenFlags::O_RDONLY);
    if fd < 0 {
        return None;
    }
    let mut data = Vec::new();
    while data.len() < METADATA_LIMIT {
        let mut chunk = [0u8; METADATA_CHUNK];
        let count = read(fd as usize, &mut chunk);
        if count < 0 {
            close(fd as usize);
            return None;
        }
        if count == 0 {
            break;
        }
        let count = count as usize;
        let remaining = METADATA_LIMIT - data.len();
        data.extend_from_slice(&chunk[..core::cmp::min(count, remaining)]);
        if count < METADATA_CHUNK {
            break;
        }
    }
    close(fd as usize);
    Some(data)
}

fn wait_for_child(pid: isize, timeout_ms: Option<usize>) -> Option<i32> {
    let mut exit_code: i32 = 0;
    let Some(timeout_ms) = timeout_ms else {
        if waitpid(pid as usize, &mut exit_code) == pid {
            return Some(exit_code);
        }
        return None;
    };

    let mut elapsed = 0usize;
    loop {
        let ret = waitpid_options(pid, &mut exit_code, WNOHANG);
        if ret == pid {
            return Some(exit_code);
        }
        if ret < 0 {
            println!("oscomp: waitpid {} failed: {}", pid, ret);
            return None;
        }
        if elapsed >= timeout_ms {
            println!(
                "oscomp: timeout after {} ms, killing pid {}",
                timeout_ms, pid
            );
            let result = kill(pid, SIGKILL);
            if result < 0 {
                println!("oscomp: kill {} failed: {}", pid, result);
                return None;
            }
            for _ in 0..100 {
                let ret = waitpid_options(pid, &mut exit_code, WNOHANG);
                if ret == pid || ret < 0 {
                    return None;
                }
                sleep(WAIT_POLL_MS);
            }
            println!("oscomp: killed pid {} but it did not exit promptly", pid);
            return None;
        }
        sleep(WAIT_POLL_MS);
        elapsed += WAIT_POLL_MS;
    }
}

fn run_test_with_argv(
    name: &[u8],
    argv0: Option<&[u8]>,
    args: &[&[u8]],
    timeout_ms: Option<usize>,
) -> Option<i32> {
    let mut path = Vec::new();
    path.extend_from_slice(b"./");
    path.extend_from_slice(name);
    path.push(0);

    let mut argv_buffers = Vec::new();
    if let Some(argv0) = argv0 {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(argv0);
        buffer.push(0);
        argv_buffers.push(buffer);
    } else {
        argv_buffers.push(path.clone());
    }
    for arg in args {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(arg);
        buffer.push(0);
        argv_buffers.push(buffer);
    }

    let mut argv = Vec::new();
    for buffer in &argv_buffers {
        argv.push(buffer.as_ptr());
    }
    argv.push(core::ptr::null());

    let pid = fork();
    if pid == 0 {
        let path = core::str::from_utf8(&path).unwrap();
        let result = execve(path, &argv, &[core::ptr::null::<u8>()]);
        if result != 0 {
            println!("oscomp: execve {} failed: {}", path, result);
        }
        exit(-1);
    } else if pid > 0 {
        wait_for_child(pid, timeout_ms)
    } else {
        println!("oscomp: fork failed");
        None
    }
}

fn run_test(name: &[u8]) {
    let _ = run_test_with_argv(name, None, &[], None);
}

fn parse_usize(bytes: &[u8]) -> Option<usize> {
    let mut value = 0usize;
    if bytes.is_empty() {
        return None;
    }
    for byte in bytes.iter().copied() {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add((byte - b'0') as usize)?;
    }
    Some(value)
}

fn run_argv_record(record: &[u8]) {
    let mut fields = record.split(|byte| *byte == b'\t');
    let Some(timeout) = fields.next().and_then(parse_usize) else {
        println!("oscomp: malformed argv timeout record");
        return;
    };
    let Some(name) = fields.next().filter(|field| !field.is_empty()) else {
        println!("oscomp: malformed argv executable record");
        return;
    };
    let mut args: Vec<&[u8]> = fields.collect();
    let argv0 = if name == b"lmbench_all" && !args.is_empty() {
        Some(args.remove(0))
    } else {
        None
    };
    let _ = run_test_with_argv(name, argv0, &args, Some(timeout));
}

fn run_libctest_record(record: &[u8]) {
    let mut fields = record.split(|byte| *byte == b'\t');
    let Some(timeout) = fields.next().and_then(parse_usize) else {
        println!("oscomp: malformed libctest timeout record");
        return;
    };
    let Some(name) = fields.next().filter(|field| !field.is_empty()) else {
        println!("oscomp: malformed libctest executable record");
        return;
    };
    let Some(case) = fields.next().filter(|field| !field.is_empty()) else {
        println!("oscomp: malformed libctest case record");
        return;
    };

    let case_name = core::str::from_utf8(case).unwrap_or("unknown");
    println!("RUN LIBCTEST CASE {}", case_name);
    match run_test_with_argv(name, None, &[case], Some(timeout)) {
        Some(0) => println!("Pass!"),
        Some(status) => println!("FAIL LIBCTEST CASE {} : {}", case_name, status),
        None => println!("FAIL LIBCTEST CASE {} : timeout", case_name),
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
            b'A' => {
                ran = true;
                run_argv_record(&record[1..]);
            }
            b'C' => {
                ran = true;
                run_libctest_record(&record[1..]);
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
