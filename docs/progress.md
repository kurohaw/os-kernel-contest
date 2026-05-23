# 初赛开发进度记录

## 当前状态

| 项目 | 内容 |
|---|---|
| 阶段 | 初赛开发期 |
| 当前日期 | 2026-05-23 |
| 当前仓库 | GitHub: `kurohaw/os-kernel-contest` |
| 当前基础版本 | `rCore-Tutorial-v3-main` |
| 主参考作品 | 2024 Phoenix |
| 当前目标 | 跑通自建 `kernel/` 最小内核，并完成 trap/timer interrupt 验收 |

## 2026-05-18

### 今日目标

运行 rCore baseline，确认基础内核可以在 QEMU 中启动。

### 运行命令

在 `rCore-Tutorial-v3-main/os` 下执行：

```bash
make run
```

### 运行结果

成功。

观察到：

- RustSBI-QEMU 启动成功。
- 内核进入初始化流程。
- GPU、keyboard、mouse 初始化成功。
- trap 初始化成功。
- 检测到 block device。
- 成功进入 Rust user shell。

### 下一步

1. 在 Rust user shell 中运行基础用户程序。
2. 记录 `hello_world`、`yield`、`forktest_simple` 的结果。
3. 阅读 boot 和 logging 相关源码。
4. 写 `docs/boot-notes.md`。

## 2026-05-23

### 今日目标

在 `kernel/` 目录中完成自建最小内核的启动闭环，并接入基础 trap/timer interrupt。

### 运行命令

在 `kernel/` 下执行：

```bash
make run
```

### 运行结果

成功。

观察到：

- RustSBI-QEMU 启动成功。
- 自建内核从 `_start` 进入 `rust_main`。
- 串口输出成功，QEMU 中能看到 `Hello kernel` 和 `kernel started`。
- trap 入口初始化成功。
- supervisor timer interrupt 能触发。
- QEMU 中能周期性看到 `timer tick`。

### 已完成内容

- 建立 `kernel/` 最小内核工程结构。
- 完成 RISC-V 裸机编译配置。
- 完成 linker 脚本，入口地址为 `0x80200000`。
- 完成汇编启动入口 `_start`，设置启动栈并跳转到 `rust_main`。
- 完成 `.bss` 清零。
- 完成 SBI 串口输出和 `print!` / `println!`。
- 完成 panic handler。
- 完成 trap 汇编入口 `__alltraps`。
- 完成 `stvec` 初始化。
- 完成 timer interrupt 开启和下一次 timer 设置。

### 关键结论

当前自建最小内核已经具备：

```text
bootloader -> _start -> rust_main -> console -> trap -> timer interrupt
```

阶段 1 的核心验收项已经完成。

### 下一步

1. 整理本次代码改动并提交 commit。
2. 补充 panic 验证记录。
3. 继续完善 trap 处理结构，为后续 syscall / 异常处理做准备。

## 下一组任务

| 顺序 | 任务 | 完成标准 | 状态 |
|---|---|---|---|
| 1 | 清理仓库 | `hello-rust/` 不再出现在 `git status` | 进行中 |
| 2 | 运行基础用户程序 | 至少记录 3 个用户程序运行结果 | 未开始 |
| 3 | 阅读启动流程 | 写出 `entry.asm -> rust_main` 的流程说明 | 已完成 |
| 4 | 阅读日志系统 | 说明 `console.rs`、`logging.rs` 的作用 | 已完成 |
| 5 | 建立 Phoenix 差距表 | 写 `docs/phoenix-gap.md` | 未开始 |
| 6 | 跑通自建最小内核 | QEMU 中看到 `Hello kernel` / `kernel started` | 已完成 |
| 7 | 接入 timer interrupt | QEMU 中周期性看到 `timer tick` | 已完成 |

## 用户程序测试记录

| 日期 | 程序 | 命令 | 结果 | 备注 |
|---|---|---|---|---|
| 2026-05-18 | `hello_world` | 待运行 | 未记录 | 进入 shell 后运行 |
| 2026-05-18 | `yield` | 待运行 | 未记录 | 进入 shell 后运行 |
| 2026-05-18 | `forktest_simple` | 待运行 | 未记录 | 进入 shell 后运行 |

## 提交计划

| 次数 | 提交内容 | 状态 |
|---|---|---|
| 1 | 初始化项目 | 已完成 |
| 2 | 初始化参赛开发文档 | 已完成 |
| 3 | 引入 rCore baseline | 已完成 |
| 4 | 记录 rCore baseline 运行结果 | 已完成 |
| 5 | 清理仓库并完善参考说明 | 进行中 |
| 6 | 阅读并记录 boot/logging 流程 | 已完成 |
| 7 | 增加 Phoenix 差距分析 | 未开始 |
| 8 | 接入测试记录矩阵 | 未开始 |
| 9 | 提交自建最小内核启动与 timer interrupt | 准备提交 |
