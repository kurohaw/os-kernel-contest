# 官方评测后续路线

本文档给队友和 Codex 使用。当前线上 `basic=102` 已确认，后续开发目标是保住 basic 分数，再逐步打开 busybox、lua、libctest 等测试组。

## 当前结论

- 官方线上 0 分阻塞已经解除，2026-06-08 评测中 `basic` 得分 102。
- fixed path basic 入口已经在评测机生效：优先探测 `musl/basic_testcode.sh`，再探测 `glibc/basic_testcode.sh`，最后探测根目录 `basic_testcode.sh`。
- 当前不要一次性打开所有官方测试组。下一阶段应先保 basic，再单独推进 busybox。
- 后续工作以真实官方 ELF 和脚本运行日志为依据，不靠零散自测猜测 syscall 行为。

## 0. 当前 git 前置事项

当前本地已有文档提交，push 曾因远端有新提交被拒绝。同步由用户本人执行，Codex 不自动运行：

```bash
git pull --rebase gitlab main
```

rebase 成功后再 push。若出现冲突，优先保留以下事实记录：

- `basic=102` 已在线上确认。
- fixed path basic 入口有效。
- 下一步转为保 basic 后推进 busybox/lua/libctest。

## 1. 固定 basic 保分基线

目标：任何后续代码改动都不能让 `basic=102` 回退。

必须保留：

- fixed path 优先级：`musl/basic_testcode.sh` -> `glibc/basic_testcode.sh` -> `basic_testcode.sh`。
- basic 只自动跑一个 libc 目录，避免 glibc/musl 重复输出污染解析。
- `#### OS COMP TEST GROUP START ... ####` 到 `END` 之间不要加入调试输出。
- 无测试盘时仍运行内嵌 `app0/app1` smoke，并主动退出 QEMU。

每次改以下模块前后都要复测 basic：

- 启动入口和 `make all`。
- virtio-blk、EXT4、脚本扫描。
- ELF loader、argv/envp/auxv。
- syscall 分发。
- fd、路径、进程模型。
- 日志输出。

最低验证标准：

```bash
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default -no-reboot
```

官方目录结构 basic 回归仍应由官方 `test_runner.py` 解析为 `TOTAL 102 / 102`。

## 2. 加测试组选择器

目标：在不破坏官方 basic 默认行为的前提下，允许本地只跑 busybox。

当前状态：已完成。默认 `make all` 运行 basic；`make all TEST_GROUP=busybox` 构建 busybox 模式。

推荐行为：

- 官方默认模式只跑 basic，保住线上 102。
- 本地开发模式可以切到 busybox，只探测：
  - `musl/busybox_testcode.sh`
  - `glibc/busybox_testcode.sh`
  - `busybox_testcode.sh`
- busybox 本地出现稳定新增通过项后，再考虑官方默认改为 `basic + busybox`。

实现建议：

- 在测试脚本扫描处加入 allowlist。
- 默认 allowlist 为 `["basic"]`。
- 本地调试时可临时改为 `["busybox"]` 或 `["basic", "busybox"]`。
- 不建议一开始打开所有 `*_testcode.sh`，否则未支持组可能污染 basic 输出或拖长评测时间。

完成标准：

- 默认构建仍只跑 basic。
- 本地调试能明确切换到 busybox。
- basic 本地回归仍为 `102/102`。

## 3. busybox 第一阶段：进入脚本并定位第一个失败点

目标不是一次通过 busybox，而是稳定进入 busybox 脚本，运行到第一个真实失败点。

当前状态：调度链路已完成。内核会直接读取 `busybox_cmd.txt`，将不含复杂 shell 语法的命令转换为 `busybox <applet> ...` 队列，并输出官方 success/fail 格式。下一步需要使用真实静态 busybox ELF 运行并分析失败。

优先处理：

- busybox 脚本中的 `busybox echo`、`cd`、嵌套 `.sh`、相对路径。
- `argv[0]` 和 applet 参数传递，确保 busybox 能识别子命令。
- 当前目录按测试组目录隔离，避免 basic 和 busybox 串目录。
- 遇到未支持命令时不要 panic，应输出可定位日志并安全结束或跳过。

