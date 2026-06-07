use super::block;
use core::ptr::addr_of_mut;

const EXT4_SUPER_OFFSET: u64 = 1024;
const EXT4_SUPER_MAGIC: u16 = 0xef53;
const EXT4_EXTENTS_FL: u32 = 0x0008_0000;
const EXT4_EXTENT_MAGIC: u16 = 0xf30a;
const EXT4_ROOT_INO: u32 = 2;
const EXT4_S_IFDIR: u16 = 0x4000;
const EXT4_S_IFREG: u16 = 0x8000;
const EXT4_MODE_TYPE_MASK: u16 = 0xf000;
const MAX_BLOCK_SIZE: usize = 4096;
const MIN_INODE_SIZE: usize = 128;
const INODE_PARSE_SIZE: usize = 160;
const GROUP_DESC_PARSE_SIZE: usize = 64;
const EXT4_NAME_MAX: usize = 255;
const TEST_SCRIPT_SUFFIX: &[u8] = b"_testcode.sh";
const SCRIPT_BUFFER_SIZE: usize = 16 * 1024;
const SCRIPT_PATH_MAX: usize = 128;
const SCRIPT_COMMAND_MAX: usize = 64;
const SCRIPT_ARG_MAX_LEN: usize = 64;
const MAX_SCRIPT_DEPTH: usize = 2;
const MAX_SCAN_DEPTH: usize = 2;
const GROUP_MARKER_PREFIX: &[u8] = b"#### OS COMP TEST GROUP ";
const GROUP_START_PREFIX: &[u8] = b"#### OS COMP TEST GROUP START ";
const MARKER_END: &[u8] = b"####";

static mut SCRIPT_BUFFER: [u8; SCRIPT_BUFFER_SIZE] = [0; SCRIPT_BUFFER_SIZE];
static mut NESTED_SCRIPT_BUFFER: [u8; SCRIPT_BUFFER_SIZE] = [0; SCRIPT_BUFFER_SIZE];
static mut MOUNTED_FS: Option<Ext4Fs> = None;
static mut SCRIPT_COMMANDS: [ScriptCommand; SCRIPT_COMMAND_MAX] =
    [const { ScriptCommand::zero() }; SCRIPT_COMMAND_MAX];
static mut SCRIPT_COMMAND_COUNT: usize = 0;
static mut SCRIPT_COMMAND_NEXT: usize = 0;

#[derive(Clone, Copy)]
struct ScriptCommand {
    path: [u8; SCRIPT_PATH_MAX],
    path_len: usize,
    args: [[u8; SCRIPT_ARG_MAX_LEN]; crate::loader::EXTERNAL_ARG_MAX],
    arg_len: [usize; crate::loader::EXTERNAL_ARG_MAX],
    argc: usize,
}

impl ScriptCommand {
    const fn zero() -> Self {
        Self {
            path: [0; SCRIPT_PATH_MAX],
            path_len: 0,
            args: [[0; SCRIPT_ARG_MAX_LEN]; crate::loader::EXTERNAL_ARG_MAX],
            arg_len: [0; crate::loader::EXTERNAL_ARG_MAX],
            argc: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Ext4Fs {
    block_size: usize,
    blocks_per_group: u32,
    inodes_per_group: u32,
    inode_size: usize,
    group_desc_size: usize,
}

#[derive(Clone, Copy)]
struct FileInfo {
    inode_no: u32,
    size: u64,
    mode: u16,
}

#[derive(Clone, Copy)]
pub struct Ext4File {
    pub inode_no: u32,
    pub size: u64,
}

pub fn init() {
    if !block::is_ready() {
        return;
    }

    match read_superblock() {
        Ok(fs) => {
            set_mounted_fs(fs);
            match scan_test_scripts(&fs) {
                Ok(count) => {
                    crate::println!("ext4: found {} test script(s)", count);
                }
                Err(message) => {
                    crate::println!("ext4: {}", message);
                }
            }
        }
        Err(message) => {
            crate::println!("ext4: {}", message);
        }
    }
}

pub fn open_path(path: &[u8]) -> Option<Ext4File> {
    let fs = mounted_fs()?;
    let info = lookup_path(&fs, path).ok().flatten()?;

    if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
        return None;
    }

    Some(Ext4File {
        inode_no: info.inode_no,
        size: info.size,
    })
}

pub fn read_file_at(
    file: Ext4File,
    offset: usize,
    output: &mut [u8],
) -> Result<usize, &'static str> {
    if output.is_empty() {
        return Ok(0);
    }

    if offset as u64 >= file.size {
        return Ok(0);
    }

    let fs = mounted_fs().ok_or("EXT4 not mounted")?;
    let inode_table = read_inode_table_block(&fs, file.inode_no)?;
    let mut inode = [0u8; INODE_PARSE_SIZE];
    read_inode(&fs, inode_table, file.inode_no, &mut inode)?;

    copy_inode_range(&fs, &inode, offset as u64, output)
}

pub fn load_next_queued_external() -> bool {
    let fs = match mounted_fs() {
        Some(fs) => fs,
        None => return false,
    };

    loop {
        let index = unsafe {
            if SCRIPT_COMMAND_NEXT >= SCRIPT_COMMAND_COUNT {
                return false;
            }

            let index = SCRIPT_COMMAND_NEXT;
            SCRIPT_COMMAND_NEXT += 1;
            index
        };

        let command = unsafe { SCRIPT_COMMANDS[index] };
        if load_script_command(&fs, command) {
            return true;
        }
    }
}

pub fn load_external_elf_path(path: &[u8]) -> bool {
    let fs = match mounted_fs() {
        Some(fs) => fs,
        None => return false,
    };

    let info = match lookup_path(&fs, path) {
        Ok(Some(info)) => info,
        _ => return false,
    };

    load_external_file(&fs, path, info, false)
}

fn set_mounted_fs(fs: Ext4Fs) {
    unsafe {
        MOUNTED_FS = Some(fs);
    }
}

fn mounted_fs() -> Option<Ext4Fs> {
    unsafe { MOUNTED_FS }
}

fn scan_test_scripts(fs: &Ext4Fs) -> Result<usize, &'static str> {
    if try_load_known_basic_script(fs, b"musl", b"basic_testcode.sh") {
        return Ok(1);
    }
    if try_load_known_basic_script(fs, b"glibc", b"basic_testcode.sh") {
        return Ok(1);
    }
    if try_load_known_basic_script(fs, b"", b"basic_testcode.sh") {
        return Ok(1);
    }

