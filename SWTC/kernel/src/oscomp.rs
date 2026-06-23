//! Adapter for the OS competition's read-only EXT4 test disk.

use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

use crate::{
    driver::BLOCK_DEVICE,
    fs::{File, Inode, InodeMode, FILE_SYSTEM_MANAGER},
    println,
};
use xmas_elf::{
    program::{SegmentData, Type},
    ElfFile,
};

const SECTOR_SIZE: usize = 512;
const EXT4_MAGIC: u16 = 0xef53;
const EXT4_ROOT_INO: u32 = 2;
const EXT4_EXTENTS_FL: u32 = 0x0008_0000;
const EXT4_EXTENT_MAGIC: u16 = 0xf30a;
const EXT4_S_IFREG: u16 = 0x8000;
const EXT4_MODE_TYPE_MASK: u16 = 0xf000;
const MAX_BLOCK_SIZE: usize = 4096;
const INODE_SIZE: usize = 160;
const GROUP_DESC_SIZE: usize = 64;
const MAX_TEST_FILE_SIZE: usize = 16 * 1024 * 1024;
const MAX_SCRIPT_DEPTH: usize = 4;
const QUEUE_FILE: &str = "oscomp-queue";
const MAX_BASIC_COMMANDS: usize = 32;
const LIBCTEST_TIMEOUT_MS: usize = 3_000;
const MAX_LIBCTEST_CASES: usize = 107;
const LTP_TIMEOUT_MS: usize = 3_000;
const LMBENCH_TIMEOUT_MS: usize = 10_000;
const LUA_RESOURCES: &[&str] = &[
    "test.sh",
    "date.lua",
    "file_io.lua",
    "max_min.lua",
    "random.lua",
    "remove.lua",
    "round_num.lua",
    "sin30.lua",
    "sort.lua",
    "strings.lua",
];
const LMBENCH_COMMANDS: &[&[&str]] = &[
    &["lat_syscall", "-P", "1", "-W", "1", "-N", "10", "null"],
    &["lat_syscall", "-P", "1", "-W", "1", "-N", "10", "read"],
    &["lat_syscall", "-P", "1", "-W", "1", "-N", "10", "write"],
    &[
        "lat_syscall",
        "-P",
        "1",
        "-W",
        "1",
        "-N",
        "10",
        "stat",
        "/var/tmp/lmbench",
    ],
    &[
        "lat_syscall",
        "-P",
        "1",
        "-W",
        "1",
        "-N",
        "10",
        "fstat",
        "/var/tmp/lmbench",
    ],
    &[
        "lat_syscall",
        "-P",
        "1",
        "-W",
        "1",
        "-N",
        "10",
        "open",
        "/var/tmp/lmbench",
    ],
    &[
        "lat_select",
        "-n",
        "100",
        "-P",
        "1",
        "-W",
        "1",
        "-N",
        "10",
        "file",
    ],
    &["lat_sig", "-P", "1", "-W", "1", "-N", "10", "install"],
    &["lat_sig", "-P", "1", "-W", "1", "-N", "10", "catch"],
];
const LTP_ALLOWLIST: &[&str] = &[
    "getpid01",
    "getpid02",
    "getppid01",
    "getuid01",
    "geteuid01",
    "getgid03",
    "getegid02",
    "gettid01",
    "time01",
    "uname01",
    "gettimeofday01",
    "getpagesize01",
];
const LIBCTEST_ALLOWLIST: &[&str] = &[
    "argv",
    "basename",
    "clocale_mbfuncs",
    "clock_gettime",
    "daemon_failure",
    "dirname",
    "dn_expand_empty",
    "dn_expand_ptr_0",
    "env",
    "fdopen",
    "fgets_eof",
    "fgetwc_buffering",
    "fflush_exit",
    "fnmatch",
    "fpclassify_invalid_ld80",
    "fscanf",
    "ftello_unflushed_append",
    "fwscanf",
    "getpwnam_r_crash",
    "getpwnam_r_errno",
    "iconv_open",
    "iconv_roundtrips",
    "inet_ntop_v4mapped",
    "inet_pton",
    "inet_pton_empty_last_field",
    "iswspace_null",
    "lrand48_signextend",
    "lseek_large",
    "malloc_0",
    "mbc",
    "mbsrtowcs_overflow",
    "memmem_oob",
    "memmem_oob_read",
    "memstream",
    "mkdtemp_failure",
    "mkstemp_failure",
    "printf_1e9_oob",
    "printf_fmt_g_round",
    "printf_fmt_g_zeros",
    "printf_fmt_n",
    "pthread_cancel",
    "pthread_cancel_points",
    "pthread_cancel_sem_wait",
    "pthread_cond",
    "pthread_cond_smasher",
    "pthread_condattr_setclock",
    "pthread_exit_cancel",
    "pthread_once_deadlock",
    "pthread_robust_detach",
    "pthread_rwlock_ebusy",
    "pthread_tsd",
    "putenv_doublefree",
    "qsort",
    "random",
    "regex_backref_0",
    "regex_bracket_icase",
    "regex_ere_backref",
    "regex_escaped_high_byte",
    "regex_negated_range",
    "regexec_nosub",
    "rewind_clear_error",
    "rlimit_open_files",
    "scanf_bytes_consumed",
    "scanf_match_literal_eof",
    "scanf_nullbyte_char",
    "search_hsearch",
    "search_insque",
    "search_lsearch",
    "search_tsearch",
    "setjmp",
    "setvbuf_unget",
    "sigprocmask_internal",
    "snprintf",
    "socket",
    "sscanf_eof",
    "sscanf",
    "sscanf_long",
    "stat",
    "statvfs",
    "strftime",
    "string",
    "string_memcpy",
    "string_memmem",
    "string_memset",
    "string_strchr",
    "string_strcspn",
    "string_strstr",
    "strptime",
    "strtod",
    "strtod_simple",
    "strtof",
    "strtol",
    "strtold",
    "strverscmp",
    "swprintf",
    "syscall_sign_extend",
    "tgmath",
    "time",
    "tls_align",
    "udiv",
    "ungetc",
    "uselocale_0",
    "utime",
    "wcstol",
    "wcsncpy_read_overflow",
    "wcsstr",
    "wcsstr_false_negative",
];

#[derive(Clone, Copy)]
struct Ext4 {
    block_size: usize,
    blocks_per_group: u32,
    inodes_per_group: u32,
    inode_size: usize,
    group_desc_size: usize,
}

#[derive(Clone, Copy)]
struct InodeInfo {
    inode_no: u32,
    mode: u16,
    size: u64,
}

struct BasicCommand {
    executable_path: String,
}

#[derive(Clone, Copy)]
enum BasicFlavor {
    Glibc,
    Musl,
    Root,
}

struct BasicPlan {
    commands: Vec<BasicCommand>,
    start_marker: String,
    end_marker: String,
    group_dir: String,
    flavor: BasicFlavor,
}

