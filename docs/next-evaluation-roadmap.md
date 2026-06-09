# Titanix 官方评测路线

## 当前起点

Titanix 已能由根目录 `make all` 构建为官方 `kernel-rv`，使用官方风格 QEMU
启动，从 EXT4 测试盘命中 fixed path basic 脚本，并输出 basic START/END。

当前只是“进入 basic”，还没有执行任何盘上 ELF。旧内核的线上 `basic=102`
保存在 `codex/basic-102-archive`。

## 第一阶段：执行第一个 basic ELF

1. 为 `oscomp` 增加 EXT4 普通文件读取能力。
2. 读取命中的 `basic_testcode.sh` 内容。
3. 只解析 basic 所需的简单命令、`cd` 和嵌套脚本。
4. 将第一个 ELF 文件读入内存。
5. 使用 Titanix 的 `Process::new`、`MemorySpace` 和 ELF loader 创建进程。
6. 输出第一个真实 testcase 结果，并主动关机。

完成标准：basic START/END 之间出现至少一个真实 testcase 结果，而不是当前的
入口提示。

## 第二阶段：恢复 basic 分数

- 将官方 EXT4 文件接入 Titanix VFS，避免长期维护两套路径语义。
- 补齐 argv、envp、auxv 和动态/静态 ELF 差异。
- 根据真实失败日志修复 Titanix syscall 行为。
- 先恢复 basic 有分，再追求 basic 全量。

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

这些能力在真实 basic ELF 尚未运行前不会带来有效评测反馈。

## 队友分工建议

- 成员 A：`oscomp` EXT4 读取、脚本解析和官方测试入口。
- 成员 B：阅读并记录 Titanix Process、MemorySpace、ELF loader、syscall 路径。
- 每次合并前共同执行根构建和官方风格 QEMU 回归。