    let mut found = 0usize;
    scan_directory(fs, EXT4_ROOT_INO, &[], 0, &mut found)?;
    Ok(found)
}

fn try_load_known_basic_script(fs: &Ext4Fs, dir_path: &[u8], name: &[u8]) -> bool {
    let mut path = [0u8; SCRIPT_PATH_MAX];
    let mut path_len = 0usize;
    if !dir_path.is_empty() {
        path_len = append_path_part(&mut path, path_len, dir_path);
    }
    path_len = append_path_part(&mut path, path_len, name);

    let info = match lookup_path(fs, &path[..path_len]) {
        Ok(Some(info)) => info,
        _ => return false,
    };

    let mut found = 0usize;
    handle_test_script(fs, info.inode_no, dir_path, name, &mut found)
}

fn scan_directory(
    fs: &Ext4Fs,
    inode_no: u32,
    dir_path: &[u8],
    scan_depth: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    let inode_table = read_inode_table_block(fs, inode_no)?;
    let mut inode = [0u8; INODE_PARSE_SIZE];
    read_inode(fs, inode_table, inode_no, &mut inode)?;

    let mode = le_u16(&inode, 0);
    if mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFDIR {
        return Err("inode is not a directory");
    }

    let dir_size = inode_size(&inode);
    let flags = le_u32(&inode, 32);

    if flags & EXT4_EXTENTS_FL != 0 {
        scan_extent_tree(fs, &inode[40..100], dir_size, dir_path, scan_depth, found)?;
    } else {
        scan_direct_blocks(fs, &inode[40..88], dir_size, dir_path, scan_depth, found)?;
    }

    Ok(())
}

fn read_superblock() -> Result<Ext4Fs, &'static str> {
    let mut superblock = [0u8; block::BLOCK_SIZE];
    read_disk_bytes(EXT4_SUPER_OFFSET, &mut superblock)?;

    if le_u16(&superblock, 56) != EXT4_SUPER_MAGIC {
        return Err("no EXT4 superblock on test disk");
    }

    let log_block_size = le_u32(&superblock, 24);
    if log_block_size > 2 {
        return Err("unsupported EXT4 block size");
    }

    let block_size = 1024usize << log_block_size;
    let inode_size = {
        let value = le_u16(&superblock, 88) as usize;
        if value == 0 { MIN_INODE_SIZE } else { value }
    };

    if inode_size < MIN_INODE_SIZE {
        return Err("unsupported EXT4 inode size");
    }

    let group_desc_size = {
        let value = le_u16(&superblock, 254) as usize;
        if value < 32 { 32 } else { value }
    };

    Ok(Ext4Fs {
        block_size,
        blocks_per_group: le_u32(&superblock, 32),
        inodes_per_group: le_u32(&superblock, 40),
        inode_size,
        group_desc_size,
    })
}

fn read_inode_table_block(fs: &Ext4Fs, inode_no: u32) -> Result<u64, &'static str> {
    if fs.inodes_per_group == 0 || fs.blocks_per_group == 0 {
        return Err("invalid EXT4 group layout");
    }

    let group = (inode_no - 1) / fs.inodes_per_group;
    let desc_offset = group_desc_table_offset(fs) + group as u64 * fs.group_desc_size as u64;
    let mut desc = [0u8; GROUP_DESC_PARSE_SIZE];
    read_disk_bytes(desc_offset, &mut desc)?;

    let lo = le_u32(&desc, 8) as u64;
    let hi = if fs.group_desc_size >= 64 {
        le_u32(&desc, 40) as u64
    } else {
        0
    };
    let inode_table = (hi << 32) | lo;
    if inode_table == 0 {
        return Err("invalid EXT4 inode table");
    }

    Ok(inode_table)
}

fn read_inode(
    fs: &Ext4Fs,
    inode_table: u64,
    inode_no: u32,
    inode: &mut [u8],
) -> Result<(), &'static str> {
    let index = (inode_no - 1) % fs.inodes_per_group;
    let offset = inode_table * fs.block_size as u64 + index as u64 * fs.inode_size as u64;
    read_disk_bytes(offset, inode)
}

fn group_desc_table_offset(fs: &Ext4Fs) -> u64 {
    let block = if fs.block_size == 1024 { 2 } else { 1 };
    block as u64 * fs.block_size as u64
}

fn scan_extent_tree(
    fs: &Ext4Fs,
    root: &[u8],
    file_size: u64,
    dir_path: &[u8],
    scan_depth: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    let depth = extent_depth(root)?;

    if depth == 0 {
        return scan_extent_leaf(fs, root, file_size, dir_path, scan_depth, found);
    }

    if depth != 1 {
        return Err("unsupported EXT4 extent depth");
    }

    let entries = extent_entries(root)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > root.len() {
            return Err("invalid EXT4 extent index");
        }

        let leaf = extent_index_leaf(&root[offset..offset + 12]);
        let mut leaf_block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, leaf, &mut leaf_block)?;
        scan_extent_leaf(
            fs,
            &leaf_block[..fs.block_size],
            file_size,
            dir_path,
            scan_depth,
            found,
        )?;

        index += 1;
    }

    Ok(())
}