/// Read the official basic scripts and stage isolated glibc/musl queues in tmpfs.
pub fn init() {
    let fs = match read_superblock() {
        Ok(fs) => fs,
        Err(message) => {
            println!("oscomp: {}", message);
            return;
        }
    };

    let candidates: [(&str, &[&[u8]], &str, BasicFlavor); 2] = [
        (
            "glibc/basic_testcode.sh",
            &[b"glibc", b"basic_testcode.sh"],
            "oscomp-glibc",
            BasicFlavor::Glibc,
        ),
        (
            "musl/basic_testcode.sh",
            &[b"musl", b"basic_testcode.sh"],
            "oscomp-musl",
            BasicFlavor::Musl,
        ),
    ];

    let mut queue = Vec::new();
    let mut found_named_group = false;
    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (label, path, group_dir, flavor) in candidates {
        if let Ok(Some(info)) = lookup_path(&fs, path) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                found_named_group = true;
                let plan = match build_basic_plan(&fs, label, group_dir, flavor) {
                    Ok(plan) => plan,
                    Err(message) => {
                        println!("oscomp: cannot parse {}: {}", label, message);
                        continue;
                    }
                };
                if let Err(message) = install_plan(&fs, &plan, &mut queue) {
                    println!("oscomp: cannot stage {}: {}", label, message);
                    continue;
                }

                println!("oscomp: found official basic script {}", label);
                println!(
                    "oscomp: staged {} basic commands for {}",
                    plan.commands.len(),
                    group_dir
                );
                installed_groups += 1;
                installed_commands += plan.commands.len();
            }
        }
    }

    if !found_named_group {
        if let Ok(Some(info)) = lookup_path(&fs, &[b"basic_testcode.sh"]) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                match build_basic_plan(&fs, "basic_testcode.sh", "oscomp-basic", BasicFlavor::Root)
                    .and_then(|plan| {
                        let command_count = plan.commands.len();
                        install_plan(&fs, &plan, &mut queue)?;
                        installed_commands += command_count;
                        installed_groups += 1;
                        Ok(())
                    }) {
                    Ok(()) => println!("oscomp: found official basic script basic_testcode.sh"),
                    Err(message) => println!("oscomp: cannot stage basic_testcode.sh: {}", message),
                }
            }
        }
    }

    let (busybox_groups, busybox_commands) = install_busybox_groups(&fs, &mut queue);
    installed_groups += busybox_groups;
    installed_commands += busybox_commands;

    let (lua_groups, lua_commands) = install_lua_groups(&fs, &mut queue);
    installed_groups += lua_groups;
    installed_commands += lua_commands;

    let (libcbench_groups, libcbench_commands) = install_libcbench_groups(&fs, &mut queue);
    installed_groups += libcbench_groups;
    installed_commands += libcbench_commands;

    let (libctest_groups, libctest_commands) = install_libctest_groups(&fs, &mut queue);
    installed_groups += libctest_groups;
    installed_commands += libctest_commands;

    let (ltp_groups, ltp_commands) = install_ltp_groups(&fs, &mut queue);
    installed_groups += ltp_groups;
    installed_commands += ltp_commands;

    let (lmbench_groups, lmbench_commands) = install_lmbench_groups(&fs, &mut queue);
    installed_groups += lmbench_groups;
    installed_commands += lmbench_commands;

    if installed_groups == 0 {
        println!("oscomp: official script not found or no runnable group");
        return;
    }

    if let Err(message) = install_tmpfs_file_path(QUEUE_FILE, &queue) {
        println!("oscomp: cannot install basic queue: {}", message);
        return;
    }
    println!(
        "oscomp: staged {} test groups with {} commands",
        installed_groups, installed_commands
    );
}

fn build_basic_plan(
    fs: &Ext4,
    script_path: &str,
    group_dir: &str,
    flavor: BasicFlavor,
) -> Result<BasicPlan, &'static str> {
    let script = read_text_file(fs, script_path)?;
    let start_marker = find_group_marker(&script, "START")
        .unwrap_or_else(|| "#### OS COMP TEST GROUP START basic ####".to_string());
    let end_marker = find_group_marker(&script, "END")
        .unwrap_or_else(|| "#### OS COMP TEST GROUP END basic ####".to_string());
    let commands = find_commands(fs, script_path, 0)?;
    Ok(BasicPlan {
        commands,
        start_marker,
        end_marker,
        group_dir: group_dir.to_string(),
        flavor,
    })
}

fn find_commands(
    fs: &Ext4,
    script_path: &str,
    depth: usize,
) -> Result<Vec<BasicCommand>, &'static str> {
    if depth >= MAX_SCRIPT_DEPTH {
        return Err("nested script limit reached");
    }
    let script = read_text_file(fs, script_path)?;
    let mut cwd = parent_path(script_path);

    if let Some(tests) = quoted_assignment(&script, "tests") {
        let mut commands = Vec::new();
        for test in tests.split_whitespace().take(MAX_BASIC_COMMANDS) {
            if should_skip_basic_command(test) {
                println!("oscomp: skip known unsafe basic command {}", test);
                continue;
            }
            let executable_path = resolve_path(&cwd, test);
            let info = lookup_path_str(fs, &executable_path)?.ok_or("basic ELF not found")?;
            if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
                return Err("basic command is not a regular file");
            }
            commands.push(BasicCommand { executable_path });
        }
        if !commands.is_empty() {
            return Ok(commands);
        }
    }

    for raw_line in script.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line == "do"
            || line == "done"
            || line.starts_with("for ")
        {
            continue;
        }
        if let Some(path) = line.strip_prefix("cd ") {
            cwd = resolve_path(&cwd, trim_shell_quotes(path.trim()));
            continue;
        }

        let argv = split_shell_words(line);
        if argv.is_empty() {
            continue;
        }
        let executable = argv[0].as_str();
        if executable == "echo"
            || executable.ends_with("/busybox")
            || executable == "busybox"
            || executable.starts_with('$')
        {
            continue;
        }

        let resolved = resolve_path(&cwd, executable);
        if executable.ends_with(".sh") {
            return find_commands(fs, &resolved, depth + 1);
        }
        if let Ok(Some(info)) = lookup_path_str(fs, &resolved) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                return Ok(vec![BasicCommand {
                    executable_path: resolved,
                }]);
            }
        }
    }

    Err("no executable command found")
}

fn should_skip_basic_command(_name: &str) -> bool {
    false
}

fn install_busybox_groups(fs: &Ext4, queue: &mut Vec<u8>) -> (usize, usize) {
    let candidates = [
        ("glibc/busybox_testcode.sh", "oscomp-busybox-glibc"),
        ("musl/busybox_testcode.sh", "oscomp-busybox-musl"),
        ("busybox_testcode.sh", "oscomp-busybox"),
    ];

    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (script_path, group_dir) in candidates {
        let Ok(Some(info)) = lookup_path_str(fs, script_path) else {
            continue;
        };
        if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
            continue;
        }
        match install_busybox_group(fs, script_path, group_dir, queue) {
            Ok(()) => {
                println!("oscomp: found official busybox script {}", script_path);
                installed_groups += 1;
                installed_commands += 1;
            }
            Err(message) => println!("oscomp: cannot stage {}: {}", script_path, message),
        }
    }

    (installed_groups, installed_commands)
}

fn install_busybox_group(
    fs: &Ext4,
    script_path: &str,
    group_dir: &str,
    queue: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let source_dir = parent_path(script_path);
    let busybox_path = resolve_path(&source_dir, "busybox");
    let command_path = resolve_path(&source_dir, "busybox_cmd.txt");
    let script = read_file(fs, script_path)?;
    let commands = read_file(fs, &command_path)?;
    let busybox = read_file(fs, &busybox_path)?;
    if busybox.get(..4) != Some(b"\x7fELF") {
        return Err("busybox is not an ELF file");
    }

    install_tmpfs_file_path("busybox", &busybox)?;
    install_tmpfs_dir_path(group_dir)?;
    install_tmpfs_file_path(&alloc::format!("{}/busybox", group_dir), &busybox)?;
    install_tmpfs_file_path(&alloc::format!("{}/ls", group_dir), &busybox)?;
    install_tmpfs_file_path(
        &alloc::format!("{}/busybox_testcode.sh", group_dir),
        &script,
    )?;
    install_tmpfs_file_path(&alloc::format!("{}/busybox_cmd.txt", group_dir), &commands)?;

    push_queue_record(queue, b'G', &alloc::format!("/{}\t", group_dir));
    push_queue_record(queue, b'X', "busybox_testcode.sh");
    Ok(())
}

