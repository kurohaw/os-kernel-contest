# 初赛开发进度记录

## 当前状态

| 项目 | 内容 |
|---|---|
| 阶段 | 初赛开发期 |
| 当前日期 | 2026-06-01 |
| 当前仓库 | GitHub: `kurohaw/os-kernel-contest` |
| 当前基础版本 | `rCore-Tutorial-v3-main` |
| 主参考作品 | 2024 Phoenix |
| 当前目标 | 完善页表映射能力，为内核地址空间恒等映射做准备 |

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

## 2026-05-30 最小单任务模型

### 今日目标

引入最小任务结构，让用户态执行实体不再由 `main.rs` 直接启动，而是通过任务模块统一管理。

### 修改内容

- 新增 `kernel/src/task/mod.rs`。
- 新增 `TaskStatus`，包含 `Ready`、`Running`、`Exited`。
- 新增 `TaskControlBlock`，保存任务状态和 `trap_cx_addr`。
- 新增 `task::init()`，创建初始用户任务。
- 新增 `task::run_first_task()`，将任务状态切换为 `Running` 并恢复用户态上下文。
- 新增 `task::exit_current(code)`，将当前任务状态切换为 `Exited`。
- 将 `user.rs` 中的用户上下文准备逻辑整理为 `init_user_context()`。
- 将 `main.rs` 中的直接用户态启动改为 `task::init()` / `task::run_first_task()`。
- 将 `SYS_EXIT` 接入 `task::exit_current()`。

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
run first task
sys_test called, arg0=41
user exited with code 0
all tasks exited
```

### 结论

当前用户态执行已经具备最小任务承载结构。虽然目前仍然只有一个任务，但后续可以在 `TaskControlBlock` 基础上继续扩展任务队列、上下文切换和调度器。

### 下一步

1. 提交最小单任务模型。
2. 设计内核任务上下文 `TaskContext`。
3. 引入任务队列，为 `yield` 和多任务轮转做准备。
4. 后续将 `exit_current()` 从死循环改为切换到下一个可运行任务。

## 2026-05-30 TaskContext 与 SYS_YIELD

### 今日目标

引入内核任务上下文 `TaskContext` 和 `SYS_YIELD`，为后续多任务轮转和上下文切换做准备。

### 修改内容

- 新增 `kernel/src/task/context.rs`。
- 新增 `TaskContext`，保存 `ra`、`sp` 和 `s0-s11`。
- 新增 `kernel/src/task/switch.S`。
- 新增 `__switch` 汇编入口，用于保存当前任务上下文并恢复下一个任务上下文。
- 在 `task/mod.rs` 中引入 `TaskContext` 和 `switch.S`。
- 在 `TaskControlBlock` 中新增 `task_cx` 字段。
- 初始化任务时使用 `TaskContext::zero_init()`。
- 新增 `task::suspend_current_and_run_next()`。
- 在 `syscall.rs` 中新增 `SYS_YIELD`。
- 用户态入口在 `SYS_TEST` 后调用 `SYS_YIELD`，再调用 `SYS_EXIT`。

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
run first task
sys_test called, arg0=41
user yield
user exited with code 0
all tasks exited
```

### 关键结论

当前已经完成任务上下文和 yield syscall 的最小接入：

```text
U-mode SYS_YIELD
-> syscall dispatcher
-> task::suspend_current_and_run_next()
```

当前仍然只有一个任务，因此 `SYS_YIELD` 暂时不会真正切换任务，只完成状态路径和接口验证。`__switch` 已经准备好，后续多任务轮转阶段再正式接入。

### 下一步

1. 提交 `TaskContext` 和 `SYS_YIELD`。
2. 将单个 `INIT_TASK` 扩展为任务数组或任务队列。
3. 创建第二个用户任务。
4. 接入真正的 round-robin 调度。

## 2026-05-31 多任务轮转

### 今日目标

将单任务模型扩展为最小多任务 round-robin，让至少两个用户任务可以通过 `SYS_YIELD` 主动让出 CPU，并在退出后调度下一个可运行任务。

### 修改内容