fn scan_extent_leaf(
    fs: &Ext4Fs,
    node: &[u8],
    file_size: u64,
    dir_path: &[u8],
    scan_depth: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    if extent_depth(node)? != 0 {
        return Err("invalid EXT4 extent leaf");
    }

    let entries = extent_entries(node)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent entry");
        }

        let logical = le_u32(node, offset) as u64;
        let len = (le_u16(node, offset + 4) & 0x7fff) as u64;
        let physical = ((le_u16(node, offset + 6) as u64) << 32) | le_u32(node, offset + 8) as u64;

        let mut block_index = 0u64;
        while block_index < len {
            let file_offset = (logical + block_index) * fs.block_size as u64;
            if file_offset >= file_size {
                break;
            }

            let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
            scan_dir_data_block(
                fs,
                physical + block_index,
                valid_len,
                dir_path,
                scan_depth,
                found,
            )?;

            block_index += 1;
        }

        index += 1;
    }

    Ok(())
}

fn scan_direct_blocks(
    fs: &Ext4Fs,
    i_block: &[u8],
    file_size: u64,
    dir_path: &[u8],
    scan_depth: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    let mut index = 0usize;
    while index < 12 {
        let block_no = le_u32(i_block, index * 4) as u64;
        if block_no == 0 {
            break;
        }

        let file_offset = index as u64 * fs.block_size as u64;
        if file_offset >= file_size {
            break;
        }

        let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
        scan_dir_data_block(fs, block_no, valid_len, dir_path, scan_depth, found)?;

        index += 1;
    }

    Ok(())
}

fn scan_dir_data_block(
    fs: &Ext4Fs,
    block_no: u64,
    valid_len: usize,
    dir_path: &[u8],
    scan_depth: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    let mut buffer = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, block_no, &mut buffer)?;
    scan_dir_entries(fs, &buffer[..valid_len], dir_path, scan_depth, found)
}

fn scan_dir_entries(
    fs: &Ext4Fs,
    block: &[u8],
    dir_path: &[u8],
    scan_depth: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    let mut offset = 0usize;
    while offset + 8 <= block.len() {
        let inode = le_u32(block, offset);
        let rec_len = le_u16(block, offset + 4) as usize;
        let name_len = block[offset + 6] as usize;

        if rec_len < 8 || offset + rec_len > block.len() {
            break;
        }

        if inode != 0 && name_len <= EXT4_NAME_MAX && name_len <= rec_len - 8 {
            let name = &block[offset + 8..offset + 8 + name_len];
            if is_basic_test_script(name) {
                if handle_test_script(fs, inode, dir_path, name, found) {
                    return Ok(());
                }
            } else if scan_depth < MAX_SCAN_DEPTH && !is_dot_entry(name) {
                let mut child_inode = [0u8; INODE_PARSE_SIZE];
                let child_result = read_inode_table_block(fs, inode)
                    .and_then(|inode_table| read_inode(fs, inode_table, inode, &mut child_inode));
                if child_result.is_ok()
                    && le_u16(&child_inode, 0) & EXT4_MODE_TYPE_MASK == EXT4_S_IFDIR
                {
                    let mut child_path = [0u8; SCRIPT_PATH_MAX];
                    let base_len = copy_path(dir_path, &mut child_path);
                    let child_len = append_path_part(&mut child_path, base_len, name);
                    let _ = scan_directory(fs, inode, &child_path[..child_len], scan_depth + 1, found);
                }
            }

            if crate::loader::has_external_app() {
                return Ok(());
            }
        }

        offset += rec_len;
    }

    Ok(())
}

fn find_dir_entry(block: &[u8], target: &[u8]) -> Option<u32> {
    let mut offset = 0usize;
    while offset + 8 <= block.len() {
        let inode = le_u32(block, offset);
        let rec_len = le_u16(block, offset + 4) as usize;
        let name_len = block[offset + 6] as usize;

        if rec_len < 8 || offset + rec_len > block.len() {
            break;
        }

        if inode != 0 && name_len == target.len() && name_len <= rec_len - 8 {
            let name = &block[offset + 8..offset + 8 + name_len];
            if name == target {
                return Some(inode);
            }
        }

        offset += rec_len;
    }

    None
}

fn handle_test_script(
    fs: &Ext4Fs,
    inode_no: u32,
    dir_path: &[u8],
    name: &[u8],
    found: &mut usize,
) -> bool {
    if crate::loader::has_external_app() {
        return false;
    }

    *found += 1;

    crate::print!("oscomp: found test script ");
    if !dir_path.is_empty() {
        print_name(dir_path);
        crate::print!("/");
    }
    print_name(name);
    crate::println!();

    let mut inode = [0u8; INODE_PARSE_SIZE];
    let read_result = read_inode_table_block(fs, inode_no)
        .and_then(|inode_table| read_inode(fs, inode_table, inode_no, &mut inode));

    if read_result.is_err() {
        emit_fallback_group_markers(name);
        return false;
    }

    let file_size = inode_size(&inode);
    let read_len = core::cmp::min(file_size as usize, SCRIPT_BUFFER_SIZE);
    let script = unsafe {
        core::slice::from_raw_parts_mut(addr_of_mut!(SCRIPT_BUFFER) as *mut u8, read_len)
    };

    if read_inode_data(fs, &inode, script).is_err() {
        emit_fallback_group_markers(name);
        return false;
    }

    if try_load_first_command_from_script(fs, script, dir_path, 0) {
        emit_group_start_from_script_or_fallback(name, script);
        set_external_group_from_script(name, script);
        return true;
    }

    if emit_group_markers_from_script(script) == 0 {
        emit_fallback_group_markers(name);
    }

    false
}

fn try_load_first_command_from_script(
    fs: &Ext4Fs,
    script: &[u8],
    initial_cwd: &[u8],
    depth: usize,
) -> bool {
    if crate::loader::has_external_app() {
        return false;
    }

    clear_script_command_queue();
    if !collect_commands_from_script(fs, script, initial_cwd, depth) {
        return false;
    }

    load_next_queued_external()
}

