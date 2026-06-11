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
- 当前已能读取 `basic_testcode.sh`，跟随 `cd` 和嵌套 `run-all.sh`，从
  `tests="..."` 中解析首个命令 `basic/brk`。
- `basic/brk` 会从 EXT4 复制到 tmpfs，由内置 `runtestcase` 通过
  `fork + execve + wait4` 执行；本地官方 `test_runner.py` 已解析为
  `test_brk 3/3`，说明 Titanix 主线已经具备产生 basic 分数的最小闭环。
- 该结果尚未经过新一轮线上评测，不能把本地 `3/3` 写成官方线上分数。

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
oscomp: first basic command musl/basic/brk
#### OS COMP TEST GROUP START basic-musl ####
========== START test_brk ==========
Before alloc,heap pos: ...
After alloc,heap pos: ...
Alloc again,heap pos: ...
========== END test_brk ==========
#### OS COMP TEST GROUP END basic-musl ####
[kernel] kernel will shutdown...
```

使用官方 `test_runner.py` 解析日志时，`test_brk` 应为 `3/3`。

## 关键技术边界

- Titanix 原生文件系统是 FAT32；当前提交模式以 tmpfs 作为根 VFS，并由
  `oscomp` 适配层独立只读探测官方 EXT4 测试盘。
- `oscomp` 尚未把整个 EXT4 挂入 Titanix VFS；当前只读脚本和首个 ELF，
  再把 ELF、argv 和结束标记复制为 tmpfs 中的 `/oscomp-*` 文件。
- 当前只执行 `run-all.sh` 的第一个测试名。继续扩展时必须保持测试串行，
  每个 ELF 都要 `wait4` 回收后才能启动下一个。
- Titanix 原文件名 `kernel/src/process/aux.rs` 与 Windows 保留名冲突，当前改为
  `aux_file.rs`，并通过 `#[path = "aux_file.rs"]` 保持模块名 `aux`。
- Titanix 原始 ELF 使用高虚拟地址，不能直接作为官方 `-kernel` 输入；根目录
  Makefile 的 wrapper ELF 流程不可随意删除。
- `kernel-la` 仍只是 RISC-V ELF 占位，不代表支持 LoongArch。
- 新主线依赖 nightly `2025-02-18`；升级工具链前必须完整回归构建和启动。

## 下一步唯一主线

1. 将 `run-all.sh` 的测试列表完整解析为命令队列。
2. 让 `runtestcase` 串行执行队列中的全部 basic ELF。
3. 每次只推进到第一个失败项，依据真实日志修复 syscall、VFS 或进程语义。
4. 保持 `test_brk 3/3` 和无测试盘主动退出回归。
5. basic 获得稳定线上分数后再推进 BusyBox。

不要在 basic 队列稳定前投入网络、图形、多核优化或展示功能。

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
- `/oscomp-first`、`/oscomp-argv`、`/oscomp-end` 是当前内核与 runner 的内部
  协议，修改其中任一方时必须同步修改另一方。
- basic START/END 区间内不要加入会伪装成 testcase 的调试输出。