fn install_lua_groups(fs: &Ext4, queue: &mut Vec<u8>) -> (usize, usize) {
    let candidates = [
        ("glibc/lua_testcode.sh", "oscomp-lua-glibc"),
        ("musl/lua_testcode.sh", "oscomp-lua-musl"),
        ("lua_testcode.sh", "oscomp-lua"),
    ];

    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (script_path, group_dir) in candidates {
        let Ok(Some(info)) = lookup_path_str(fs, script_path) else {
            continue;
        };
        if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
            continue;
        }
        match install_lua_group(fs, script_path, group_dir, queue) {
            Ok(()) => {
                println!("oscomp: found official lua script {}", script_path);
                installed_groups += 1;
                installed_commands += 1;
            }
            Err(message) => println!("oscomp: cannot stage {}: {}", script_path, message),
        }
    }

    (installed_groups, installed_commands)
}

fn install_lua_group(
    fs: &Ext4,
    script_path: &str,
    group_dir: &str,
    queue: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let source_dir = parent_path(script_path);
    let lua_path = resolve_path(&source_dir, "lua");
    let busybox_path = resolve_path(&source_dir, "busybox");
    let script = read_file(fs, script_path)?;
    let lua = read_file(fs, &lua_path)?;
    let busybox = read_file(fs, &busybox_path)?;
    if lua.get(..4) != Some(b"\x7fELF") {
        return Err("lua is not an ELF file");
    }
    if busybox.get(..4) != Some(b"\x7fELF") {
        return Err("busybox is not an ELF file");
    }

    install_tmpfs_file_path("busybox", &busybox)?;
    install_tmpfs_dir_path(group_dir)?;
    install_tmpfs_file_path(&alloc::format!("{}/busybox", group_dir), &busybox)?;
    install_tmpfs_file_path(&alloc::format!("{}/lua", group_dir), &lua)?;
    install_tmpfs_file_path(&alloc::format!("{}/lua_testcode.sh", group_dir), &script)?;
    for resource in LUA_RESOURCES {
        let path = resolve_path(&source_dir, resource);
        install_tmpfs_file_path(
            &alloc::format!("{}/{}", group_dir, resource),
            &read_file(fs, &path)?,
        )?;
    }

    push_queue_record(queue, b'G', &alloc::format!("/{}\t", group_dir));
    push_queue_record(queue, b'X', "lua_testcode.sh");
    Ok(())
}

fn install_libcbench_groups(fs: &Ext4, queue: &mut Vec<u8>) -> (usize, usize) {
    let candidates = [
        ("glibc/libcbench_testcode.sh", "oscomp-libcbench-glibc"),
        ("musl/libcbench_testcode.sh", "oscomp-libcbench-musl"),
        ("libcbench_testcode.sh", "oscomp-libcbench"),
    ];

    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (script_path, group_dir) in candidates {
        let Ok(Some(info)) = lookup_path_str(fs, script_path) else {
            continue;
        };
        if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
            continue;
        }
        match install_libcbench_group(fs, script_path, group_dir, queue) {
            Ok(()) => {
                println!("oscomp: found official libcbench script {}", script_path);
                installed_groups += 1;
                installed_commands += 1;
            }
            Err(message) => println!("oscomp: cannot stage {}: {}", script_path, message),
        }
    }

    (installed_groups, installed_commands)
}

fn install_libcbench_group(
    fs: &Ext4,
    script_path: &str,
    group_dir: &str,
    queue: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let source_dir = parent_path(script_path);
    let busybox_path = resolve_path(&source_dir, "busybox");
    let bench_path = resolve_path(&source_dir, "libc-bench");
    let script = read_file(fs, script_path)?;
    let busybox = read_file(fs, &busybox_path)?;
    let bench = read_file(fs, &bench_path)?;
    if busybox.get(..4) != Some(b"\x7fELF") {
        return Err("busybox is not an ELF file");
    }
    if bench.get(..4) != Some(b"\x7fELF") {
        return Err("libc-bench is not an ELF file");
    }

    install_tmpfs_file_path("busybox", &busybox)?;
    install_tmpfs_dir_path(group_dir)?;
    install_tmpfs_file_path(&alloc::format!("{}/busybox", group_dir), &busybox)?;
    install_tmpfs_file_path(&alloc::format!("{}/libc-bench", group_dir), &bench)?;
    install_tmpfs_file_path(
        &alloc::format!("{}/libcbench_testcode.sh", group_dir),
        &script,
    )?;

    push_queue_record(queue, b'G', &alloc::format!("/{}\t", group_dir));
    push_queue_record(queue, b'X', "libcbench_testcode.sh");
    Ok(())
}

fn install_libctest_groups(fs: &Ext4, queue: &mut Vec<u8>) -> (usize, usize) {
    let candidates = [
        (
            "musl/libctest_testcode.sh",
            "oscomp-libctest-musl",
            "libctest-musl",
        ),
        (
            "musl/libctest/libctest_testcode.sh",
            "oscomp-libctest-musl",
            "libctest-musl",
        ),
    ];

    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (script_path, group_dir, marker_name) in candidates {
        let Ok(Some(info)) = lookup_path_str(fs, script_path) else {
            continue;
        };
        if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
            continue;
        }
        match install_libctest_group(fs, script_path, group_dir, marker_name, queue) {
            Ok(command_count) => {
                println!("oscomp: found official libctest script {}", script_path);
                installed_groups += 1;
                installed_commands += command_count;
            }
            Err(message) => println!("oscomp: cannot stage {}: {}", script_path, message),
        }
    }

    (installed_groups, installed_commands)
}