fn collect_commands_from_script(
    fs: &Ext4Fs,
    script: &[u8],
    initial_cwd: &[u8],
    depth: usize,
) -> bool {
    let mut cwd = [0u8; SCRIPT_PATH_MAX];
    let mut cwd_len = copy_path(initial_cwd, &mut cwd);
    let mut line_start = 0usize;
    let mut queued = false;

    while line_start < script.len() {
        let line_end = find_line_end(script, line_start);
        let line = &script[line_start..line_end];

        if collect_command_from_line(fs, line, &mut cwd, &mut cwd_len, depth) {
            queued = true;
        }

        line_start = line_end + 1;
    }

    queued
}

fn collect_command_from_line(
    fs: &Ext4Fs,
    line: &[u8],
    cwd: &mut [u8; SCRIPT_PATH_MAX],
    cwd_len: &mut usize,
    depth: usize,
) -> bool {
    let (cmd_start, cmd_end, next_index) = match next_token(line, 0) {
        Some(token) => token,
        None => return false,
    };

    let command = &line[cmd_start..cmd_end];
    if command.is_empty() || command[0] == b'#' {
        return false;
    }

    if command == b"cd" {
        if let Some((arg_start, arg_end, _)) = next_token(line, next_index) {
            let mut next_cwd = [0u8; SCRIPT_PATH_MAX];
            let next_len = resolve_path(&cwd[..*cwd_len], &line[arg_start..arg_end], &mut next_cwd);
            cwd.fill(0);
            cwd[..next_len].copy_from_slice(&next_cwd[..next_len]);
            *cwd_len = next_len;
        }
        return false;
    }

    if is_busybox_echo(command, line, next_index) {
        return false;
    }

    let mut path = [0u8; SCRIPT_PATH_MAX];
    let path_len = resolve_path(&cwd[..*cwd_len], command, &mut path);
    if path_len == 0 {
        return false;
    }

    let path_slice = &path[..path_len];
    if path_slice.ends_with(b".sh") {
        return collect_nested_script(fs, path_slice, &cwd[..*cwd_len], depth + 1);
    }

    enqueue_elf_command(fs, path_slice, line, next_index)
}

fn collect_nested_script(fs: &Ext4Fs, path: &[u8], cwd: &[u8], depth: usize) -> bool {
    if depth > MAX_SCRIPT_DEPTH {
        return false;
    }

    let info = match lookup_path(fs, path) {
        Ok(Some(info)) => info,
        _ => return false,
    };

    if info.size == 0 || info.size as usize > SCRIPT_BUFFER_SIZE {
        return false;
    }

    let read_len = info.size as usize;
    let script = unsafe {
        core::slice::from_raw_parts_mut(addr_of_mut!(NESTED_SCRIPT_BUFFER) as *mut u8, read_len)
    };

    if read_root_file_into(fs, info, script).is_err() {
        return false;
    }

    collect_commands_from_script(fs, script, cwd, depth)
}

fn enqueue_elf_command(
    fs: &Ext4Fs,
    path: &[u8],
    line: &[u8],
    mut next_index: usize,
) -> bool {
    let command_index = unsafe {
        if SCRIPT_COMMAND_COUNT >= SCRIPT_COMMAND_MAX {
            return false;
        }

        SCRIPT_COMMAND_COUNT
    };

    let info = match lookup_path(fs, path) {
        Ok(Some(info)) => info,
        _ => return false,
    };

    if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
        return false;
    }

    if info.size == 0 || info.size as usize > crate::loader::EXTERNAL_APP_MAX_SIZE {
        return false;
    }

    let command = unsafe { &mut *core::ptr::addr_of_mut!(SCRIPT_COMMANDS[command_index]) };
    *command = ScriptCommand::zero();
    command.path_len = copy_path(path, &mut command.path);
    push_script_command_arg(command, path);
    while let Some((arg_start, arg_end, next)) = next_token(line, next_index) {
        push_script_command_arg(command, &line[arg_start..arg_end]);
        next_index = next;
    }

    unsafe {
        SCRIPT_COMMAND_COUNT += 1;
    }

    true
}

fn clear_script_command_queue() {
    unsafe {
        SCRIPT_COMMAND_COUNT = 0;
        SCRIPT_COMMAND_NEXT = 0;
    }
}

fn push_script_command_arg(command: &mut ScriptCommand, arg: &[u8]) -> bool {
    if command.argc >= crate::loader::EXTERNAL_ARG_MAX {
        return false;
    }

    let index = command.argc;
    let copy_len = core::cmp::min(arg.len(), SCRIPT_ARG_MAX_LEN);
    command.args[index].fill(0);
    command.args[index][..copy_len].copy_from_slice(&arg[..copy_len]);
    command.arg_len[index] = copy_len;
    command.argc += 1;
    true
}

fn load_script_command(fs: &Ext4Fs, command: ScriptCommand) -> bool {
    if command.path_len == 0 {
        return false;
    }

    let path = &command.path[..command.path_len];
    let info = match lookup_path(fs, path) {
        Ok(Some(info)) => info,
        _ => return false,
    };

    crate::loader::clear_external_args();
    let mut index = 0usize;
    while index < command.argc {
        let len = command.arg_len[index];
        crate::loader::push_external_arg(&command.args[index][..len]);
        index += 1;
    }

    load_external_file(fs, path, info, true)
}

fn load_external_file(fs: &Ext4Fs, path: &[u8], info: FileInfo, announce: bool) -> bool {
    if info.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFREG {
        return false;
    }

    if info.size == 0 || info.size as usize > crate::loader::EXTERNAL_APP_MAX_SIZE {
        return false;
    }

    let buffer = crate::loader::external_app_buffer_mut();
    let read_len = info.size as usize;

    if read_root_file_into(fs, info, &mut buffer[..read_len]).is_err() {
        return false;
    }

    if !is_elf(&buffer[..read_len]) {
        return false;
    }

    set_external_cwd_from_path(path);
    if announce {
        crate::print!("loader: selected external ELF ");
        print_name(path);
        crate::println!();
    }
    crate::loader::set_external_app(read_len);

    true
}

