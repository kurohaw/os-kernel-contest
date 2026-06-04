# OS Kernel Contest

操作系统内核比赛开发仓库。

当前开发路线：

- 以 `rCore-Tutorial-v3-main` 作为学习和参考 baseline。
- 在 `kernel/` 中实现自建 Rust/RISC-V 内核。
- 在 `user/` 中维护独立用户程序，编译为二进制后由内核嵌入、加载和运行。

## 当前进度

截至 2026-06-04，`kernel/` 已完成：

- QEMU + RustSBI 启动。
- `_start -> rust_main` 启动链路。
- `.bss` 清零。
- SBI 串口输出。
- `print!` / `println!`。
- panic handler。
- trap 入口和 timer interrupt。
- `TrapContext` 保存和恢复。
- syscall dispatcher。
- `SYS_TEST`、`SYS_EXIT`、`SYS_YIELD`。
- 两个用户任务 round-robin 轮转。
- 物理页帧分配器。
- Sv39 地址类型、页表项和页表映射。
- 内核地址空间 `MemorySet`。
- Sv39 分页开启。
- 用户地址空间自检。
- 每个任务绑定独立用户地址空间并切换 `satp`。
- 独立 `user/` 用户程序构建。
- 用户程序二进制嵌入内核。
- 用户程序加载到用户地址空间 `0x10000` 并运行。

当前用户程序仍是最小测试程序，下一阶段目标是补齐比赛常用基础 syscall，优先实现 `write`、`getpid`、`read` 等接口。

QEMU 中可以看到类似：

```text
loader: app0 binary size=... bytes, entry=0x10000
user app loaded: app_id=0, va=0x10000, bytes=...
run task 0, switch_satp=...
sys_test called, arg0=100
task 0 yield
run task 1, switch_satp=...
sys_test called, arg0=200
task 1 exited with code 1
all tasks exited
```

## 运行方式

在 WSL/bash 中执行：

```bash
cd /mnt/d/os-kernel-contest/kernel
make run
```

`kernel/Makefile` 会先构建 `user/` 中的用户程序，再构建并运行内核。

也可以只构建用户程序：

```bash
cd /mnt/d/os-kernel-contest/user
make build
```

退出 QEMU：

```text
Ctrl + A
X
```

## 文档

- 开发进度记录：`docs/progress.md`
- 启动流程阅读记录：`docs/boot-notes.md`
- 本地开发路线：`docs/local-kernel-roadmap.md`
- 参考来源说明：`docs/references.md`

# 参考来源与增量贡献说明

## 基础参考
- rCore-Tutorial-v3：作为 Rust/RISC-V 内核学习基础。
- 2024 Phoenix：参考其比赛工程路线、模块划分和开发顺序。
- 2025 Starry Mix / NoAxiom：后期参考 syscall 兼容和测试处理。

## 原则
不直接复制 Phoenix 代码。
如果复用 rCore 代码，保留原许可证和来源说明。
本项目的增量贡献会在提交记录和文档中持续记录。
