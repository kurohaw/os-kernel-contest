# 初赛开发进度记录

## 当前状态

| 项目 | 内容 |
|---|---|
| 阶段 | 初赛开发期 |
| 当前日期 | 2026-05-30 |
| 当前仓库 | GitHub: `kurohaw/os-kernel-contest` |
| 当前基础版本 | `rCore-Tutorial-v3-main` |
| 主参考作品 | 2024 Phoenix |
| 当前目标 | 完成最小 exit syscall，并准备进入最小单任务模型设计 |

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

## 2026-05-24

### 今日目标

整理 `kernel/` 中的 trap 处理结构，让当前 timer interrupt 逻辑更清晰，并为后续 syscall 和异常处理预留扩展位置。

### 修改内容

- 将 trap 类型判断逻辑拆分为 `decode_trap()`。
- 新增 `Trap` 枚举，用于区分已支持的 trap 类型和未知 trap。
- 将 supervisor timer interrupt 的处理逻辑拆分到 `handle_timer_interrupt()`。
- 在未知 trap 的 panic 信息中补充 `scause`、`stval` 和 `sepc`，方便后续定位异常来源。

### 当前处理流程

```text
__alltraps
-> trap_handler
-> read scause / stval / sepc
-> decode_trap
-> handle_timer_interrupt
-> timer::set_next_trigger
-> sret
```

### 验证计划

在 `kernel/` 下执行：

```bash
make build
make run
```

验收标准：

- `make build` 能成功生成 `kernel.bin`。
- QEMU 中能看到 `Hello kernel` 和 `kernel started`。
- QEMU 中仍能周期性看到 `timer tick`。

### 下一步

1. 运行 `make build` / `make run` 验证 trap 重构没有破坏 timer interrupt。
2. 验证通过后，将本次 trap 结构整理和文档记录一起提交。
3. 后续继续补充 panic 验证记录，并准备进入 syscall / 异常处理设计。

## 2026-05-26

### 今日目标

引入 `TrapContext`，让 trap handler 可以接收完整的 trap 现场，为后续 syscall 和异常处理做准备。

### 修改内容

- 在 `trap/mod.rs` 中新增 `TrapContext`，保存 32 个通用寄存器、`sstatus` 和 `sepc`。
- 修改 `trap.S`，在 trap 入口保存 `x0`、原始 `sp`、通用寄存器、`sstatus` 和 `sepc`。
- 通过 `mv a0, sp` 将当前 trap frame 地址传给 `trap_handler`。
- 修改 `trap_handler` 签名，让 Rust 侧通过 `&mut TrapContext` 访问 trap 现场。
- 未知 trap 的 panic 信息改为从 `TrapContext` 中读取 `sepc`。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make build
make run
```

QEMU 中仍能看到：

```text
Hello kernel
kernel started
timer tick
timer tick
```

### 结论

当前 trap 流程已经从“只处理 timer interrupt”升级为“保存并传递 trap 上下文”。后续可以基于 `TrapContext` 继续实现 syscall 参数读取、返回值写回和 `sepc` 调整。

### 下一步

1. 提交本次 `TrapContext` 结构整理。
2. 设计最小 syscall 分发入口。
3. 先实现一个最小测试 syscall，再扩展到用户态程序加载。

## 2026-05-30

### 今日目标

加入最小 syscall 分发模块，并验证内核侧 syscall dispatcher 的基本行为。

### 修改内容

- 新增 `kernel/src/syscall.rs`。
- 定义 `SYS_TEST` 测试 syscall。
- 实现最小 syscall dispatcher：根据 syscall id 分发到对应处理函数。
- 在 `main.rs` 中注册 `syscall` 模块。
- 在 `main.rs` 中通过直接调用 `syscall::syscall()` 验证分发逻辑。
- 在 `trap/mod.rs` 中保留 `UserEnvCall` 处理分支，为后续用户态 `ecall` 接入做准备。
- 移除将 S-mode `ecall` 当作用户 syscall 测试的做法，避免和 SBI 调用语义混淆。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make build
make run
```

QEMU 中可以看到：

```text
Hello kernel
kernel started
sys_test called, arg0=41
syscall dispatch test ret = 42
timer tick
```

### 关键结论

当前已经验证：

```text
kernel -> syscall dispatcher -> SYS_TEST -> return value
```

当前尚未验证：

```text
U-mode ecall -> trap_handler -> syscall dispatcher -> return to U-mode
```