fn set_external_cwd_from_path(path: &[u8]) {
    let mut last_slash = None;
    let mut index = 0usize;
    while index < path.len() {
        if path[index] == b'/' {
            last_slash = Some(index);
        }
        index += 1;
    }

    if let Some(index) = last_slash {
        crate::loader::set_external_cwd(&path[..index]);
    } else {
        crate::loader::set_external_cwd(&[]);
    }
}

fn is_busybox_echo(command: &[u8], line: &[u8], next_index: usize) -> bool {
    if !path_basename_eq(command, b"busybox") {
        return false;
    }

    if let Some((arg_start, arg_end, _)) = next_token(line, next_index) {
        &line[arg_start..arg_end] == b"echo"
    } else {
        false
    }
}

fn path_basename_eq(path: &[u8], name: &[u8]) -> bool {
    let mut start = 0usize;
    let mut index = 0usize;
    while index < path.len() {
        if path[index] == b'/' {
            start = index + 1;
        }
        index += 1;
    }

    &path[start..] == name
}

fn find_line_end(buffer: &[u8], start: usize) -> usize {
    let mut index = start;
    while index < buffer.len() && buffer[index] != b'\n' && buffer[index] != 0 {
        index += 1;
    }
    index
}

fn next_token(buffer: &[u8], mut index: usize) -> Option<(usize, usize, usize)> {
    while index < buffer.len() {
        let byte = buffer[index];
        if byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b';' {
            index += 1;
        } else {
            break;
        }
    }

    if index >= buffer.len() || buffer[index] == b'#' || buffer[index] == 0 {
        return None;
    }

    let quote = if buffer[index] == b'"' || buffer[index] == b'\'' {
        let quote = buffer[index];
        index += 1;
        quote
    } else {
        0
    };

    let start = index;
    while index < buffer.len() {
        let byte = buffer[index];
        if quote != 0 {
            if byte == quote {
                break;
            }
        } else if byte == b' '
            || byte == b'\t'
            || byte == b'\r'
            || byte == b'\n'
            || byte == b';'
            || byte == b'|'
            || byte == 0
        {
            break;
        }

        index += 1;
    }

    let end = index;
    if quote != 0 && index < buffer.len() {
        index += 1;
    }

    Some((start, end, index))
}

fn copy_path(input: &[u8], output: &mut [u8; SCRIPT_PATH_MAX]) -> usize {
    let copy_len = core::cmp::min(input.len(), SCRIPT_PATH_MAX);
    output[..copy_len].copy_from_slice(&input[..copy_len]);
    copy_len
}

fn resolve_path(cwd: &[u8], path: &[u8], output: &mut [u8; SCRIPT_PATH_MAX]) -> usize {
    let mut source = path;
    let mut len = 0usize;

    if source.starts_with(b"/") {
        source = trim_leading_slashes(source);
    } else {
        while source.starts_with(b"./") {
            source = &source[2..];
        }

        if !cwd.is_empty() {
            len = append_path_part(output, len, cwd);
        }
    }

    append_path_part(output, len, source)
}

fn append_path_part(output: &mut [u8; SCRIPT_PATH_MAX], mut len: usize, part: &[u8]) -> usize {
    if part.is_empty() {
        return len;
    }

    if len > 0 && len < SCRIPT_PATH_MAX {
        output[len] = b'/';
        len += 1;
    }

    let copy_len = core::cmp::min(part.len(), SCRIPT_PATH_MAX - len);
    output[len..len + copy_len].copy_from_slice(&part[..copy_len]);
    len + copy_len
}

fn trim_leading_slashes(mut path: &[u8]) -> &[u8] {
    while path.starts_with(b"/") {
        path = &path[1..];
    }
    path
}

fn lookup_path(fs: &Ext4Fs, path: &[u8]) -> Result<Option<FileInfo>, &'static str> {
    let mut index = 0usize;
    let mut current_inode_no = EXT4_ROOT_INO;
    let mut saw_component = false;

    while index < path.len() {
        while index < path.len() && path[index] == b'/' {
            index += 1;
        }

        if index >= path.len() {
            break;
        }

        let component_start = index;
        while index < path.len() && path[index] != b'/' {
            index += 1;
        }

        let component = &path[component_start..index];
        if component.is_empty() || component == b"." {
            continue;
        }

        saw_component = true;
        if component == b".." {
            current_inode_no = EXT4_ROOT_INO;
            continue;
        }

        let child_inode_no = match lookup_child_inode(fs, current_inode_no, component)? {
            Some(inode_no) => inode_no,
            None => return Ok(None),
        };

        let mut next_index = index;
        while next_index < path.len() && path[next_index] == b'/' {
            next_index += 1;
        }

        let child = read_file_info(fs, child_inode_no)?;
        if next_index >= path.len() {
            return Ok(Some(child));
        }

        if child.mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFDIR {
            return Ok(None);
        }

        current_inode_no = child_inode_no;
        index = next_index;
    }

    if saw_component {
        read_file_info(fs, current_inode_no).map(Some)
    } else {
        Ok(None)
    }
}

fn lookup_child_inode(
    fs: &Ext4Fs,
    parent_inode_no: u32,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    if target.is_empty() || contains_byte(target, 0) {
        return Ok(None);
    }

    let mut parent_inode = [0u8; INODE_PARSE_SIZE];
    read_inode_by_no(fs, parent_inode_no, &mut parent_inode)?;

    if le_u16(&parent_inode, 0) & EXT4_MODE_TYPE_MASK != EXT4_S_IFDIR {
        return Ok(None);
    }

    let parent_size = inode_size(&parent_inode);
    if le_u32(&parent_inode, 32) & EXT4_EXTENTS_FL != 0 {
        find_in_extent_tree(fs, &parent_inode[40..100], parent_size, target)
    } else {
        find_in_direct_blocks(fs, &parent_inode[40..88], parent_size, target)
    }
}

fn read_file_info(fs: &Ext4Fs, inode_no: u32) -> Result<FileInfo, &'static str> {
    let mut inode = [0u8; INODE_PARSE_SIZE];
    read_inode_by_no(fs, inode_no, &mut inode)?;

    Ok(FileInfo {
        inode_no,
        size: inode_size(&inode),
        mode: le_u16(&inode, 0),
    })
}