fn install_libctest_group(
    fs: &Ext4,
    script_path: &str,
    group_dir: &str,
    marker_name: &str,
    queue: &mut Vec<u8>,
) -> Result<usize, &'static str> {
    let source_dir = parent_path(script_path);
    let run_static_path = find_first_regular_path(
        fs,
        &[
            resolve_path(&source_dir, "run-static.sh"),
            resolve_path(&source_dir, "run-static"),
            resolve_path(&source_dir, "libctest/run-static.sh"),
            resolve_path(&source_dir, "libctest/run-static"),
            resolve_path(&source_dir, "libc-test/run-static.sh"),
            resolve_path(&source_dir, "libc-test/run-static"),
        ],
    )?;
    let entry_path = find_first_regular_path(
        fs,
        &[
            resolve_path(&source_dir, "entry-static.exe"),
            resolve_path(&source_dir, "libctest/entry-static.exe"),
            resolve_path(&source_dir, "libc-test/entry-static.exe"),
        ],
    )?;
    let runtest_path = find_optional_regular_path(
        fs,
        &[
            resolve_path(&source_dir, "runtest.exe"),
            resolve_path(&source_dir, "libctest/runtest.exe"),
            resolve_path(&source_dir, "libc-test/runtest.exe"),
        ],
    )?;
    let cases = find_libctest_cases(fs, &run_static_path)?;
    if cases.is_empty() {
        return Err("no allowed libctest case found");
    }

    let script = read_file(fs, script_path)?;
    let run_static = read_file(fs, &run_static_path)?;
    let entry = read_file(fs, &entry_path)?;
    if entry.get(..4) != Some(b"\x7fELF") {
        return Err("entry-static.exe is not an ELF file");
    }
    let runtest = match &runtest_path {
        Some(path) => {
            let data = read_file(fs, path)?;
            if data.get(..4) != Some(b"\x7fELF") {
                return Err("runtest.exe is not an ELF file");
            }
            Some(data)
        }
        None => None,
    };

    let (start_marker, end_marker) = match core::str::from_utf8(&script) {
        Ok(script) => (
            find_group_marker(script, "START").unwrap_or_else(|| {
                alloc::format!("#### OS COMP TEST GROUP START {} ####", marker_name)
            }),
            find_group_marker(script, "END").unwrap_or_else(|| {
                alloc::format!("#### OS COMP TEST GROUP END {} ####", marker_name)
            }),
        ),
        Err(_) => (
            alloc::format!("#### OS COMP TEST GROUP START {} ####", marker_name),
            alloc::format!("#### OS COMP TEST GROUP END {} ####", marker_name),
        ),
    };

    install_tmpfs_dir_path(group_dir)?;
    install_tmpfs_file_path(
        &alloc::format!("{}/libctest_testcode.sh", group_dir),
        &script,
    )?;
    install_tmpfs_file_path(&alloc::format!("{}/run-static.sh", group_dir), &run_static)?;
    install_tmpfs_file_path(&alloc::format!("{}/entry-static.exe", group_dir), &entry)?;
    if let Some(runtest) = &runtest {
        install_tmpfs_file_path(&alloc::format!("{}/runtest.exe", group_dir), runtest)?;
    }

    push_queue_record(
        queue,
        b'G',
        &alloc::format!("/{}\t{}", group_dir, start_marker),
    );
    for case in &cases {
        if runtest.is_some() {
            push_libctest_runtest_record(queue, LIBCTEST_TIMEOUT_MS, case);
        } else {
            push_libctest_record(queue, LIBCTEST_TIMEOUT_MS, "entry-static.exe", case);
        }
    }
    push_queue_record(queue, b'E', &end_marker);
    Ok(cases.len())
}

fn find_libctest_cases(fs: &Ext4, run_static_path: &str) -> Result<Vec<String>, &'static str> {
    let script = read_text_file(fs, run_static_path)?;
    let mut cases = Vec::new();
    for raw_line in script.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        for word in split_shell_words(line) {
            if let Some((_, value)) = word.split_once('=') {
                push_libctest_words(value, &mut cases);
            } else {
                push_libctest_words(&word, &mut cases);
            }
            if cases.len() >= MAX_LIBCTEST_CASES {
                return Ok(cases);
            }
        }
    }
    Ok(cases)
}

fn push_libctest_words(value: &str, cases: &mut Vec<String>) {
    for word in value.split_whitespace() {
        let candidate = normalize_libctest_case(word);
        if is_allowed_libctest_case(&candidate) && !cases.iter().any(|known| known == &candidate) {
            cases.push(candidate);
            if cases.len() >= MAX_LIBCTEST_CASES {
                return;
            }
        }
    }
}

fn normalize_libctest_case(value: &str) -> String {
    let trimmed =
        trim_shell_quotes(value).trim_matches(|ch| matches!(ch, ';' | ',' | '(' | ')' | '[' | ']'));
    let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    let name = name
        .strip_suffix("-static.exe")
        .or_else(|| name.strip_suffix(".exe"))
        .or_else(|| name.strip_suffix("-static"))
        .unwrap_or(name);
    name.to_string()
}

fn is_allowed_libctest_case(name: &str) -> bool {
    LIBCTEST_ALLOWLIST.iter().any(|allowed| *allowed == name)
}

fn install_ltp_groups(fs: &Ext4, queue: &mut Vec<u8>) -> (usize, usize) {
    let candidates = [
        (
            "glibc/ltp/testcases/bin",
            "oscomp-ltp-glibc",
            "ltp-glibc",
            BasicFlavor::Glibc,
        ),
        (
            "musl/ltp/testcases/bin",
            "oscomp-ltp-musl",
            "ltp-musl",
            BasicFlavor::Musl,
        ),
    ];

    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (source_dir, group_dir, marker_name, flavor) in candidates {
        match install_ltp_group(fs, source_dir, group_dir, marker_name, flavor, queue) {
            Ok(command_count) if command_count > 0 => {
                println!(
                    "oscomp: found official ltp dir {} with {} cases",
                    source_dir, command_count
                );
                installed_groups += 1;
                installed_commands += command_count;
            }
            Ok(_) => {}
            Err(message) => println!("oscomp: cannot stage {}: {}", source_dir, message),
        }
    }

    (installed_groups, installed_commands)
}

fn install_ltp_group(
    fs: &Ext4,
    source_dir: &str,
    group_dir: &str,
    marker_name: &str,
    flavor: BasicFlavor,
    queue: &mut Vec<u8>,
) -> Result<usize, &'static str> {
    let Ok(Some(info)) = lookup_path_str(fs, source_dir) else {
        return Ok(0);
    };
    if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
        return Err("ltp source path is not a directory");
    }

    install_tmpfs_dir_path(group_dir)?;
    let mut interp_paths = Vec::new();
    let mut staged_cases = Vec::new();
    for case in LTP_ALLOWLIST {
        let source_path = resolve_path(source_dir, case);
        let Ok(Some(info)) = lookup_path_str(fs, &source_path) else {
            continue;
        };
        if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
            continue;
        }
        let elf = read_file(fs, &source_path)?;
        if elf.get(..4) != Some(b"\x7fELF") {
            continue;
        }
        install_tmpfs_file_path(&alloc::format!("{}/{}", group_dir, case), &elf)?;
        remember_interp(group_dir, &elf, &mut interp_paths)?;
        staged_cases.push(*case);
    }

    if staged_cases.is_empty() {
        return Ok(0);
    }
    if !interp_paths.is_empty() {
        install_group_runtime(fs, flavor, group_dir, &interp_paths)?;
    }
    install_tmpfs_dir_path(&alloc::format!("{}/tmp", group_dir))?;

    push_queue_record(
        queue,
        b'G',
        &alloc::format!(
            "/{}\t#### OS COMP TEST GROUP START {} ####",
            group_dir,
            marker_name
        ),
    );
    for case in &staged_cases {
        push_ltp_record(queue, LTP_TIMEOUT_MS, case);
    }
    push_queue_record(
        queue,
        b'E',
        &alloc::format!("#### OS COMP TEST GROUP END {} ####", marker_name),
    );
    Ok(staged_cases.len())
}

fn find_optional_regular_path(fs: &Ext4, paths: &[String]) -> Result<Option<String>, &'static str> {
    for path in paths {
        if let Ok(Some(info)) = lookup_path_str(fs, path) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                return Ok(Some(path.clone()));
            }
        }
    }
    Ok(None)
}

