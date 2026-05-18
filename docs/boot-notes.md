# Boot 与日志流程阅读笔记

## 阅读范围

本次阅读聚焦 rCore baseline 从 `make run` 到进入 `Rust user shell` 的完整路径。

相关文件：

| 文件 | 作用 |
|---|---|
| `rCore-Tutorial-v3-main/os/Makefile` | 构建内核、生成文件系统镜像、启动 QEMU |
| `rCore-Tutorial-v3-main/os/src/linker-qemu.ld` | 指定内核入口、加载地址和段布局 |
| `rCore-Tutorial-v3-main/os/src/entry.asm` | 设置启动栈并跳转到 Rust 入口 |
| `rCore-Tutorial-v3-main/os/src/main.rs` | 内核初始化主流程 |
| `rCore-Tutorial-v3-main/os/src/console.rs` | `print!` / `println!` 输出实现 |
| `rCore-Tutorial-v3-main/os/src/logging.rs` | `log` crate 的 logger 实现 |
| `rCore-Tutorial-v3-main/os/src/trap/mod.rs` | trap 初始化和 timer interrupt 开启 |
| `rCore-Tutorial-v3-main/user/src/bin/initproc.rs` | 第一个用户进程 |
| `rCore-Tutorial-v3-main/user/src/bin/user_shell.rs` | 用户态 shell |

## 构建和运行入口

在 `rCore-Tutorial-v3-main/os` 下执行：

```bash
make run
```

实际执行链路：

1. `run` 调用 `run-inner`。
2. `run-inner` 先检查 QEMU 版本，再执行 `build`。
3. `build` 依次构建内核二进制和用户程序文件系统镜像。
4. `kernel` 目标会把 `src/linker-qemu.ld` 复制成 `src/linker.ld`，再执行 `cargo build --release`。
5. `fs-img` 会进入 `../user` 构建用户程序，再进入 `../easy-fs-fuse` 生成 `fs.img`。
6. QEMU 使用 `QEMU_ARGS` 启动内核和文件系统镜像。

关键 QEMU 参数：

```text
-machine virt
-bios ../bootloader/rustsbi-qemu.bin
-serial stdio
-device loader,file=$(KERNEL_BIN),addr=0x80200000
-drive file=$(FS_IMG),if=none,format=raw,id=x0
-device virtio-blk-device,drive=x0
```

这里 `-serial stdio` 是我们能在终端看到内核输出的原因。

## 链接脚本和入口地址

`linker-qemu.ld` 中的关键配置：

```text
ENTRY(_start)
BASE_ADDRESS = 0x80200000;
```

含义：

- 内核入口符号是 `_start`。
- 内核被加载到物理地址 `0x80200000`。
- `.text.entry` 被放在 `.text` 最前面，因此 `_start` 会最先执行。
- `.bss.stack` 被放入 `.bss`，用于启动栈。
- 链接脚本导出 `sbss`、`ebss` 等符号，供 Rust 代码清空 BSS。

## 汇编启动入口

`entry.asm` 内容很短：

```asm
.section .text.entry
.globl _start
_start:
    la sp, boot_stack_top
    call rust_main
```

它做了两件事：

1. 把栈指针 `sp` 设置到 `boot_stack_top`。
2. 调用 Rust 入口 `rust_main`。

启动栈大小：

```asm
.space 4096 * 16
```

也就是 64 KiB。

## Rust 内核入口

`rust_main` 是当前 baseline 的核心初始化流程：

```rust
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    mm::init();
    UART.init();
    info!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    info!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    info!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    info!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    board::device_init();
    fs::list_apps();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
```

按功能拆分：

