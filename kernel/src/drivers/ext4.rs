use super::block;
use core::ptr::addr_of_mut;

const EXT4_SUPER_OFFSET: u64 = 1024;
const EXT4_SUPER_MAGIC: u16 = 0xef53;
const EXT4_EXTENTS_FL: u32 = 0x0008_0000;
const EXT4_EXTENT_MAGIC: u16 = 0xf30a;
const EXT4_ROOT_INO: u32 = 2;
const EXT4_S_IFDIR: u16 = 0x4000;
const EXT4_MODE_TYPE_MASK: u16 = 0xf000;
const MAX_BLOCK_SIZE: usize = 4096;
const MIN_INODE_SIZE: usize = 128;
const INODE_PARSE_SIZE: usize = 160;
const GROUP_DESC_PARSE_SIZE: usize = 64;
const EXT4_NAME_MAX: usize = 255;
const TEST_SCRIPT_SUFFIX: &[u8] = b"_testcode.sh";
const SCRIPT_BUFFER_SIZE: usize = 16 * 1024;
const GROUP_MARKER_PREFIX: &[u8] = b"#### OS COMP TEST GROUP ";
const GROUP_START_PREFIX: &[u8] = b"#### OS COMP TEST GROUP START ";
const MARKER_END: &[u8] = b"####";

static mut SCRIPT_BUFFER: [u8; SCRIPT_BUFFER_SIZE] = [0; SCRIPT_BUFFER_SIZE];

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
}

pub fn init() {
    if !block::is_ready() {
        return;
    }

    match scan_test_scripts() {
        Ok(count) => {
            crate::println!("ext4: found {} test script(s)", count);
        }
        Err(message) => {
            crate::println!("ext4: {}", message);
        }
    }
}

fn scan_test_scripts() -> Result<usize, &'static str> {
    let fs = read_superblock()?;
    let inode_table = read_inode_table_block(&fs, EXT4_ROOT_INO)?;

    let mut inode = [0u8; INODE_PARSE_SIZE];
    read_inode(&fs, inode_table, EXT4_ROOT_INO, &mut inode)?;

    let mode = le_u16(&inode, 0);
    if mode & EXT4_MODE_TYPE_MASK != EXT4_S_IFDIR {
        return Err("root inode is not a directory");
    }

    let root_size = inode_size(&inode);
    let flags = le_u32(&inode, 32);
    let mut found = 0usize;

    if flags & EXT4_EXTENTS_FL != 0 {
        scan_extent_tree(&fs, &inode[40..100], root_size, &mut found)?;
    } else {
        scan_direct_blocks(&fs, &inode[40..88], root_size, &mut found)?;
    }

    Ok(found)
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
    found: &mut usize,
) -> Result<(), &'static str> {
    let depth = extent_depth(root)?;

    if depth == 0 {
        return scan_extent_leaf(fs, root, file_size, found);
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
        scan_extent_leaf(fs, &leaf_block[..fs.block_size], file_size, found)?;

        index += 1;
    }

    Ok(())
}

fn scan_extent_leaf(
    fs: &Ext4Fs,
    node: &[u8],
    file_size: u64,
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
            scan_dir_data_block(fs, physical + block_index, valid_len, found)?;

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
        scan_dir_data_block(fs, block_no, valid_len, found)?;

        index += 1;
    }

    Ok(())
}

fn scan_dir_data_block(
    fs: &Ext4Fs,
    block_no: u64,
    valid_len: usize,
    found: &mut usize,
) -> Result<(), &'static str> {
    let mut buffer = [0u8; MAX_BLOCK_SIZE];
    read_fs_block(fs, block_no, &mut buffer)?;
    scan_dir_entries(fs, &buffer[..valid_len], found);
    Ok(())
}

fn scan_dir_entries(fs: &Ext4Fs, block: &[u8], found: &mut usize) {
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
            if is_test_script(name) {
                handle_test_script(fs, inode, name, found);
            }
        }

        offset += rec_len;
    }
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