fn read_inode_by_no(
    fs: &Ext4Fs,
    inode_no: u32,
    inode: &mut [u8],
) -> Result<(), &'static str> {
    let inode_table = read_inode_table_block(fs, inode_no)?;
    read_inode(fs, inode_table, inode_no, inode)
}

fn read_fs_block(fs: &Ext4Fs, block_no: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
    if fs.block_size > MAX_BLOCK_SIZE || buffer.len() < fs.block_size {
        return Err("unsupported EXT4 block buffer");
    }

    read_disk_bytes(block_no * fs.block_size as u64, &mut buffer[..fs.block_size])
}

fn find_in_extent_tree(
    fs: &Ext4Fs,
    root: &[u8],
    file_size: u64,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    let depth = extent_depth(root)?;

    if depth == 0 {
        return find_in_extent_leaf(fs, root, file_size, target);
    }

    if depth != 1 {
        return Err("unsupported EXT4 extent depth");
    }

    let entries = extent_entries(root)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > root.len() {
            return Err("invalid EXT4 extent index");
        }

        let leaf = extent_index_leaf(&root[offset..offset + 12]);
        let mut leaf_block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, leaf, &mut leaf_block)?;

        if let Some(inode_no) = find_in_extent_leaf(fs, &leaf_block[..fs.block_size], file_size, target)? {
            return Ok(Some(inode_no));
        }

        index += 1;
    }

    Ok(None)
}

fn find_in_extent_leaf(
    fs: &Ext4Fs,
    node: &[u8],
    file_size: u64,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    if extent_depth(node)? != 0 {
        return Err("invalid EXT4 extent leaf");
    }

    let entries = extent_entries(node)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent entry");
        }

        let logical = le_u32(node, offset) as u64;
        let len = (le_u16(node, offset + 4) & 0x7fff) as u64;
        let physical = ((le_u16(node, offset + 6) as u64) << 32) | le_u32(node, offset + 8) as u64;

        let mut block_index = 0u64;
        while block_index < len {
            let file_offset = (logical + block_index) * fs.block_size as u64;
            if file_offset >= file_size {
                break;
            }

            let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
            if let Some(inode_no) = find_in_dir_data_block(fs, physical + block_index, valid_len, target)? {
                return Ok(Some(inode_no));
            }

            block_index += 1;
        }

        index += 1;
    }

    Ok(None)
}

fn find_in_direct_blocks(
    fs: &Ext4Fs,
    i_block: &[u8],
    file_size: u64,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    let mut index = 0usize;
    while index < 12 {
        let block_no = le_u32(i_block, index * 4) as u64;
        if block_no == 0 {
            break;
        }

        let file_offset = index as u64 * fs.block_size as u64;
        if file_offset >= file_size {
            break;
        }

        let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
        if let Some(inode_no) = find_in_dir_data_block(fs, block_no, valid_len, target)? {
            return Ok(Some(inode_no));
        }

        index += 1;
    }

    Ok(None)
}

fn find_in_dir_data_block(
    fs: &Ext4Fs,
    block_no: u64,
    valid_len: usize,
    target: &[u8],
) -> Result<Option<u32>, &'static str> {
    let mut buffer = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, block_no, &mut buffer)?;
    Ok(find_dir_entry(&buffer[..valid_len], target))
}

fn read_root_file_into(
    fs: &Ext4Fs,
    info: FileInfo,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let inode_table = read_inode_table_block(fs, info.inode_no)?;
    let mut inode = [0u8; INODE_PARSE_SIZE];
    read_inode(fs, inode_table, info.inode_no, &mut inode)?;
    read_inode_data(fs, &inode, output)
}

fn read_inode_data(fs: &Ext4Fs, inode: &[u8], output: &mut [u8]) -> Result<(), &'static str> {
    output.fill(0);

    let file_size = inode_size(inode);
    let flags = le_u32(inode, 32);

    if flags & EXT4_EXTENTS_FL != 0 {
        copy_extent_tree(fs, &inode[40..100], file_size, output)
    } else {
        copy_direct_blocks(fs, &inode[40..88], file_size, output)
    }
}

fn copy_extent_tree(
    fs: &Ext4Fs,
    root: &[u8],
    file_size: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let depth = extent_depth(root)?;

    if depth == 0 {
        return copy_extent_leaf(fs, root, file_size, output);
    }

    if depth != 1 {
        return Err("unsupported EXT4 extent depth");
    }

    let entries = extent_entries(root)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > root.len() {
            return Err("invalid EXT4 extent index");
        }

        let leaf = extent_index_leaf(&root[offset..offset + 12]);
        let mut leaf_block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, leaf, &mut leaf_block)?;
        copy_extent_leaf(fs, &leaf_block[..fs.block_size], file_size, output)?;

        index += 1;
    }

    Ok(())
}

fn copy_extent_leaf(
    fs: &Ext4Fs,
    node: &[u8],
    file_size: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    if extent_depth(node)? != 0 {
        return Err("invalid EXT4 extent leaf");
    }

    let entries = extent_entries(node)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent entry");
        }

        let logical = le_u32(node, offset) as u64;
        let len = (le_u16(node, offset + 4) & 0x7fff) as u64;
        let physical = ((le_u16(node, offset + 6) as u64) << 32) | le_u32(node, offset + 8) as u64;

        let mut block_index = 0u64;
        while block_index < len {
            let file_offset = (logical + block_index) * fs.block_size as u64;
            if file_offset >= file_size {
                break;
            }

            let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
            copy_data_block(fs, physical + block_index, file_offset, valid_len, output)?;

            block_index += 1;
        }

        index += 1;
    }

    Ok(())
}