- 在 `user.rs` 中增加 `APP_NUM`，表示当前内嵌用户任务数量。
- 将单个用户栈扩展为 `USER_STACK_0` 和 `USER_STACK_1`。
- 将 `init_user_context()` 改为接收 `app_id`，为不同任务构造不同的 `TrapContext`。
- 新增 `user_entry_0()` 和 `user_entry_1()`，分别使用不同的 syscall 参数和 exit code，便于观察任务切换顺序。
- 在 `task/mod.rs` 中将单个 `INIT_TASK` 扩展为 `TASKS` 数组。
- 新增 `CURRENT` 记录当前运行任务编号。
- 新增 `find_next_ready()`，从当前任务后面开始查找下一个 `Ready` 任务。
- 修改 `suspend_current_and_run_next()`，让当前任务变回 `Ready` 后调度下一个任务。
- 修改 `exit_current()`，让当前任务变为 `Exited` 后继续调度下一个任务；所有任务结束后打印 `all tasks exited`。
- 将 `SYS_YIELD` 的真正调度放到 `trap/mod.rs` 的 `handle_environment_call()` 中，在 syscall 返回值写回 `a0` 之后执行。

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
run task 0
sys_test called, arg0=100
task 0 yield
run task 1
sys_test called, arg0=200
task 1 yield
run task 0
task 0 exited with code 0
run task 1
task 1 exited with code 1
all tasks exited
```

### 关键结论

当前已经完成最小多任务调度闭环：

```text
task 0
-> SYS_YIELD
-> trap_handler
-> scheduler
-> task 1
-> SYS_YIELD
-> scheduler
-> task 0 / task 1 exit
```

当前调度仍然是教学型最小实现：任务上下文结构已经存在，但真实调度路径仍主要依赖恢复不同任务的 `TrapContext`。后续进入内存管理和用户程序加载前，需要继续保持调度逻辑简单，避免过早引入复杂进程模型。

### 下一步

1. 提交本次多任务轮转修改。
2. 进入内存管理基础阶段。
3. 先建立物理页帧分配器，再进入 Sv39 页表。
4. 验证目标不是完整虚拟内存，而是先保证已有两个用户任务在引入内存模块后仍能正常运行。

## 2026-05-31 物理页帧分配器

### 今日目标

建立最小物理页帧分配器，为后续 Sv39 页表和用户地址空间映射做准备。

### 修改内容

- 新增 `kernel/src/mm/mod.rs`，作为内存管理模块入口。
- 新增 `kernel/src/mm/frame_allocator.rs`。
- 定义 `PAGE_SIZE = 4096`。
- 定义 `MEMORY_END = 0x8800_0000`，匹配当前 QEMU virt 平台的物理内存结束地址。
- 通过 linker symbol `ekernel` 确定内核镜像结束位置。
- 从 `ekernel` 向上按 4KiB 对齐后开始分配物理页帧。
- 新增 `FrameTracker`，保存分配到的物理页号，并提供 `start_pa()` 和 `zero()`。
- 新增 `StackFrameAllocator`，当前只支持顺序分配，不支持回收。
- 在 `main.rs` 中注册 `mm` 模块，并在任务初始化前调用 `mm::init()`。
- 修复 `trap.S` 中 `__alltraps` 和 `__restore` 的 4 字节对齐问题，避免 `stvec` 写入非对齐 trap 入口后导致用户态 `ecall` 无法进入 `trap_handler`。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make clean
make run
```

QEMU 中可以看到类似输出：

```text
Hello kernel
kernel started
frame allocator: start=0x80219000, end=0x88000000, frames=32231
run task 0
sys_test called, arg0=100
task 0 yield
run task 1
sys_test called, arg0=200
task 1 yield
run task 0
task 0 exited with code 0
run task 1
task 1 exited with code 1
all tasks exited
```

### 关键结论

当前已经完成最小物理页帧分配闭环：

```text
linker ekernel
-> frame allocator init
-> allocatable physical frames
-> existing user tasks still run correctly
```

这一步还没有启用分页，也没有建立页表。它只解决后续页表页和用户内存页从哪里来的问题。

### 下一步

1. 提交多任务轮转和物理页帧分配器。
2. 新增地址类型封装：`PhysAddr`、`VirtAddr`、`PhysPageNum`、`VirtPageNum`。
3. 新增 Sv39 页表项 `PageTableEntry`。
4. 再实现最小 `PageTable`，先做到可以创建空页表和映射单页。

## 2026-06-01 地址类型和页表项

### 今日目标

新增 Sv39 地址类型封装和页表项结构，为后续实现 `PageTable`、`map()` 和地址空间隔离做准备。

### 修改内容

