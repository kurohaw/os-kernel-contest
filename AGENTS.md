# 项目协作说明

本文档给队内成员和 Codex 使用。开始开发前先读这里，再看 `docs/progress.md` 和 `docs/test-matrix.md`。

## 当前项目状态

- 当前仓库：`D:\os-kernel-contest`。
- 当前阶段：初赛开发期，优先目标是打通官方测试和最小 Linux ABI。
- 当前重点不是继续零散补自测 syscall，而是用官方 ELF/脚本实际运行结果反推缺失 ABI，再逐组推进 basic、busybox、lua、libctest 等测试。
- 2026-06-06 官方评测结果：提交被 Accepted，但总分 0.0。历史原因判断为官方测例入口和 Linux ABI 当时尚未打通。
- 2026-06-06 本地 official basic 结果：使用官方 `pre-2025` basic 源码手工编译 RISC-V ELF，打包为无分区 EXT4 镜像后，QEMU 运行日志经官方 `test_runner.py` 解析为 `102/102`。
- 当前 `main` 已完成 RISC-V 官方提交入口适配：根目录 `make all` 能生成 ELF `kernel-rv`，并可用官方风格 QEMU 命令启动和主动退出。
- 当前已能识别官方风格挂载的 virtio-blk 测试盘，从无分区 EXT4 扫描并读取 `*_testcode.sh`，输出官方测试组 START/END 标记；能跳过 `busybox echo`、处理 `cd`、读取嵌套 `.sh`，把脚本中的多个真实 ELF 命令排队串行运行并传入 argv；外部 ELF 支持最小 `argc/argv/envp/auxv` 启动栈、EXT4 普通文件读取（含子目录路径）、`brk` 增长映射真实用户堆页、official basic 常用 Linux syscall 编号兼容，以及最小 `clone/fork/execve/wait4/nanosleep` 路径。
- `kernel-la` 目前只是临时占位文件，不代表已经支持 LoongArch。

## 目录说明

- `kernel/`：自建 Rust/RISC-V 内核，当前主要实现启动、trap、分页、任务、syscall、最小文件描述符层。
- `user/`：自建用户态测试程序，当前有 `app0` 和 `app1`，会编译为裸二进制并由内核 `include_bytes!` 嵌入。
- `docs/`：开发进度、测试矩阵、参考来源和路线说明。阶段完成后必须更新相关文档。
- `rCore-Tutorial-v3-main/`：学习和参考 baseline，不是当前提交内核主体。不要直接在里面改比赛实现。
- `office-test.txt`：官方提交说明原文参考，不要当垃圾文件直接删除；后续可整理进 `docs/`。

## 官方评测硬约束

- 根目录必须保留 `Makefile`，评测机会执行 `make all`。
- `make all` 应生成 ELF 格式的 `kernel-rv` 和 `kernel-la`。当前只真正支持 `kernel-rv`。
- RISC-V 评测启动方式类似：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default -no-reboot
```

- 评测机会挂载一个无分区表 EXT4 测试盘，里面有预编译 ELF 程序和 `xxxxx_testcode.sh` 脚本。
- 内核启动后需要主动扫描测试盘，串行运行测试点，并把结果输出到屏幕。
- 如果暂时跳过某个测试点，也要按官方格式输出测试组起止提示，例如 `#### OS COMP TEST GROUP START basic ####`。
- 所有测试完成后必须主动关机退出 QEMU，避免被判定为评测时间过长。
- 评测系统 clone 后会过滤隐藏文件和目录。不能只依赖仓库里的 `.cargo`；必要配置要放在非隐藏目录，例如 `kernel/cargo-config/` 和 `user/cargo-config/`，构建时再恢复。
- 不要对评测机做任何逆向工程。

## 常用命令

优先在 WSL/bash 中运行：

```bash
cd /mnt/d/os-kernel-contest
git pull --rebase gitlab main
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default -no-reboot
```

旧 RustSBI loader 本地流程仍保留：

```bash
cd /mnt/d/os-kernel-contest/kernel
make run-loader
```

只构建用户程序：

```bash
cd /mnt/d/os-kernel-contest/user
make build
```

退出旧的交互式 QEMU：

```text
Ctrl + A
X
```

## 当前 smoke test 标准

运行官方风格 QEMU 后，至少应看到：

```text
Hello kernel
kernel started
app0: hello from write
app0: read hello ok
app1: hello from write
app1: read hello ok
all tasks exited
```

出现 `all tasks exited` 后，QEMU 应主动退出。

官方 basic 本地通过标准：

- QEMU 输出完整 `#### OS COMP TEST GROUP START basic ####` 与 `#### OS COMP TEST GROUP END basic ####`。
- 官方 `test_runner.py` 解析日志得到 `TOTAL 102 / 102`。
- 无测试盘的 `app0/app1` 回归仍能正常退出，避免只为 official basic 破坏内置 smoke。

## 已支持与已知限制

当前已支持的最小 syscall：