fn install_lmbench_groups(fs: &Ext4, queue: &mut Vec<u8>) -> (usize, usize) {
    let candidates = [
        (
            "glibc/lmbench_testcode.sh",
            "oscomp-lmbench-glibc",
            "lmbench-glibc",
            BasicFlavor::Glibc,
        ),
        (
            "musl/lmbench_testcode.sh",
            "oscomp-lmbench-musl",
            "lmbench-musl",
            BasicFlavor::Musl,
        ),
    ];

    let mut installed_groups = 0usize;
    let mut installed_commands = 0usize;
    for (script_path, group_dir, marker_name, flavor) in candidates {
        let Ok(Some(info)) = lookup_path_str(fs, script_path) else {
            continue;
        };
        if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
            continue;
        }
        match install_lmbench_group(fs, script_path, group_dir, marker_name, flavor, queue) {
            Ok(command_count) => {
                println!("oscomp: found official lmbench script {}", script_path);
                installed_groups += 1;
                installed_commands += command_count;
            }
            Err(message) => println!("oscomp: cannot stage {}: {}", script_path, message),
        }
    }

    (installed_groups, installed_commands)
}

fn install_lmbench_group(
    fs: &Ext4,
    script_path: &str,
    group_dir: &str,
    marker_name: &str,
    flavor: BasicFlavor,
    queue: &mut Vec<u8>,
) -> Result<usize, &'static str> {
    let source_dir = parent_path(script_path);
    let lmbench_path = find_first_regular_path(
        fs,
        &[
            resolve_path(&source_dir, "lmbench_all"),
            resolve_path(&source_dir, "bin/lmbench_all"),
            resolve_path(&source_dir, "lmbench/bin/lmbench_all"),
        ],
    )?;
    let lmbench = read_file(fs, &lmbench_path)?;
    if lmbench.get(..4) != Some(b"\x7fELF") {
        return Err("lmbench_all is not an ELF file");
    }

    let script = read_file(fs, script_path)?;
    let (start_marker, end_marker) = match core::str::from_utf8(&script) {
        Ok(script) => (
            find_group_marker(script, "START").unwrap_or_else(|| {
                alloc::format!("#### OS COMP TEST GROUP START {} ####", marker_name)
            }),
            find_group_marker(script, "END").unwrap_or_else(|| {
                alloc::format!("#### OS COMP TEST GROUP END {} ####", marker_name)
            }),
        ),
        Err(_) => (
            alloc::format!("#### OS COMP TEST GROUP START {} ####", marker_name),
            alloc::format!("#### OS COMP TEST GROUP END {} ####", marker_name),
        ),
    };
    install_tmpfs_dir_path(group_dir)?;
    install_tmpfs_file_path(
        &alloc::format!("{}/lmbench_testcode.sh", group_dir),
        &script,
    )?;
    install_tmpfs_file_path(&alloc::format!("{}/lmbench_all", group_dir), &lmbench)?;
    // Keep this in sync with sys_readlinkat's /proc/self/exe compatibility path.
    install_tmpfs_file_path("lmbench_all", &lmbench)?;

    let mut interp_paths = Vec::new();
    remember_interp(group_dir, &lmbench, &mut interp_paths)?;
    install_optional_lmbench_resource(fs, &source_dir, group_dir, "hello", &mut interp_paths)?;
    let lat_sig_path = resolve_path(&source_dir, "lat_sig");
    match lookup_path_str(fs, &lat_sig_path) {
        Ok(Some(info)) if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG => {
            let lat_sig = read_file(fs, &lat_sig_path)?;
            install_tmpfs_file_path(&alloc::format!("{}/lat_sig", group_dir), &lat_sig)?;
            install_tmpfs_file_path("lat_sig", &lat_sig)?;
            if lat_sig.get(..4) == Some(b"\x7fELF") {
                remember_interp(group_dir, &lat_sig, &mut interp_paths)?;
            }
        }
        _ => {
            install_tmpfs_file_path(&alloc::format!("{}/lat_sig", group_dir), &lmbench)?;
            install_tmpfs_file_path("lat_sig", &lmbench)?;
        }
    }

    if !interp_paths.is_empty() {
        install_group_runtime(fs, flavor, group_dir, &interp_paths)?;
    }

    install_tmpfs_dir_path("var/tmp")?;
    install_tmpfs_dir_path("tmp")?;
    install_tmpfs_file_path("var/tmp/lmbench", b"")?;
    install_tmpfs_file_path("var/tmp/XXX", b"")?;
    if let Ok(hello) = read_file(fs, &resolve_path(&source_dir, "hello")) {
        install_tmpfs_file_path("tmp/hello", &hello)?;
    }

    push_queue_record(
        queue,
        b'G',
        &alloc::format!("/{}\t{}", group_dir, start_marker),
    );
    for args in LMBENCH_COMMANDS {
        push_argv_record(queue, LMBENCH_TIMEOUT_MS, "lmbench_all", args);
    }
    push_queue_record(queue, b'E', &end_marker);
    Ok(LMBENCH_COMMANDS.len())
}

fn find_first_regular_path(fs: &Ext4, paths: &[String]) -> Result<String, &'static str> {
    for path in paths {
        if let Ok(Some(info)) = lookup_path_str(fs, path) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                return Ok(path.clone());
            }
        }
    }
    Err("required executable not found")
}

fn remember_interp(
    group_dir: &str,
    elf: &[u8],
    interp_paths: &mut Vec<String>,
) -> Result<(), &'static str> {
    if let Some(interp) = elf_interp_path(elf)? {
        if !interp_paths.iter().any(|known| known == &interp) {
            println!("oscomp: {} PT_INTERP {}", group_dir, interp);
            interp_paths.push(interp);
        }
    }
    Ok(())
}

fn install_optional_lmbench_resource(
    fs: &Ext4,
    source_dir: &str,
    group_dir: &str,
    name: &str,
    interp_paths: &mut Vec<String>,
) -> Result<(), &'static str> {
    let path = resolve_path(source_dir, name);
    let Ok(Some(info)) = lookup_path_str(fs, &path) else {
        return Ok(());
    };
    if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
        return Ok(());
    }
    let data = read_file(fs, &path)?;
    install_tmpfs_file_path(&alloc::format!("{}/{}", group_dir, name), &data)?;
    if data.get(..4) == Some(b"\x7fELF") {
        remember_interp(group_dir, &data, interp_paths)?;
    }
    Ok(())
}

fn install_plan(fs: &Ext4, plan: &BasicPlan, queue: &mut Vec<u8>) -> Result<(), &'static str> {
    let mut group_queue = Vec::new();
    let group_path = alloc::format!("/{}", plan.group_dir);
    push_queue_record(
        &mut group_queue,
        b'G',
        &alloc::format!("{}\t{}", group_path, plan.start_marker),
    );
    install_tmpfs_dir_path(&plan.group_dir)?;

    let mut needs_runtime = false;
    let mut interp_paths = Vec::new();
    for command in &plan.commands {
        let name = file_name(&command.executable_path)?;
        let staged_name = alloc::format!("oscomp-basic-{}-elf", name);
        let staged_path = alloc::format!("{}/{}", plan.group_dir, staged_name);
        let elf = read_file(fs, &command.executable_path)?;
        if elf.get(..4) != Some(b"\x7fELF") {
            return Err("basic command is not an ELF file");
        }
        if let Some(interp) = elf_interp_path(&elf)? {
            needs_runtime = true;
            if !interp_paths.iter().any(|known| known == &interp) {
                println!("oscomp: {} PT_INTERP {}", plan.group_dir, interp);
                interp_paths.push(interp);
            }
        }
        install_tmpfs_file_path(&staged_path, &elf)?;
        push_queue_record(&mut group_queue, b'X', &staged_name);
    }

    let basic_dir = plan
        .commands
        .first()
        .map(|command| parent_path(&command.executable_path))
        .ok_or("empty basic command queue")?;
    install_optional_group_resource(fs, &basic_dir, &plan.group_dir, "test_echo")?;
    install_optional_group_resource(fs, &basic_dir, &plan.group_dir, "text.txt")?;
    install_tmpfs_dir_path(&alloc::format!("{}/mnt", plan.group_dir))?;

    if needs_runtime {
        install_group_runtime(fs, plan.flavor, &plan.group_dir, &interp_paths)?;
    }

    push_queue_record(&mut group_queue, b'E', &plan.end_marker);
    queue.extend_from_slice(&group_queue);
    Ok(())
}

