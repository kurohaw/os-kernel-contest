# 项目协作说明

本文档给队内成员和 Codex 使用。开始开发前先读这里，再看
`docs/progress.md`、`docs/test-matrix.md` 和 `docs/next-evaluation-roadmap.md`。

## 当前状态

- 当前开发分支：`codex/titanix-architecture`。
- 当前内核主体：`titanix/`，来源为往届开源作品 Titanix。
- 旧自建内核的官方 `basic=102` 版本永久保存在
  `codex/basic-102-archive`，不要删除或改写该分支。
- 新主线中的旧 `kernel/`、`user/` 和 `rCore-Tutorial-v3-main/`
  已移除，避免同时维护三套架构。
- 根目录 `make all` 已能离线构建 Titanix，并生成官方要求的 ELF
  `kernel-rv` 和临时占位 `kernel-la`。
- `kernel-rv` 已通过官方风格的 `256M`、单核 QEMU 启动验证。
- `titanix/kernel/src/oscomp.rs` 已能从 x0 virtio-blk 测试盘识别 EXT4，
  并优先命中 `musl/basic_testcode.sh`、`glibc/basic_testcode.sh` 或根目录
  `basic_testcode.sh`。
- 当前已能输出 basic START/END 并主动关机，但**尚未从 EXT4 加载并执行
  basic ELF**，因此当前 Titanix 主线还不应被视为已经恢复官方分数。

## 目录说明

- `titanix/kernel/`：当前唯一内核主体，包含进程、线程、异步执行器、内存、
  VFS、FAT32、tmpfs、网络、驱动和 syscall。
- `titanix/user/`：Titanix 内置用户程序，目前提交模式运行 `runtestcase`。
- `titanix/vendor/`：离线 Cargo 依赖。官方构建依赖它，不要随意删除。
- `titanix/dependencies/`：Titanix 使用的本地 RISC-V 等依赖源码。
- `titanix/docs/`：Titanix 上游架构与实现说明，阅读模块前优先参考。
- `titanix/LICENSE`：Titanix 的 GPLv3 许可证，必须保留。
- `docs/`：当前进度、测试矩阵、路线和来源说明。
- `office-test.txt`：官方提交说明原文参考，不要删除。

## 构建与运行

优先在 WSL/bash 中执行：

```bash
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic \
  -smp 1 -bios default -no-reboot
```

带官方风格测试盘：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic \
  -smp 1 -bios default \
  -drive file=/path/to/test.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot
```

根构建流程会：

1. 从非隐藏 `cargo-config/` 恢复 `.cargo/config.toml`。
2. 从非隐藏 `cargo-checksum.json` 恢复 vendor 所需的
   `.cargo-checksum.json`。
3. 使用 `titanix/vendor/` 离线编译用户态和内核。
4. 构建 `submit + tmpfs` 模式的 Titanix。
5. 将 Titanix 原始高虚拟地址内核转为 raw binary。
6. 再封装为物理加载地址 `0x80200000` 的可执行 ELF `kernel-rv`。

## 当前验收标准

无 EXT4 basic 测试盘时：

- `make all` 成功。
- `kernel-rv` 是 RISC-V executable ELF，入口为 `0x80200000`。
- QEMU 能看到 Titanix 启动画面，并主动关机。

有官方目录结构 EXT4 测试盘时：

```text
oscomp: found official basic script musl/basic_testcode.sh
#### OS COMP TEST GROUP START basic ####
oscomp: Titanix official basic entry reached
#### OS COMP TEST GROUP END basic ####
[kernel] kernel will shutdown...
```

这只代表已经进入 basic 入口，不代表执行了 basic ELF 或获得分数。

## 关键技术边界

- Titanix 原生文件系统是 FAT32；当前提交模式以 tmpfs 作为根 VFS，并由
  `oscomp` 适配层独立只读探测官方 EXT4 测试盘。
- `oscomp` 当前只识别 fixed path basic 脚本，尚未把 EXT4 文件挂入 Titanix
  VFS，也尚未读取脚本内容或执行盘上的 ELF。
- Titanix 原文件名 `kernel/src/process/aux.rs` 与 Windows 保留名冲突，当前改为
  `aux_file.rs`，并通过 `#[path = "aux_file.rs"]` 保持模块名 `aux`。
- Titanix 原始 ELF 使用高虚拟地址，不能直接作为官方 `-kernel` 输入；根目录
  Makefile 的 wrapper ELF 流程不可随意删除。
- `kernel-la` 仍只是 RISC-V ELF 占位，不代表支持 LoongArch。
- 新主线依赖 nightly `2025-02-18`；升级工具链前必须完整回归构建和启动。

## 下一步唯一主线

1. 让 `oscomp` 读取命中的 `basic_testcode.sh` 内容。
2. 将官方 EXT4 普通文件接入 Titanix VFS，或建立受控的只读外部文件接口。
3. 从 EXT4 读取 basic ELF，并使用 Titanix `Process/MemorySpace` 创建进程。
4. 运行第一个真实 basic ELF，依据日志修复 ABI。
5. 恢复 Titanix 主线的官方 basic 分数，再推进 BusyBox。

不要在执行第一个官方 basic ELF 前投入网络、图形、多核优化或展示功能。

## 协作注意事项

- 远端同步由用户本人执行；Codex 不自动执行用户指定的 rebase pull 命令。
- Titanix 是 GPLv3 往届作品。必须保留许可证、原作者和来源说明，并持续记录
  本队的适配与新增贡献。
- 使用完整往届作品作为基线前，队伍应向指导老师或组委会确认复用边界。
- 队员必须能够解释 Titanix 的启动、异步执行器、进程、MemorySpace、VFS 和
  syscall 路径；不能只把它当作不可理解的黑盒。
- 不要把 `codex/titanix-architecture` 直接误认为保分分支。需要线上保分时，
  使用已验证的 `codex/basic-102-archive`。
- 不要提交生成产物：`target/`、`kernel-rv`、`kernel-la`、`disk.img`。
- 不要删除 `titanix/vendor/`、`titanix/dependencies/`、许可证或来源说明。
- 不要删除 vendor 内的非隐藏 `cargo-checksum.json`；官方过滤隐藏文件后，
  根 Makefile 依靠它恢复 Cargo 校验信息。
- 不要恢复 `aux.rs` 文件名，否则 Windows 工作区无法检出。
- 改根 Makefile、wrapper ELF、virtio、EXT4 或 `oscomp` 后，必须运行官方风格
  QEMU 回归。
- 当前 basic START/END 区间内不要加入调试噪声，避免后续影响官方解析。
