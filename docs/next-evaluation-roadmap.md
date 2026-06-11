# Titanix 官方评测路线

## 当前起点

Titanix 已能由根目录 `make all` 构建为官方 `kernel-rv`，使用官方风格 QEMU
启动，从 EXT4 测试盘读取 basic 脚本和首个 ELF `basic/brk`，再通过
`fork + execve + wait4` 执行。

本地官方 `test_runner.py` 已确认 `test_brk=3/3`。该结果说明新架构具备最小
得分闭环，但线上分数仍需新一轮评测确认。旧内核的线上 `basic=102` 保存在
`codex/basic-102-archive`。

## 已完成：执行第一个 basic ELF

- EXT4 普通文件读取。
- `basic_testcode.sh`、`cd`、嵌套 `run-all.sh` 解析。
- 首个 ELF 和 argv 暂存到 tmpfs。
- 使用 Titanix 现有进程、VFS、ELF loader 和 syscall 路径执行。
- 输出真实 testcase 结果并主动关机。

## 第一阶段：完整 basic 命令队列

1. 将 `tests="..."` 全部解析成有序命令队列。
2. 设计多个 ELF 和 argv 的 tmpfs 暂存协议。
3. runner 必须逐个 `fork/execve/wait4`，禁止并发。
4. 在第一个失败项停止扩展，先修复对应 ABI。
5. 每次修改都回归 `test_brk=3/3` 和无盘主动退出。

完成标准：至少连续执行 `brk` 和第二个 basic 测试，并由官方解析器识别。

## 第二阶段：扩大 basic 分数

- 将官方 EXT4 文件接入 Titanix VFS，避免长期维护两套路径语义。
- 补齐 envp、auxv 和动态/静态 ELF 差异。
- 根据真实失败日志修复 Titanix syscall 行为。
- 先稳定线上有分，再追求 basic 全量。

## 第三阶段：推进 BusyBox

- basic 稳定后再启用 BusyBox。
- 优先验证目录、fd、fork/exec/wait、pipe 和相对路径。
- 使用 Titanix 已有完整架构补语义，不重新堆最小 stub。

## 暂不投入

- 网络性能。
- 多核优化。
- 图形界面。
- 展示性功能。
- LoongArch。

这些能力在 basic 串行队列尚未稳定前不会带来有效评测反馈。

## 队友分工建议

- 成员 A：`oscomp` EXT4 读取、脚本解析和官方测试入口。
- 成员 B：阅读并记录 Titanix Process、MemorySpace、ELF loader、syscall 路径。
- 每次合并前共同执行根构建和官方风格 QEMU 回归。