fn copy_direct_blocks(
    fs: &Ext4Fs,
    i_block: &[u8],
    file_size: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let mut index = 0usize;
    while index < 12 {
        let block_no = le_u32(i_block, index * 4) as u64;
        if block_no == 0 {
            break;
        }

        let file_offset = index as u64 * fs.block_size as u64;
        if file_offset >= file_size {
            break;
        }

        let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
        copy_data_block(fs, block_no, file_offset, valid_len, output)?;

        index += 1;
    }

    Ok(())
}

fn copy_data_block(
    fs: &Ext4Fs,
    block_no: u64,
    file_offset: u64,
    valid_len: usize,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let dest_start = file_offset as usize;
    if dest_start >= output.len() {
        return Ok(());
    }

    let copy_len = core::cmp::min(valid_len, output.len() - dest_start);
    let mut buffer = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, block_no, &mut buffer)?;
    output[dest_start..dest_start + copy_len].copy_from_slice(&buffer[..copy_len]);

    Ok(())
}

fn copy_inode_range(
    fs: &Ext4Fs,
    inode: &[u8],
    read_offset: u64,
    output: &mut [u8],
) -> Result<usize, &'static str> {
    let file_size = inode_size(inode);
    if read_offset >= file_size {
        return Ok(0);
    }

    let read_len = core::cmp::min(output.len() as u64, file_size - read_offset) as usize;
    let output = &mut output[..read_len];
    output.fill(0);

    let flags = le_u32(inode, 32);
    if flags & EXT4_EXTENTS_FL != 0 {
        copy_extent_tree_range(fs, &inode[40..100], file_size, read_offset, output)?;
    } else {
        copy_direct_blocks_range(fs, &inode[40..88], file_size, read_offset, output)?;
    }

    Ok(read_len)
}

fn copy_extent_tree_range(
    fs: &Ext4Fs,
    root: &[u8],
    file_size: u64,
    read_offset: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let depth = extent_depth(root)?;

    if depth == 0 {
        return copy_extent_leaf_range(fs, root, file_size, read_offset, output);
    }

    if depth != 1 {
        return Err("unsupported EXT4 extent depth");
    }

    let entries = extent_entries(root)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > root.len() {
            return Err("invalid EXT4 extent index");
        }

        let leaf = extent_index_leaf(&root[offset..offset + 12]);
        let mut leaf_block = [0u8; MAX_BLOCK_SIZE];
        read_fs_block(fs, leaf, &mut leaf_block)?;
        copy_extent_leaf_range(fs, &leaf_block[..fs.block_size], file_size, read_offset, output)?;

        index += 1;
    }

    Ok(())
}

fn copy_extent_leaf_range(
    fs: &Ext4Fs,
    node: &[u8],
    file_size: u64,
    read_offset: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    if extent_depth(node)? != 0 {
        return Err("invalid EXT4 extent leaf");
    }

    let entries = extent_entries(node)? as usize;
    let mut index = 0usize;
    while index < entries {
        let offset = 12 + index * 12;
        if offset + 12 > node.len() {
            return Err("invalid EXT4 extent entry");
        }

        let logical = le_u32(node, offset) as u64;
        let len = (le_u16(node, offset + 4) & 0x7fff) as u64;
        let physical = ((le_u16(node, offset + 6) as u64) << 32) | le_u32(node, offset + 8) as u64;

        let mut block_index = 0u64;
        while block_index < len {
            let file_offset = (logical + block_index) * fs.block_size as u64;
            if file_offset >= file_size {
                break;
            }

            let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
            copy_data_block_range(
                fs,
                physical + block_index,
                file_offset,
                valid_len,
                read_offset,
                output,
            )?;

            block_index += 1;
        }

        index += 1;
    }

    Ok(())
}

fn copy_direct_blocks_range(
    fs: &Ext4Fs,
    i_block: &[u8],
    file_size: u64,
    read_offset: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let mut index = 0usize;
    while index < 12 {
        let block_no = le_u32(i_block, index * 4) as u64;
        if block_no == 0 {
            break;
        }

        let file_offset = index as u64 * fs.block_size as u64;
        if file_offset >= file_size {
            break;
        }

        let valid_len = remaining_block_len(file_size, file_offset, fs.block_size);
        copy_data_block_range(fs, block_no, file_offset, valid_len, read_offset, output)?;

        index += 1;
    }

    Ok(())
}

fn copy_data_block_range(
    fs: &Ext4Fs,
    block_no: u64,
    file_offset: u64,
    valid_len: usize,
    read_offset: u64,
    output: &mut [u8],
) -> Result<(), &'static str> {
    let block_start = file_offset;
    let block_end = file_offset + valid_len as u64;
    let read_start = read_offset;
    let read_end = read_offset + output.len() as u64;

    let overlap_start = if block_start > read_start {
        block_start
    } else {
        read_start
    };
    let overlap_end = if block_end < read_end { block_end } else { read_end };

    if overlap_start >= overlap_end {
        return Ok(());
    }

    let copy_len = (overlap_end - overlap_start) as usize;
    let src_offset = (overlap_start - block_start) as usize;
    let dest_offset = (overlap_start - read_start) as usize;

    let mut buffer = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, block_no, &mut buffer)?;
    output[dest_offset..dest_offset + copy_len]
        .copy_from_slice(&buffer[src_offset..src_offset + copy_len]);

    Ok(())
}

fn read_disk_bytes(mut offset: u64, mut output: &mut [u8]) -> Result<(), &'static str> {
    let mut sector = [0u8; block::BLOCK_SIZE];

    while !output.is_empty() {
        let sector_no = offset / block::BLOCK_SIZE as u64;
        let sector_offset = (offset % block::BLOCK_SIZE as u64) as usize;

        if !block::read_sector(sector_no, &mut sector) {
            return Err("disk read failed");
        }

        let copy_len = core::cmp::min(block::BLOCK_SIZE - sector_offset, output.len());
        let (head, tail) = output.split_at_mut(copy_len);
        head.copy_from_slice(&sector[sector_offset..sector_offset + copy_len]);

        offset += copy_len as u64;
        output = tail;
    }

    Ok(())
}