fn push_queue_record(queue: &mut Vec<u8>, kind: u8, payload: &str) {
    queue.push(kind);
    queue.extend_from_slice(payload.as_bytes());
    queue.push(0);
}

fn push_argv_record(queue: &mut Vec<u8>, timeout_ms: usize, executable: &str, args: &[&str]) {
    let mut payload = alloc::format!("{}\t{}", timeout_ms, executable);
    for arg in args {
        payload.push('\t');
        payload.push_str(arg);
    }
    push_queue_record(queue, b'A', &payload);
}

fn push_libctest_record(queue: &mut Vec<u8>, timeout_ms: usize, executable: &str, case: &str) {
    let payload = alloc::format!("{}\t{}\t{}", timeout_ms, executable, case);
    push_queue_record(queue, b'C', &payload);
}

fn push_libctest_runtest_record(queue: &mut Vec<u8>, timeout_ms: usize, case: &str) {
    let payload = alloc::format!(
        "{}\truntest.exe\t-w\tentry-static.exe\t{}",
        timeout_ms,
        case
    );
    push_queue_record(queue, b'A', &payload);
}

fn push_ltp_record(queue: &mut Vec<u8>, timeout_ms: usize, case: &str) {
    let payload = alloc::format!("{}\t{}\t{}", timeout_ms, case, case);
    push_queue_record(queue, b'L', &payload);
}

fn elf_interp_path(elf_data: &[u8]) -> Result<Option<String>, &'static str> {
    let elf = ElfFile::new(elf_data).map_err(|_| "invalid ELF")?;
    for index in 0..elf.header.pt2.ph_count() {
        let header = elf
            .program_header(index)
            .map_err(|_| "invalid ELF program header")?;
        if header.get_type().ok() == Some(Type::Interp) {
            let data = match header
                .get_data(&elf)
                .map_err(|_| "invalid ELF interpreter segment")?
            {
                SegmentData::Undefined(data) => data,
                _ => return Err("invalid ELF interpreter segment"),
            };
            let data = data.strip_suffix(&[0]).unwrap_or(data);
            let interp = core::str::from_utf8(data)
                .map_err(|_| "ELF interpreter path is not UTF-8")?
                .trim();
            if interp.is_empty() {
                return Err("empty ELF interpreter path");
            }
            return Ok(Some(interp.to_string()));
        }
    }
    Ok(None)
}

fn install_optional_group_resource(
    fs: &Ext4,
    directory: &str,
    group_dir: &str,
    name: &str,
) -> Result<(), &'static str> {
    let path = resolve_path(directory, name);
    if let Ok(Some(info)) = lookup_path_str(fs, &path) {
        if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
            install_tmpfs_file_path(
                &alloc::format!("{}/{}", group_dir, name),
                &read_file(fs, &path)?,
            )?;
        }
    }
    Ok(())
}

fn install_group_runtime(
    fs: &Ext4,
    flavor: BasicFlavor,
    group_dir: &str,
    interp_paths: &[String],
) -> Result<(), &'static str> {
    match flavor {
        BasicFlavor::Glibc => {
            let loader = read_file(fs, "glibc/lib/ld-linux-riscv64-lp64d.so.1")
                .map_err(|_| "required dynamic runtime file not found")?;
            install_tmpfs_file_path(
                &alloc::format!("{}/lib/ld-linux-riscv64-lp64d.so.1", group_dir),
                &loader,
            )?;
            for interp in interp_paths {
                install_interp_alias(group_dir, interp, &loader)?;
            }
            install_required_disk_file(
                fs,
                "glibc/lib/libc.so",
                &alloc::format!("{}/lib/libc.so", group_dir),
            )?;
            install_required_disk_file(
                fs,
                "glibc/lib/libc.so.6",
                &alloc::format!("{}/lib/libc.so.6", group_dir),
            )?;
            install_optional_disk_file(
                fs,
                "glibc/lib/libm.so",
                &alloc::format!("{}/lib/libm.so", group_dir),
            )?;
            install_optional_disk_file(
                fs,
                "glibc/lib/libm.so.6",
                &alloc::format!("{}/lib/libm.so.6", group_dir),
            )
        }
        BasicFlavor::Musl => {
            let libc = read_first_disk_file(
                fs,
                &[
                    "musl/lib/libc.so",
                    "musl/lib/ld-musl-riscv64-lp64d.so.1",
                    "musl/lib/ld-musl-riscv64-lp64.so.1",
                    "musl/lib/ld-musl-riscv64.so.1",
                    "musl/lib/ld-musl-riscv64-sf.so.1",
                    "lib/ld-musl-riscv64-lp64d.so.1",
                    "lib/ld-musl-riscv64-lp64.so.1",
                    "lib/ld-musl-riscv64.so.1",
                    "lib/ld-musl-riscv64-sf.so.1",
                ],
            )
            .ok_or("required musl runtime not found")?;
            for name in [
                "libc.so",
                "lib/libc.so",
                "lib/ld-musl-riscv64-lp64d.so.1",
                "lib/ld-musl-riscv64-lp64.so.1",
                "lib/ld-musl-riscv64.so.1",
                "lib/ld-musl-riscv64-sf.so.1",
            ] {
                install_tmpfs_file_path(&alloc::format!("{}/{}", group_dir, name), &libc)?;
            }
            for interp in interp_paths {
                install_interp_alias(group_dir, interp, &libc)?;
                if let Ok(name) = file_name(interp) {
                    install_tmpfs_file_path(&alloc::format!("{}/lib/{}", group_dir, name), &libc)?;
                }
            }
            install_tmpfs_file_path(
                &alloc::format!("{}/etc/ld-musl-riscv64.path", group_dir),
                b"/\n/lib\n",
            )?;
            install_tmpfs_file_path(
                &alloc::format!("{}/etc/ld-musl-riscv64-sf.path", group_dir),
                b"/\n/lib\n",
            )
        }
        BasicFlavor::Root => Err("dynamic root basic group has no isolated runtime"),
    }
}

fn install_interp_alias(group_dir: &str, interp: &str, data: &[u8]) -> Result<(), &'static str> {
    let path = interp.trim_start_matches('/');
    if path.is_empty() || path.contains("..") {
        return Err("invalid ELF interpreter path");
    }
    install_tmpfs_file_path(&alloc::format!("{}/{}", group_dir, path), data)
}

fn read_first_disk_file(fs: &Ext4, paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|path| read_file(fs, path).ok())
}

fn install_required_disk_file(
    fs: &Ext4,
    source: &str,
    destination: &str,
) -> Result<(), &'static str> {
    let data = read_file(fs, source).map_err(|_| "required dynamic runtime file not found")?;
    install_tmpfs_file_path(destination, &data)
}