原因是当前内核还没有用户态执行环境。S-mode 下直接执行 `ecall` 更接近 SBI 调用语义，不适合作为用户 syscall 路径测试。

### 下一步

1. 提交本次最小 syscall dispatcher。
2. 设计最小用户态执行闭环。
3. 准备用户态入口、用户栈和进入 U-mode 所需的 `TrapContext`。
4. 后续用真正的 U-mode `ecall` 验证 syscall path。

## 2026-05-30 最小用户态闭环

### 今日目标

建立最小用户态执行闭环，让内核可以从 S-mode 构造用户态上下文，进入 U-mode，并由 U-mode 通过 `ecall` 回到内核 syscall dispatcher。

### 修改内容

- 新增 `kernel/src/user.rs`。
- 增加临时用户栈 `USER_STACK`。
- 增加临时用户态入口 `user_entry()`。
- 在 `TrapContext` 中增加 `app_init_context()`，用于构造用户态初始上下文。
- 在 `trap/mod.rs` 中暴露 `restore(cx_addr)`，复用 `__restore` 完成 `sret`。
- 在 `main.rs` 中注册 `user` 模块，并从 `rust_main` 调用 `user::run_first_user()`。
- 用户态入口通过 `ecall` 触发 `UserEnvCall`，进入内核 syscall dispatcher。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make build
make run
```

QEMU 中可以看到类似输出：

```text
Hello kernel
kernel started
enter user mode
sys_test called, arg0=41
timer tick
```

### 关键结论

当前已经验证：

```text
S-mode kernel
-> 构造用户态 TrapContext
-> sret 进入 U-mode
-> U-mode ecall
-> S-mode trap_handler
-> syscall dispatcher
```

这说明当前内核已经具备最小的用户态 syscall 闭环。

当前尚未完成：

```text
用户态读取 syscall 返回值
用户态 exit
用户程序加载
用户地址空间隔离
```

### 下一步

1. 提交最小用户态闭环。
2. 增加 `SYS_EXIT`，让用户态可以主动结束。
3. 让用户态验证 syscall 返回值，而不是只停在死循环。
4. 后续再进入用户程序加载和地址空间设计。

## 2026-05-30 最小 exit syscall

### 今日目标

增加最小 `SYS_EXIT`，让用户态可以通过 syscall 主动结束执行。

### 修改内容

- 在 `syscall.rs` 中新增 `SYS_EXIT`。
- 在 syscall dispatcher 中接入 `SYS_EXIT`。
- 新增 `sys_exit(code)`，打印用户态退出码。
- 修改 `user_entry()`，在 `SYS_TEST` 后继续调用 `SYS_EXIT`。
- 保留 `j .` 作为兜底路径。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make build
make run
```

QEMU 中可以看到：

```text
Hello kernel
kernel started
enter user mode
sys_test called, arg0=41
user exited with code 0
```

### 结论

当前用户态已经可以通过 syscall 通知内核结束执行。后续引入任务系统后，需要将 `sys_exit()` 从死循环改为任务退出、资源回收和调度下一个任务。

### 下一步

1. 提交最小 exit syscall。
2. 引入最小 `TaskControlBlock` 和任务状态。
3. 将当前直接运行用户态的逻辑整理为 `create_init_task()` / `run_task()`。
4. 后续让 `sys_exit()` 修改任务状态，而不是直接死循环。

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
| 8 | 整理 trap 处理结构 | trap 判断、timer 处理和 `TrapContext` 传递逻辑拆分清楚 | 已完成 |
| 9 | 设计最小 syscall 入口 | 能完成 syscall id 分发和返回值验证 | 已完成 |
| 10 | 设计最小用户态闭环 | 能从 U-mode 执行 `ecall` 进入 syscall dispatcher | 已完成 |
| 11 | 增加最小 exit syscall | 用户态可以主动结束并打印退出码 | 已完成 |
| 12 | 设计最小单任务模型 | 用户态执行实体由任务结构管理 | 未开始 |

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
| 10 | 提交 trap 处理结构整理 | 已验证，准备提交 |
| 11 | 提交最小 syscall 分发 | 已验证，准备提交 |
| 12 | 提交最小用户态闭环 | 已验证，准备提交 |
| 13 | 提交最小 exit syscall | 已验证，准备提交 |
| 14 | 设计最小单任务模型 | 未开始 |