fn handle_test_script(fs: &Ext4Fs, inode_no: u32, name: &[u8], found: &mut usize) {
    *found += 1;

    crate::print!("oscomp: found test script ");
    print_name(name);
    crate::println!();

    let mut inode = [0u8; INODE_PARSE_SIZE];
    let read_result = read_inode_table_block(fs, inode_no)
        .and_then(|inode_table| read_inode(fs, inode_table, inode_no, &mut inode));

    if read_result.is_err() {
        emit_fallback_group_markers(name);
        return;
    }

    let file_size = inode_size(&inode);
    let read_len = core::cmp::min(file_size as usize, SCRIPT_BUFFER_SIZE);
    let script = unsafe {
        core::slice::from_raw_parts_mut(addr_of_mut!(SCRIPT_BUFFER) as *mut u8, read_len)
    };

    if read_inode_data(fs, &inode, script).is_err() {
        emit_fallback_group_markers(name);
        return;
    }

    if try_load_first_elf_from_script(fs, script) {
        emit_group_start_from_script_or_fallback(name, script);
        set_external_group_from_script_name(name);
        return;
    }

    if emit_group_markers_from_script(script) == 0 {
        emit_fallback_group_markers(name);
    }
}

fn try_load_first_elf_from_script(fs: &Ext4Fs, script: &[u8]) -> bool {
    if crate::loader::has_external_app() {
        return false;
    }

    let mut index = 0usize;
    while index + 2 <= script.len() {
        if script[index] == b'.' && script[index + 1] == b'/' {
            let path_start = index + 2;
            let path_end = find_token_end(script, path_start);
            let path = &script[path_start..path_end];

            if is_root_candidate_path(path) && try_load_root_elf(fs, path) {
                return true;
            }

            index = path_end;
        } else {
            index += 1;
        }
    }

    false
}

fn try_load_root_elf(fs: &Ext4Fs, name: &[u8]) -> bool {
    let info = match lookup_root_file(fs, name) {
        Ok(Some(info)) => info,
        _ => return false,
    };

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

    crate::print!("loader: selected external ELF ");
    print_name(name);
    crate::println!();
    crate::loader::set_external_app(read_len);

    true
}

fn lookup_root_file(fs: &Ext4Fs, target: &[u8]) -> Result<Option<FileInfo>, &'static str> {
    let inode_table = read_inode_table_block(fs, EXT4_ROOT_INO)?;
    let mut inode = [0u8; INODE_PARSE_SIZE];
    read_inode(fs, inode_table, EXT4_ROOT_INO, &mut inode)?;

    let root_size = inode_size(&inode);
    let found_inode_no = if le_u32(&inode, 32) & EXT4_EXTENTS_FL != 0 {
        find_in_extent_tree(fs, &inode[40..100], root_size, target)?
    } else {
        find_in_direct_blocks(fs, &inode[40..88], root_size, target)?
    };

    let inode_no = match found_inode_no {
        Some(inode_no) => inode_no,
        None => return Ok(None),
    };

    let file_inode_table = read_inode_table_block(fs, inode_no)?;
    let mut file_inode = [0u8; INODE_PARSE_SIZE];
    read_inode(fs, file_inode_table, inode_no, &mut file_inode)?;

    Ok(Some(FileInfo {
        inode_no,
        size: inode_size(&file_inode),
    }))
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

fn is_test_script(name: &[u8]) -> bool {
    name.ends_with(TEST_SCRIPT_SUFFIX)
}

fn find_token_end(buffer: &[u8], start: usize) -> usize {
    let mut index = start;
    while index < buffer.len() {
        let byte = buffer[index];
        if byte == 0
            || byte == b' '
            || byte == b'\n'
            || byte == b'\r'
            || byte == b'\t'
            || byte == b'"'
            || byte == b'\''
            || byte == b';'
            || byte == b'|'
        {
            break;
        }

        index += 1;
    }

    index
}

fn is_root_candidate_path(path: &[u8]) -> bool {
    !path.is_empty()
        && !path.ends_with(TEST_SCRIPT_SUFFIX)
        && !path.ends_with(b".sh")
        && !contains_byte(path, b'/')
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