fn install_optional_disk_file(
    fs: &Ext4,
    source: &str,
    destination: &str,
) -> Result<(), &'static str> {
    if lookup_path_str(fs, source)?.is_some() {
        install_tmpfs_file_path(destination, &read_file(fs, source)?)?;
    }
    Ok(())
}

fn install_tmpfs_file_path(path: &str, data: &[u8]) -> Result<(), &'static str> {
    let (parent_path, name) = path
        .trim_matches('/')
        .rsplit_once('/')
        .unwrap_or(("", path.trim_matches('/')));
    if name.is_empty() {
        return Err("invalid tmpfs file path");
    }
    let parent = ensure_tmpfs_dir_path(parent_path)?;
    let inode = match parent
        .lookup(name)
        .map_err(|_| "cannot lookup tmpfs inode")?
    {
        Some(inode) => inode,
        None => parent
            .mknod_v(name, InodeMode::FileREG, None)
            .map_err(|_| "cannot create tmpfs inode")?,
    };
    let file = inode
        .open(inode.clone())
        .map_err(|_| "cannot open tmpfs inode")?;
    let written = file
        .sync_write(data)
        .map_err(|_| "cannot write tmpfs file")?;
    if written != data.len() {
        return Err("short tmpfs write");
    }
    Ok(())
}

fn install_tmpfs_dir_path(path: &str) -> Result<(), &'static str> {
    ensure_tmpfs_dir_path(path).map(|_| ())
}

fn ensure_tmpfs_dir_path(path: &str) -> Result<Arc<dyn Inode>, &'static str> {
    let mut parent = FILE_SYSTEM_MANAGER.root_inode();
    for name in path.split('/').filter(|name| !name.is_empty()) {
        parent = match parent.lookup(name).map_err(|_| "cannot lookup tmpfs dir")? {
            Some(inode) => inode,
            None => parent
                .mkdir_v(name, InodeMode::FileDIR)
                .map_err(|_| "cannot create tmpfs dir")?,
        };
    }
    Ok(parent)
}

fn file_name(path: &str) -> Result<&str, &'static str> {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or("invalid basic path")
}

fn read_text_file(fs: &Ext4, path: &str) -> Result<String, &'static str> {
    String::from_utf8(read_file(fs, path)?).map_err(|_| "script is not UTF-8")
}

fn read_file(fs: &Ext4, path: &str) -> Result<Vec<u8>, &'static str> {
    let info = lookup_path_str(fs, path)?.ok_or("file not found")?;
    if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
        return Err("path is not a regular file");
    }
    let size = usize::try_from(info.size).map_err(|_| "file is too large")?;
    if size > MAX_TEST_FILE_SIZE {
        return Err("file exceeds staging limit");
    }

    let mut inode = [0u8; INODE_SIZE];
    read_inode(fs, info.inode_no, &mut inode)?;
    let mut output = vec![0u8; size];
    if output.is_empty() {
        return Ok(output);
    }

    if le_u32(&inode, 32) & EXT4_EXTENTS_FL != 0 {
        read_extent_file_node(fs, &inode[40..100], &mut output)?;
    } else {
        read_classic_file(fs, &inode[40..100], &mut output)?;
    }
    Ok(output)
}

fn read_extent_file_node(fs: &Ext4, node: &[u8], output: &mut [u8]) -> Result<(), &'static str> {
    if le_u16(node, 0) != EXT4_EXTENT_MAGIC {
        return Err("invalid EXT4 extent header");
    }
    let entries = le_u16(node, 2) as usize;
    let depth = le_u16(node, 6);
    if depth == 0 {
        for index in 0..entries {
            let offset = 12 + index * 12;
            if offset + 12 > node.len() {
                return Err("invalid EXT4 extent entry");
            }
            let logical = le_u32(node, offset) as usize;
            let len = (le_u16(node, offset + 4) & 0x7fff) as usize;
            let physical =
                ((le_u16(node, offset + 6) as u64) << 32) | le_u32(node, offset + 8) as u64;
            for block_index in 0..len {
                copy_file_block(
                    fs,
                    physical + block_index as u64,
                    logical + block_index,
                    output,
                )?;
            }
        }
        return Ok(());
    }

    for index in 0..entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent index");
        }
        let child = ((le_u16(node, offset + 8) as u64) << 32) | le_u32(node, offset + 4) as u64;
        let mut block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, child, &mut block)?;
        read_extent_file_node(fs, &block[..fs.block_size], output)?;
    }
    Ok(())
}

fn read_classic_file(fs: &Ext4, blocks: &[u8], output: &mut [u8]) -> Result<(), &'static str> {
    let mut logical = 0usize;
    for index in 0..12 {
        if logical * fs.block_size >= output.len() {
            return Ok(());
        }
        let block_no = le_u32(blocks, index * 4) as u64;
        if block_no != 0 {
            copy_file_block(fs, block_no, logical, output)?;
        }
        logical += 1;
    }

    if logical * fs.block_size < output.len() {
        let indirect = le_u32(blocks, 12 * 4) as u64;
        if indirect == 0 {
            return Err("missing EXT4 indirect block");
        }
        let mut block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, indirect, &mut block)?;
        for offset in (0..fs.block_size).step_by(4) {
            if logical * fs.block_size >= output.len() {
                break;
            }
            let block_no = le_u32(&block, offset) as u64;
            if block_no != 0 {
                copy_file_block(fs, block_no, logical, output)?;
            }
            logical += 1;
        }
    }
    Ok(())
}

fn copy_file_block(
    fs: &Ext4,
    physical: u64,
    logical: usize,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let start = logical
        .checked_mul(fs.block_size)
        .ok_or("EXT4 file offset overflow")?;
    if start >= output.len() {
        return Ok(());
    }
    let mut block = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, physical, &mut block)?;
    let count = core::cmp::min(fs.block_size, output.len() - start);
    output[start..start + count].copy_from_slice(&block[..count]);
    Ok(())
}

fn lookup_path_str(fs: &Ext4, path: &str) -> Result<Option<InodeInfo>, &'static str> {
    let components: Vec<&[u8]> = path
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .map(str::as_bytes)
        .collect();
    lookup_path(fs, &components)
}

fn parent_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn resolve_path(base: &str, path: &str) -> String {
    let mut components: Vec<&str> = if path.starts_with('/') {
        Vec::new()
    } else {
        base.split('/').filter(|part| !part.is_empty()).collect()
    };
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            part => components.push(part),
        }
    }
    components.join("/")
}

fn quoted_assignment<'a>(script: &'a str, name: &str) -> Option<&'a str> {
    let prefix = alloc::format!("{}=\"", name);
    let start = script.find(&prefix)? + prefix.len();
    let end = script[start..].find('"')? + start;
    Some(&script[start..end])
}

fn find_group_marker(script: &str, kind: &str) -> Option<String> {
    let prefix = alloc::format!("#### OS COMP TEST GROUP {} ", kind);
    let start = script.find(&prefix)?;
    let rest = &script[start..];
    let end = rest[prefix.len()..].find("####")? + prefix.len() + 4;
    Some(rest[..end].to_string())
}