完成标准：

- QEMU 输出 busybox START marker。
- 至少能运行一个 busybox applet。
- 日志中能看到明确失败原因，例如 unsupported syscall、page fault、open 失败、execve 失败。
- QEMU 最终主动退出。

## 4. busybox 第二阶段：按日志补 ABI

不要一次补一大堆 syscall。每次根据失败日志补最小闭环，然后复跑 basic 和 busybox。

优先顺序：

| 类别 | 优先补齐内容 | 目的 |
|---|---|---|
| 路径和目录 | `getcwd`、`chdir`、`getdents64`、`newfstatat/fstatat`、`readlinkat`、`faccessat` | 让 busybox 能遍历目录、判断文件和处理相对路径 |
| fd 行为 | per-process fd table、`dup/dup2/dup3`、`pipe2`、`fcntl` 最小行为、tty `ioctl` stub | 降低多个程序、fork/exec、管道和标准流互相污染 |
| 进程行为 | 更完整的 `fork/clone`、`execve`、`wait4`、exit code 回收 | 支持 busybox 脚本中多进程和子命令执行 |
| 文件系统 | EXT4 目录读取、目录 fd、更多 pseudo path | 支持真实测试盘文件结构 |

完成标准：

- busybox 本地能跑完部分测试并产生可解析通过项。
- basic 本地仍保持 `102/102`。
- 无测试盘 smoke 仍主动退出。

## 5. busybox 有分后再提交官方

不要等 busybox 全部通过。只要满足以下条件，就可以考虑提交官方评测：

- basic 本地官方目录结构回归仍是 `102/102`。
- busybox 本地已经有明确新增通过项。
- QEMU 能主动退出。
- START/END 之间没有额外内核调试输出。
- 没有启用未验证的大测试组。

官方提交后，根据分数变化决定下一步：

- basic 回退：立即回滚或修复 fixed path、日志输出、脚本选择。
- basic 保持 102 且 busybox 增分：记录结果，再继续扩 busybox。
- 仍只有 basic：保留日志，继续从本地 busybox 失败点补 ABI。

## 6. busybox 后的测试组顺序

建议顺序：

1. `lua`
   - 重点关注 `brk`、`mmap`、`munmap`、`open/read/write/fstat`、`gettimeofday` 和路径行为。
   - 先确认 lua ELF 是否静态；如果需要动态链接器，必须另开阶段处理。

2. `libctest`
   - 覆盖面比 busybox/lua 更广，容易一次暴露大量 libc 语义问题。
   - 放在 busybox 和 lua 后更稳。

3. `ltp`、`libcbench`、`lmbench`
   - 等进程模型、fd table、文件系统语义更完整后再推进。

4. 网络类 `iperf`、`netperf`
   - 暂时后置。当前阶段投入大，且对 basic/busybox/lua 提分帮助有限。

## 队友协作注意事项

- 开始开发前先读 `AGENTS.md`、`docs/progress.md`、`docs/test-matrix.md` 和本文档。
- 需要同步远端时，由用户本人执行 `git pull --rebase gitlab main`；Codex 不自动执行。
- 改动前先确认自己负责的模块，避免多人同时改 `loader`、`task`、`fs`、`drivers/ext4` 等核心文件。
- 每次新增 syscall，都同步更新 `docs/test-matrix.md`。
- 每次改变官方评测入口、脚本解析、日志格式，都同步更新 `docs/progress.md`。
- 不提交生成产物：`target/`、`kernel-rv`、`kernel-la`、`exec.out`、`disk.img`、`disk-la.img`。
- 不要为了临时调试在官方 START/END 区间内打印大量日志；这会直接影响解析和得分。
- 不要直接复制其他队伍代码；只能参考设计和路线，并保留来源说明。

## 下一次代码任务建议

下一次建议只做一件事：

> 加测试组选择器，默认继续保 basic，本地开发模式只跑 busybox；然后用本地 busybox 日志定位第一个真实失败点。

这一步完成后，再决定补哪一批 syscall 或文件系统能力。