fn extent_entries(node: &[u8]) -> Result<u16, &'static str> {
    if node.len() < 12 || le_u16(node, 0) != EXT4_EXTENT_MAGIC {
        return Err("invalid EXT4 extent header");
    }

    Ok(le_u16(node, 2))
}

fn extent_depth(node: &[u8]) -> Result<u16, &'static str> {
    if node.len() < 12 || le_u16(node, 0) != EXT4_EXTENT_MAGIC {
        return Err("invalid EXT4 extent header");
    }

    Ok(le_u16(node, 6))
}

fn extent_index_leaf(index: &[u8]) -> u64 {
    let lo = le_u32(index, 4) as u64;
    let hi = le_u16(index, 8) as u64;
    (hi << 32) | lo
}

fn inode_size(inode: &[u8]) -> u64 {
    let lo = le_u32(inode, 4) as u64;
    let hi = le_u32(inode, 108) as u64;
    (hi << 32) | lo
}

fn remaining_block_len(file_size: u64, file_offset: u64, block_size: usize) -> usize {
    let remaining = file_size - file_offset;
    if remaining < block_size as u64 {
        remaining as usize
    } else {
        block_size
    }
}

fn is_basic_test_script(name: &[u8]) -> bool {
    name == b"basic_testcode.sh"
}

fn is_dot_entry(name: &[u8]) -> bool {
    name == b"." || name == b".."
}

fn contains_byte(buffer: &[u8], target: u8) -> bool {
    let mut index = 0usize;
    while index < buffer.len() {
        if buffer[index] == target {
            return true;
        }
        index += 1;
    }
    false
}

fn is_elf(buffer: &[u8]) -> bool {
    buffer.len() >= 4 && buffer[0] == 0x7f && buffer[1] == b'E' && buffer[2] == b'L' && buffer[3] == b'F'
}

fn emit_group_markers_from_script(script: &[u8]) -> usize {
    let mut emitted = 0usize;
    let mut index = 0usize;

    while index + GROUP_MARKER_PREFIX.len() <= script.len() {
        if starts_with_at(script, index, GROUP_MARKER_PREFIX) {
            let marker_start = index;
            let search_start = index + GROUP_MARKER_PREFIX.len();

            if let Some(marker_end_start) = find_bytes_from(script, MARKER_END, search_start) {
                let marker_end = marker_end_start + MARKER_END.len();
                print_name(&script[marker_start..marker_end]);
                crate::println!();
                emitted += 1;
                index = marker_end;
                continue;
            }
        }

        index += 1;
    }

    emitted
}

fn emit_group_start_from_script_or_fallback(name: &[u8], script: &[u8]) {
    if emit_first_group_start_from_script(script) {
        return;
    }

    emit_fallback_group_start(name);
}

fn emit_first_group_start_from_script(script: &[u8]) -> bool {
    let mut index = 0usize;

    while index + GROUP_START_PREFIX.len() <= script.len() {
        if starts_with_at(script, index, GROUP_START_PREFIX) {
            let search_start = index + GROUP_START_PREFIX.len();
            if let Some(marker_end_start) = find_bytes_from(script, MARKER_END, search_start) {
                let marker_end = marker_end_start + MARKER_END.len();
                print_name(&script[index..marker_end]);
                crate::println!();
                return true;
            }
        }

        index += 1;
    }

    false
}

fn emit_fallback_group_markers(name: &[u8]) {
    emit_fallback_group_start(name);
    emit_fallback_group_end(name);
}

fn emit_fallback_group_start(name: &[u8]) {
    let group_len = name.len() - TEST_SCRIPT_SUFFIX.len();
    let group = &name[..group_len];

    crate::print!("#### OS COMP TEST GROUP START ");
    print_name(group);
    crate::println!(" ####");
}

fn emit_fallback_group_end(name: &[u8]) {
    let group_len = name.len() - TEST_SCRIPT_SUFFIX.len();
    let group = &name[..group_len];

    crate::print!("#### OS COMP TEST GROUP END ");
    print_name(group);
    crate::println!(" ####");
}

fn set_external_group_from_script_name(name: &[u8]) {
    let group_len = name.len() - TEST_SCRIPT_SUFFIX.len();
    crate::loader::set_external_group(&name[..group_len]);
}

fn set_external_group_from_script(name: &[u8], script: &[u8]) {
    if let Some(group) = first_group_start_name(script) {
        crate::loader::set_external_group(group);
    } else {
        set_external_group_from_script_name(name);
    }
}

fn first_group_start_name(script: &[u8]) -> Option<&[u8]> {
    let mut index = 0usize;

    while index + GROUP_START_PREFIX.len() <= script.len() {
        if starts_with_at(script, index, GROUP_START_PREFIX) {
            let group_start = index + GROUP_START_PREFIX.len();
            if let Some(marker_end_start) = find_bytes_from(script, MARKER_END, group_start) {
                let mut group_end = marker_end_start;
                while group_end > group_start && script[group_end - 1] == b' ' {
                    group_end -= 1;
                }
                if group_end > group_start {
                    return Some(&script[group_start..group_end]);
                }
            }
        }

        index += 1;
    }

    None
}

fn starts_with_at(buffer: &[u8], offset: usize, pattern: &[u8]) -> bool {
    offset + pattern.len() <= buffer.len()
        && &buffer[offset..offset + pattern.len()] == pattern
}

fn find_bytes_from(buffer: &[u8], pattern: &[u8], start: usize) -> Option<usize> {
    if pattern.is_empty() || start >= buffer.len() {
        return None;
    }

    let mut index = start;
    while index + pattern.len() <= buffer.len() {
        if &buffer[index..index + pattern.len()] == pattern {
            return Some(index);
        }

        index += 1;
    }

    None
}

fn print_name(name: &[u8]) {
    for &byte in name {
        crate::sbi::console_putchar(byte as usize);
    }
}

fn le_u16(buffer: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buffer[offset], buffer[offset + 1]])
}

fn le_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}