fn split_shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in line.chars() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                current.push(ch);
            }
        } else {
            match ch {
                '\'' | '"' => quote = Some(ch),
                ' ' | '\t' => {
                    if !current.is_empty() {
                        words.push(core::mem::take(&mut current));
                    }
                }
                _ => current.push(ch),
            }
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn trim_shell_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn read_superblock() -> Result<Ext4, &'static str> {
    let mut superblock = [0u8; 1024];
    read_disk_bytes(1024, &mut superblock)?;
    if le_u16(&superblock, 56) != EXT4_MAGIC {
        return Err("no EXT4 superblock on test disk");
    }

    let log_block_size = le_u32(&superblock, 24);
    if log_block_size > 2 {
        return Err("unsupported EXT4 block size");
    }

    let inode_size = le_u16(&superblock, 88) as usize;
    let group_desc_size = le_u16(&superblock, 254) as usize;
    Ok(Ext4 {
        block_size: 1024usize << log_block_size,
        blocks_per_group: le_u32(&superblock, 32),
        inodes_per_group: le_u32(&superblock, 40),
        inode_size: inode_size.max(128),
        group_desc_size: group_desc_size.max(32),
    })
}

fn lookup_path(fs: &Ext4, components: &[&[u8]]) -> Result<Option<InodeInfo>, &'static str> {
    let mut inode_no = EXT4_ROOT_INO;
    for component in components {
        inode_no = match lookup_child(fs, inode_no, component)? {
            Some(inode_no) => inode_no,
            None => return Ok(None),
        };
    }

    let mut inode = [0u8; INODE_SIZE];
    read_inode(fs, inode_no, &mut inode)?;
    Ok(Some(InodeInfo {
        inode_no,
        mode: le_u16(&inode, 0),
        size: inode_file_size(&inode),
    }))
}

fn lookup_child(
    fs: &Ext4,
    parent_inode_no: u32,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    let mut inode = [0u8; INODE_SIZE];
    read_inode(fs, parent_inode_no, &mut inode)?;
    let file_size = inode_file_size(&inode);

    if le_u32(&inode, 32) & EXT4_EXTENTS_FL != 0 {
        find_in_extent_tree(fs, &inode[40..100], file_size, target)
    } else {
        find_in_direct_blocks(fs, &inode[40..88], file_size, target)
    }
}

fn find_in_extent_tree(
    fs: &Ext4,
    node: &[u8],
    file_size: u64,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    if le_u16(node, 0) != EXT4_EXTENT_MAGIC {
        return Err("invalid EXT4 extent header");
    }

    let entries = le_u16(node, 2) as usize;
    let depth = le_u16(node, 6);
    if depth == 0 {
        return find_in_extent_leaf(fs, node, file_size, target);
    }
    if depth != 1 {
        return Err("unsupported EXT4 extent depth");
    }

    for index in 0..entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent index");
        }
        let leaf = ((le_u16(node, offset + 8) as u64) << 32) | le_u32(node, offset + 4) as u64;
        let mut block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, leaf, &mut block)?;
        if let Some(inode_no) = find_in_extent_leaf(fs, &block[..fs.block_size], file_size, target)?
        {
            return Ok(Some(inode_no));
        }
    }
    Ok(None)
}

fn find_in_extent_leaf(
    fs: &Ext4,
    node: &[u8],
    file_size: u64,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    if le_u16(node, 0) != EXT4_EXTENT_MAGIC || le_u16(node, 6) != 0 {
        return Err("invalid EXT4 extent leaf");
    }

    let entries = le_u16(node, 2) as usize;
    for index in 0..entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent entry");
        }
        let logical = le_u32(node, offset) as u64;
        let len = (le_u16(node, offset + 4) & 0x7fff) as u64;
        let physical = ((le_u16(node, offset + 6) as u64) << 32) | le_u32(node, offset + 8) as u64;

        for block_index in 0..len {
            let file_offset = (logical + block_index) * fs.block_size as u64;
            if file_offset >= file_size {
                break;
            }
            if let Some(inode_no) = find_in_dir_block(fs, physical + block_index, target)? {
                return Ok(Some(inode_no));
            }
        }
    }
    Ok(None)
}

fn find_in_direct_blocks(
    fs: &Ext4,
    blocks: &[u8],
    file_size: u64,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    for index in 0..12 {
        let block_no = le_u32(blocks, index * 4) as u64;
        if block_no == 0 || index as u64 * fs.block_size as u64 >= file_size {
            break;
        }
        if let Some(inode_no) = find_in_dir_block(fs, block_no, target)? {
            return Ok(Some(inode_no));
        }
    }
    Ok(None)
}

fn find_in_dir_block(fs: &Ext4, block_no: u64, target: &[u8]) -> Result<Option<u32>, &'static str> {
    let mut block = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, block_no, &mut block)?;
    let mut offset = 0usize;
    while offset + 8 <= fs.block_size {
        let inode_no = le_u32(&block, offset);
        let record_len = le_u16(&block, offset + 4) as usize;
        let name_len = block[offset + 6] as usize;
        if record_len < 8 || offset + record_len > fs.block_size {
            break;
        }
        if inode_no != 0 && name_len == target.len() && name_len <= record_len - 8 {
            if &block[offset + 8..offset + 8 + name_len] == target {
                return Ok(Some(inode_no));
            }
        }
        offset += record_len;
    }
    Ok(None)
}

fn read_inode(fs: &Ext4, inode_no: u32, inode: &mut [u8]) -> Result<(), &'static str> {
    if fs.inodes_per_group == 0 || fs.blocks_per_group == 0 {
        return Err("invalid EXT4 group layout");
    }
    let group = (inode_no - 1) / fs.inodes_per_group;
    let desc_block = if fs.block_size == 1024 { 2 } else { 1 };
    let desc_offset =
        desc_block as u64 * fs.block_size as u64 + group as u64 * fs.group_desc_size as u64;
    let mut desc = [0u8; GROUP_DESC_SIZE];
    read_disk_bytes(desc_offset, &mut desc)?;
    let table_hi = if fs.group_desc_size >= 64 {
        le_u32(&desc, 40) as u64
    } else {
        0
    };
    let table = (table_hi << 32) | le_u32(&desc, 8) as u64;
    if table == 0 {
        return Err("invalid EXT4 inode table");
    }
    let index = (inode_no - 1) % fs.inodes_per_group;
    let offset = table * fs.block_size as u64 + index as u64 * fs.inode_size as u64;
    read_disk_bytes(offset, inode)
}

fn read_fs_block(fs: &Ext4, block_no: u64, output: &mut [u8]) -> Result<(), &'static str> {
    read_disk_bytes(
        block_no * fs.block_size as u64,
        &mut output[..fs.block_size],
    )
}

fn read_disk_bytes(offset: u64, output: &mut [u8]) -> Result<(), &'static str> {
    let device = BLOCK_DEVICE
        .lock()
        .clone()
        .ok_or("block device unavailable")?;
    let mut copied = 0usize;
    let mut sector = [0u8; SECTOR_SIZE];
    while copied < output.len() {
        let absolute = offset as usize + copied;
        let sector_no = absolute / SECTOR_SIZE;
        let sector_offset = absolute % SECTOR_SIZE;
        device.read_block(sector_no, &mut sector);
        let count = core::cmp::min(SECTOR_SIZE - sector_offset, output.len() - copied);
        output[copied..copied + count]
            .copy_from_slice(&sector[sector_offset..sector_offset + count]);
        copied += count;
    }
    Ok(())
}

fn inode_file_size(inode: &[u8]) -> u64 {
    ((le_u32(inode, 108) as u64) << 32) | le_u32(inode, 4) as u64
}

fn le_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn le_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}