| 顺序 | 动作 | 目的 |
|---|---|---|
| 1 | `clear_bss()` | 清空未初始化全局变量区 |
| 2 | `logging::init()` | 初始化日志系统 |
| 3 | `mm::init()` | 初始化内存管理 |
| 4 | `UART.init()` | 初始化串口输出 |
| 5 | 初始化 GPU/keyboard/mouse | 初始化 VirtIO 设备 |
| 6 | `trap::init()` | 设置 kernel trap 入口 |
| 7 | `enable_timer_interrupt()` | 开启 supervisor timer interrupt |
| 8 | `timer::set_next_trigger()` | 设置第一次时钟中断 |
| 9 | `board::device_init()` | 初始化板级设备和中断控制器 |
| 10 | `fs::list_apps()` | 列出文件系统镜像里的用户程序 |
| 11 | `task::add_initproc()` | 创建并加入第一个用户进程 |
| 12 | `task::run_tasks()` | 进入调度循环 |

## 日志输出路径

日志初始化在 `logging.rs`：

```rust
log::set_logger(&LOGGER).unwrap();
log::set_max_level(...);
```

日志输出时：

```text
info! / warn! / error!
-> SimpleLogger::log
-> println!
-> console::print
-> UART.write
-> QEMU -serial stdio
-> 终端
```

`console.rs` 中 `Stdout` 实现了 `core::fmt::Write`，每个字符都会写到 UART：

```rust
for c in s.chars() {
    UART.write(c as u8);
}
```

因此当前 baseline 的日志能力依赖 UART 和 QEMU 串口重定向。

## Trap 初始化

`trap::init()` 调用 `set_kernel_trap_entry()`。

核心逻辑：

```rust
stvec::write(__alltraps_k_va, TrapMode::Direct);
sscratch::write(trap_from_kernel as usize);
```

含义：

- `stvec` 设置为 kernel trap 入口。
- `sscratch` 保存 kernel trap 回调函数地址。
- `enable_timer_interrupt()` 开启 supervisor timer interrupt。
- 后续 timer interrupt 会进入 trap 流程，调用 `set_next_trigger()` 和调度逻辑。

用户态 trap 返回时，`trap_return()` 会切换到用户 trap 入口：

```rust
stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
```

这说明当前 baseline 已经具备 kernel/user trap 入口切换机制。

## 用户程序和 shell 启动

用户程序来源：

```text
rCore-Tutorial-v3-main/user/src/bin/*
```

`fs-img` 目标会把这些程序打包到文件系统镜像 `fs.img`。

内核初始化时：

1. `fs::list_apps()` 列出镜像中的程序。
2. `task::add_initproc()` 触发 `INITPROC` 初始化。
3. `INITPROC` 打开并加载 `initproc`。
4. `initproc` fork 子进程。
5. 子进程执行 `exec("user_shell")`。
6. shell 打印：

```text
Rust user shell
>>
```

到这里说明 baseline 已经完成：

- 内核启动。
- 基础设备初始化。
- trap 初始化。
- 文件系统镜像加载。
- 第一个用户进程启动。
- 用户态 shell 运行。

## 和比赛改造的关系

当前 baseline 已经有比较完整的教学 OS 框架。后续不能只停留在“能启动”，要把它改造成比赛项目：

| 方向 | 当前状态 | 下一步 |
|---|---|---|
| 启动流程 | 已能启动 | 写清楚启动过程，保留最小可解释路径 |
| 日志 | 已有 `log` + UART | 增加关键模块调试日志，方便测例定位 |
| trap | 已有 kernel/user trap 切换 | 梳理 syscall、page fault、timer 的路径 |
| 用户程序 | 已能进入 shell | 运行基础程序并记录结果 |
| 测试 | 尚未接官方测试 | 建立 syscall 和测试矩阵 |

## 下一步任务

1. 在 shell 中运行 `hello_world`、`yield`、`forktest_simple`。
2. 把运行结果记录到 `docs/progress.md`。
3. 继续阅读 `trap/mod.rs`、`trap/context.rs`、`syscall/mod.rs`。
4. 建立 `docs/phoenix-gap.md`，对标 Phoenix 的模块完整度。