- `test = 0`
- `exit = 1`
- `yield = 2`
- `openat = 56`
- `close = 57`
- `read = 63`
- `write = 64`
- `fstat = 80`
- `exit/exit_group = 93/94`
- `nanosleep = 101`
- `sched_yield = 124`
- `times = 153`
- `uname = 160`
- `gettimeofday = 169`
- `getpid = 172`
- `getppid = 173`
- `clone/fork = 220`
- `brk = 214`
- `munmap = 215`
- `mmap = 222`
- `execve = 221`
- `wait4 = 260`

当前限制：

- 无测试盘时仍运行内嵌 `app0/app1` 作为回归；挂载测试盘且脚本中能找到根目录 ELF 时，会改为运行外部 ELF。
- `brk` 增长时会映射用户零页；缩小时暂不回收页。
- 文件系统支持 `/dev/null`、内嵌只读 `/hello.txt`，以及测试盘 EXT4 普通文件的只读 `openat/read/fstat`，路径解析已支持多级子目录。
- fd 表仍是全局表，尚未按进程隔离。
- virtio-blk 与 EXT4 目前支持只读扫描测试脚本、读取脚本内容、输出测试组 START/END 标记、记录脚本命令队列、串行读取 ELF，以及对用户态暴露普通文件读取。
- ELF loader 目前只支持把整个 ELF 读入 4 MiB 内核缓冲并映射 `PT_LOAD` 段；已构造脚本命令 argv、空 `envp` 和基础 `auxv`，并支持一个测试组内多个外部 ELF 串行运行；但尚未支持动态链接器或解释器路径。
- `clone/fork/execve/wait4` 是为 official basic 打通的最小实现：clone/fork 共享地址空间，fork 复制用户栈，clone 使用每任务静态用户栈，wait4 通过重试 syscall 阻塞等待；这还不是完整进程隔离模型。
- `nanosleep` 为最小 busy-wait/cap 实现，不保证真实 POSIX 时间精度。
- 尚未真正支持 LoongArch。

## 下一步优先级

1. 在官方平台重新提交，确认 `basic` 是否从 0 分变为本地一致的通过结果。
2. 用官方 busybox/lua/libctest ELF 运行结果反推下一批缺失 Linux ABI/syscall。
3. 补 per-process fd table，避免多程序、fork/exec、pipe/close 路径相互串扰。
4. 将最小 clone/fork/exec/wait4 升级为更接近 Linux 的进程模型和父子资源模型。
5. 尽快替换当前“陷入后借用户栈跑内核”的临时做法，补独立内核栈。

不要在测试盘入口没打通前，把大量时间花在展示性功能、网络、图形界面或复杂优化上。

## 协作与提交习惯

- 每天开始开发前先执行 `git pull --rebase gitlab main`。
- 当前用户要求：`git pull --rebase gitlab main` 由用户本人执行，Codex 不要自动执行；需要同步时只提醒用户。
- 自己改完及时 commit，commit message 使用 Conventional Commits，描述部分可用中文。
- push 前需要确认已经与 `gitlab/main` 同步；`git pull --rebase gitlab main` 仍由用户本人执行，Codex 不自动执行。
- 不要随便对共享 `main` 分支使用 `git push --force`。
- 多人同时开发时，先在群里说清楚自己要改的模块，避免同时改同一个文件。
- 每个阶段完成后先验证，再提交，再更新 `docs/progress.md` 和 `docs/test-matrix.md`。
- 不要把多个大里程碑混到同一个 commit。
- 不要提交生成产物：`target/`、`kernel-rv`、`kernel-la`、`exec.out`、`disk.img`、`disk-la.img`。
- 不要直接运行 `git clean -fdX`，它可能删除有用的本地说明、bootloader 或用户程序生成文件。

## 队友注意事项

- Windows PowerShell 里可能没有 `make`，建议用 WSL/bash。
- 这台机器上 `rg.exe` 可能被系统拒绝运行，可改用 PowerShell `Select-String` 或 WSL 工具。
- PowerShell 读取中文时注意编码，必要时使用 `Get-Content -Encoding UTF8`。
- `user/build/app0.bin` 和 `user/build/app1.bin` 是生成物，但内核 `include_bytes!` 会用到；删除后必须重新 `make all`。
- `rCore-Tutorial-v3-main/bootloader/rustsbi-qemu.bin` 旧本地流程会用到，不要随手删。
- 如果复用 rCore 代码，要保留原许可证和来源说明。
- 不要直接复制 Phoenix、Starry Mix 或 NoAxiom 代码；只能参考设计和路线。
- 官方评测看屏幕输出，日志格式会影响得分。改输出前要确认不会破坏测试识别。
- 如果新增 syscall，请同步更新 `docs/test-matrix.md`，写明编号、当前行为、验证方式和限制。
- 如果改启动、构建、磁盘或文件系统，请优先验证 `make all` 和官方风格 QEMU 命令。
