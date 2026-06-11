//! Adapter for the OS competition's read-only EXT4 test disk.

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    driver::BLOCK_DEVICE,
    fs::{File, InodeMode, FILE_SYSTEM_MANAGER},
    println,
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
const MAX_TEST_FILE_SIZE: usize = 4 * 1024 * 1024;
const MAX_SCRIPT_DEPTH: usize = 4;
const EXECUTABLE_FILE: &str = "oscomp-first";
const ARGV_FILE: &str = "oscomp-argv";
const END_MARKER_FILE: &str = "oscomp-end";

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
    argv: Vec<String>,
}

struct BasicPlan {
    command: BasicCommand,
    start_marker: String,
    end_marker: String,
}

/// Read the official basic script, stage its first ELF in tmpfs, and enter the group.
pub fn init() {
    let fs = match read_superblock() {
        Ok(fs) => fs,
        Err(message) => {
            println!("oscomp: {}", message);
            return;
        }
    };

    let candidates: [(&str, &[&[u8]]); 3] = [
        ("musl/basic_testcode.sh", &[b"musl", b"basic_testcode.sh"]),
        ("glibc/basic_testcode.sh", &[b"glibc", b"basic_testcode.sh"]),
        ("basic_testcode.sh", &[b"basic_testcode.sh"]),
    ];

    for (label, path) in candidates {
        if let Ok(Some(info)) = lookup_path(&fs, path) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                let plan = match build_basic_plan(&fs, label) {
                    Ok(plan) => plan,
                    Err(message) => {
                        println!("oscomp: cannot parse {}: {}", label, message);
                        continue;
                    }
                };
                let elf = match read_file(&fs, &plan.command.executable_path) {
                    Ok(elf) => elf,
                    Err(message) => {
                        println!(
                            "oscomp: cannot read {}: {}",
                            plan.command.executable_path, message
                        );
                        continue;
                    }
                };
                if elf.get(..4) != Some(b"\x7fELF") {
                    println!(
                        "oscomp: {} is not an ELF file",
                        plan.command.executable_path
                    );
                    continue;
                }
                if let Err(message) = install_plan(&elf, &plan) {
                    println!("oscomp: cannot stage basic testcase: {}", message);
                    continue;
                }

                println!("oscomp: found official basic script {}", label);
                println!(
                    "oscomp: first basic command {}",
                    plan.command.executable_path
                );
                println!("{}", plan.start_marker);
                return;
            }
        }
    }

    println!("oscomp: official basic script not found");
}

fn build_basic_plan(fs: &Ext4, script_path: &str) -> Result<BasicPlan, &'static str> {
    let script = read_text_file(fs, script_path)?;
    let start_marker = find_group_marker(&script, "START")
        .unwrap_or_else(|| "#### OS COMP TEST GROUP START basic ####".to_string());
    let end_marker = find_group_marker(&script, "END")
        .unwrap_or_else(|| "#### OS COMP TEST GROUP END basic ####".to_string());
    let command = find_first_command(fs, script_path, 0)?;
    Ok(BasicPlan {
        command,
        start_marker,
        end_marker,
    })
}

fn find_first_command(
    fs: &Ext4,
    script_path: &str,
    depth: usize,
) -> Result<BasicCommand, &'static str> {
    if depth >= MAX_SCRIPT_DEPTH {
        return Err("nested script limit reached");
    }
    let script = read_text_file(fs, script_path)?;
    let mut cwd = parent_path(script_path);

    if let Some(tests) = quoted_assignment(&script, "tests") {
        if let Some(test) = tests.split_whitespace().next() {
            let executable_path = resolve_path(&cwd, test);
            return Ok(BasicCommand {
                executable_path,
                argv: vec![format_command_arg(test)],
            });
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
            return find_first_command(fs, &resolved, depth + 1);
        }
        if let Ok(Some(info)) = lookup_path_str(fs, &resolved) {
            if info.mode & EXT4_MODE_TYPE_MASK == EXT4_S_IFREG {
                return Ok(BasicCommand {
                    executable_path: resolved,
                    argv,
                });
            }
        }
    }

    Err("no executable command found")
}

fn install_plan(elf: &[u8], plan: &BasicPlan) -> Result<(), &'static str> {
    install_tmpfs_file(EXECUTABLE_FILE, elf)?;

    let mut argv_data = Vec::new();
    for arg in &plan.command.argv {
        argv_data.extend_from_slice(arg.as_bytes());
        argv_data.push(0);
    }
    install_tmpfs_file(ARGV_FILE, &argv_data)?;
    install_tmpfs_file(END_MARKER_FILE, plan.end_marker.as_bytes())
}

fn install_tmpfs_file(name: &str, data: &[u8]) -> Result<(), &'static str> {
    let root = FILE_SYSTEM_MANAGER.root_inode();
    let inode = root
        .mknod_v(name, InodeMode::FileREG, None)
        .map_err(|_| "cannot create tmpfs inode")?;
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

fn format_command_arg(arg: &str) -> String {
    if arg.starts_with("./") {
        arg.to_string()
    } else {
        alloc::format!("./{}", arg)
    }
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
