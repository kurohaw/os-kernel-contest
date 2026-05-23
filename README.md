# OS Kernel Contest

操作系统比赛开发仓库。

当前开发路线：

- 以 `rCore-Tutorial-v3-main` 作为学习和参考 baseline。
- 在 `kernel/` 目录中逐步实现自建最小内核。
- 当前阶段目标是跑通最小内核启动、串口输出、trap 初始化和 timer interrupt。

## 当前进度

截至 2026-05-23，`kernel/` 已完成：

- RISC-V 裸机工程配置
- linker 脚本
- 汇编启动入口 `_start`
- Rust 入口 `rust_main`
- `.bss` 清零
- SBI 串口输出
- `print!` / `println!`
- panic handler
- trap 入口初始化
- supervisor timer interrupt

QEMU 中可以看到：

```text
Hello kernel
kernel started
timer tick

# 参考来源与增量贡献说明

## 基础参考
- rCore-Tutorial-v3：作为 Rust/RISC-V 内核学习基础。
- 2024 Phoenix：参考其比赛工程路线、模块划分和开发顺序。
- 2025 Starry Mix / NoAxiom：后期参考 syscall 兼容和测试处理。

## 原则
不直接复制 Phoenix 代码。
如果复用 rCore 代码，保留原许可证和来源说明。
本项目的增量贡献会在提交记录和文档中持续记录。