- 新增 `kernel/src/mm/address.rs`。
- 定义 `PhysAddr`、`VirtAddr`、`PhysPageNum`、`VirtPageNum`。
- 为物理地址和虚拟地址提供 `floor()`、`ceil()`、`page_offset()` 和 `is_aligned()`。
- 为物理页号提供 `start_pa()`。
- 为虚拟页号提供 `start_va()` 和三级页表索引拆分 `indexes()`。
- 新增 `kernel/src/mm/page_table.rs`。
- 定义 `PTEFlags`，表示 Sv39 页表项低 10 位 flags。
- 定义 `PageTableEntry`，支持从 `PhysPageNum + PTEFlags` 构造页表项。
- 支持从页表项中解析 `ppn()`、`flags()`，并判断 `V/R/W/X` 权限。
- 在 `mm::init()` 中加入临时 `page_table::self_check()`，验证页表项编码和解码。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make build
make run
```

QEMU 中可以看到：

```text
page table entry test: ppn=0x80200, flags=0x7
```

并且原有多任务轮转仍然正常：

```text
run task 0
sys_test called, arg0=100
task 0 yield
run task 1
sys_test called, arg0=200
task 1 yield
task 0 exited with code 0
task 1 exited with code 1
all tasks exited
```

### 关键结论

当前已经完成页表实现前的基础类型准备：

```text
usize address
-> typed address/page number
-> Sv39 PTE encode/decode
```

这一步仍未启用分页，也未修改 `satp`。系统仍运行在直接物理地址访问模式下，因此可以安全验证，不影响现有任务调度。

### 下一步

1. 提交地址类型和页表项。
2. 实现最小 `PageTable`。
3. 支持创建根页表。
4. 支持单页 `map(vpn, ppn, flags)`。
5. 先用自检验证映射，不急着开启分页。

## 2026-06-01 最小 PageTable

### 今日目标

实现最小 `PageTable`，支持创建根页表、自动分配中间页表页、映射单个虚拟页，并通过 `translate()` 查回映射结果。

### 修改内容

- 在 `page_table.rs` 中新增 `PageTable`。
- `PageTable::new()` 会通过 `alloc_frame()` 分配一个根页表页。
- `map(vpn, ppn, flags)` 支持建立单页映射。
- `find_pte_create()` 会按 Sv39 三级页表索引查找页表项；中间页表不存在时自动分配新的页表页。
- `translate(vpn)` 支持从根页表查询一个虚拟页对应的页表项。
- `self_check()` 增加页表映射测试，验证 `vpn -> ppn` 能正确写入并查回。

### 验证结果

成功。

在 `kernel/` 下执行：

```bash
make build
make run
```

QEMU 中可以看到：

```text
page table map test: vpn=0x100, ppn=0x80200, flags=0x7
```

并且原有多任务轮转仍然正常：

```text
run task 0
sys_test called, arg0=100
task 0 yield
run task 1
sys_test called, arg0=200
task 1 yield
task 0 exited with code 0
task 1 exited with code 1
all tasks exited
```

### 关键结论

当前已经完成最小页表数据结构闭环：

```text
alloc root page table
-> allocate intermediate page table frames
-> map single VPN to PPN
-> translate VPN back to PTE
```

这一步仍然没有写 `satp`，没有开启分页，因此不会影响当前物理地址直接运行模式。

### 下一步

1. 支持页表区间映射。
2. 将一段虚拟地址区间映射到连续物理页。
3. 用自检验证多个连续页都能被 `translate()` 查回。
4. 完成后再进入内核地址空间恒等映射。

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
| 12 | 设计最小单任务模型 | 用户态执行实体由任务结构管理 | 已完成 |
| 13 | 设计任务上下文和 yield | 为多任务轮转准备 `TaskContext` 并接入 `SYS_YIELD` | 已完成 |
| 14 | 设计多任务轮转 | 至少两个用户任务可以通过 yield 轮转 | 已完成 |
| 15 | 设计内存管理基础 | 建立物理页帧分配器，为 Sv39 页表做准备 | 已完成 |
| 16 | 设计地址类型和页表项 | 支持 Sv39 地址转换基础类型和 `PageTableEntry` | 已完成 |
| 17 | 设计最小页表 | 支持创建根页表和单页映射自检 | 已完成 |
| 18 | 设计页表区间映射 | 支持连续虚拟页映射到连续物理页并通过自检 | 未开始 |

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
| 14 | 提交最小单任务模型 | 已验证，准备提交 |
| 15 | 提交任务上下文和 yield | 已验证，准备提交 |
| 16 | 设计多任务轮转 | 已验证，准备提交 |
| 17 | 设计内存管理基础 | 已验证，准备提交 |
| 18 | 设计地址类型和页表项 | 已验证，准备提交 |
| 19 | 设计最小页表 | 已验证，准备提交 |
| 20 | 设计页表区间映射 | 未开始 |
