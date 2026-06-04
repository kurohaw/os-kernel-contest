# 初赛开发进度记录

## 当前状态

| 项目 | 内容 |
|---|---|
| 阶段 | 初赛开发期 |
| 当前日期 | 2026-06-04 |
| 当前仓库 | GitHub: `kurohaw/os-kernel-contest`；GitLab: `gitlab.eduxiji.net/T2026102569910192/oskernel2025-sudo_win_the_cscc` |
| 当前基础版本 | `rCore-Tutorial-v3-main` |
| 主参考作品 | 2024 Phoenix |
| 当前目标 | 将独立用户程序二进制加载到用户地址空间运行 |
| 当前完成度 | 已完成最小启动、trap、syscall、两任务轮转、物理页帧分配、Sv39 页表基础、区间映射、内核地址空间结构、临时用户段权限映射、Sv39 内核分页开启、用户地址空间自检、任务绑定用户地址空间、按任务切换页表、用户程序 loader 边界、独立用户程序构建和用户程序二进制嵌入自检 |

## 2026-05-18 rCore baseline

### 今日目标

运行 rCore baseline，确认基础内核可以在 QEMU 中启动。

### 修改内容

- 运行 rCore 原始工程。
- 观察 RustSBI、内核初始化、设备初始化和用户 shell。

### 结论

rCore baseline 可以正常启动，后续自建内核以它作为学习参考。

## 2026-05-23 最小内核启动

### 今日目标

在 `kernel/` 中完成自建最小内核启动闭环，并接入基础 timer interrupt。

### 修改内容

- 建立 `kernel/` 最小工程结构。
- 完成 `_start -> rust_main` 启动链路。
- 完成 linker 脚本、`.bss` 清零、SBI 串口输出、panic handler。
- 接入 trap 入口和 supervisor timer interrupt。

### 结论

自建内核已具备 `bootloader -> _start -> rust_main -> console -> trap -> timer` 的最小闭环。

## 2026-05-24 trap 结构整理

### 今日目标

整理 trap 处理结构，为 syscall 和异常处理预留扩展位置。

### 修改内容

- 新增 `decode_trap()`。
- 新增 `Trap` 枚举。
- 将 timer interrupt 处理拆分到 `handle_timer_interrupt()`。
- 在未知 trap 的 panic 信息中补充 `scause`、`stval`、`sepc`。

### 结论

trap 判断和处理路径更清晰，后续可以继续接入 syscall、page fault 等异常处理。

## 2026-05-26 TrapContext

### 今日目标

引入 `TrapContext`，让 trap handler 能访问完整 trap 现场。

### 修改内容

- 新增 `TrapContext`，保存通用寄存器、`sstatus`、`sepc`。
- 修改 `trap.S` 保存和恢复 trap 上下文。
- 通过 `a0` 将 trap frame 地址传给 `trap_handler`。
- 修改 panic 信息从 `TrapContext` 中读取 `sepc`。

### 结论

trap 处理从简单中断处理升级为完整上下文保存，为 syscall 返回值写回和用户态恢复打下基础。

## 2026-05-30 syscall 与用户态闭环

### 今日目标

建立最小 syscall 分发和 U-mode `ecall` 闭环。

### 修改内容

- 新增 `kernel/src/syscall.rs`。
- 实现 `SYS_TEST`。
- 在 `trap/mod.rs` 中处理 `UserEnvCall`。
- 新增 `kernel/src/user.rs`。
- 构造用户态 `TrapContext`，通过 `sret` 进入 U-mode。
- 用户态通过 `ecall` 回到内核 syscall dispatcher。

### 结论

内核已具备最小用户态 syscall 闭环：`S-mode -> U-mode -> ecall -> trap_handler -> syscall`。

## 2026-05-30 exit syscall 与单任务模型

### 今日目标

让用户态可以主动退出，并把用户态执行实体纳入任务结构管理。

### 修改内容

- 新增 `SYS_EXIT`。
- 新增 `TaskStatus` 和 `TaskControlBlock`。
- 新增 `task::init()`、`task::run_first_task()`、`task::exit_current()`。
- 将用户态启动从 `main.rs` 移入 `task` 模块。

### 结论

用户态执行已经由任务结构承载，后续可以在此基础上扩展调度。

## 2026-05-30 TaskContext 与 SYS_YIELD

### 今日目标

引入任务上下文和 yield syscall，为多任务轮转做准备。

### 修改内容

- 新增 `kernel/src/task/context.rs`。
- 新增 `TaskContext`。
- 新增 `kernel/src/task/switch.S`。
- 新增 `SYS_YIELD`。
- 在 `TaskControlBlock` 中加入 `task_cx` 字段。

### 结论

任务切换所需的数据结构和接口已准备好，当前阶段先用最小路径验证 yield。

## 2026-05-31 多任务轮转

### 今日目标

将单任务模型扩展为两个用户任务的 round-robin 轮转。

### 修改内容

- 增加 `APP_NUM`。
- 增加两个用户栈和两个用户入口。
- 将单个任务扩展为 `TASKS` 数组。
- 新增 `CURRENT`。
- 新增 `find_next_ready()`。
- 修改 `suspend_current_and_run_next()` 和 `exit_current()`，支持调度下一个可运行任务。

### 结论

两个用户任务可以通过 `SYS_YIELD` 轮转，并在退出后继续调度剩余任务。

## 2026-05-31 物理页帧分配器

### 今日目标

建立最小物理页帧分配器，为页表和用户地址空间做准备。

### 修改内容

- 新增 `kernel/src/mm/mod.rs`。
- 新增 `kernel/src/mm/frame_allocator.rs`。
- 从 linker symbol `ekernel` 后开始管理可分配物理页帧。
- 新增 `FrameTracker`。
- 在 `main.rs` 中初始化 `mm`。
- 修复 `trap.S` 中 `__alltraps` 和 `__restore` 的 4 字节对齐问题。

### 结论

内核已经可以按 4KiB 页帧分配物理内存，现阶段暂不支持回收。

## 2026-06-01 地址类型和页表项

### 今日目标

新增 Sv39 地址类型封装和页表项结构。

### 修改内容

- 新增 `kernel/src/mm/address.rs`。
- 定义 `PhysAddr`、`VirtAddr`、`PhysPageNum`、`VirtPageNum`。
- 提供地址对齐、页号转换和三级页表索引拆分。
- 新增 `kernel/src/mm/page_table.rs`。
- 定义 `PTEFlags` 和 `PageTableEntry`。

### 结论

页表实现所需的地址类型和 Sv39 PTE 编码/解码基础已完成。

## 2026-06-01 最小 PageTable

### 今日目标

实现最小 `PageTable`，支持创建根页表、单页映射和查询。

### 修改内容

- 新增 `PageTable`。
- `PageTable::new()` 分配根页表页。
- `map()` 支持单页映射。
- `find_pte_create()` 自动分配中间页表页。
- `translate()` 支持查询虚拟页映射。

### 结论

最小页表数据结构闭环已完成：可以创建页表、写入单页映射并查回页表项。

## 2026-06-02 页表区间映射

### 今日目标

支持连续虚拟页到连续物理页的区间映射。

### 修改内容

- 在 `PageTable` 中新增 `map_range()`。
- 对 `start_va`、`end_va`、`start_pa` 做页对齐检查。
- 将虚拟地址区间转换为连续 `VirtPageNum`。
- 将物理地址转换为连续 `PhysPageNum`。
- 循环调用 `map()` 建立连续页映射。

### 结论

页表已支持连续页映射，可以开始抽象内核地址空间。

## 2026-06-03 内核地址空间结构

### 今日目标

建立 `MemorySet::new_kernel()`，为开启分页准备内核地址空间。

### 修改内容

- 新增内核地址空间抽象 `MemorySet`。
- 对 `.text`、`.rodata`、`.data`、`.bss` 做恒等映射。
- 将临时用户入口放入 `.user.text`，并以 `R | X | U` 权限映射。
- 将临时用户栈放入 `.user.stack`，并以 `R | W | U` 权限映射。
- 新增 `satp_token()` 和 `activate()` 接口，为后续写入 `satp` 做准备。

### 结论

内核地址空间结构已具备基本形态，下一步可以实际开启 Sv39 分页并验证 trap、syscall、任务轮转是否仍然正确。

## 2026-06-03 开启内核分页

### 今日目标

写入 `satp` 开启 Sv39 分页，并确认当前内核功能在分页下仍能运行。

### 修改内容

- 在 `mm::init()` 中创建并保存全局 `KERNEL_SPACE`。
- 调用 `MemorySet::activate()` 写入 `satp` 并刷新 TLB。
- 新增 `trap::enable_user_memory_access()` 打开 `sstatus.SUM`。
- 在任务初始化前允许 S-mode 临时访问用户页。

### 结论

分页开启后，内核仍能启动、打印、进入用户态、处理 syscall，并完成两个临时用户任务的轮转和退出。

## 2026-06-04 用户地址空间结构

### 今日目标

为每个临时用户任务创建独立 `MemorySet`，并验证用户代码和用户栈权限。

### 修改内容

- 新增 `PageTableEntry::user()`，用于检查 PTE 的 `U` 权限。
- 新增 `user::user_stack_range(app_id)`，返回指定任务的用户栈范围。
- 新增 `MemorySet::new_user(app_id)`，构造单个任务的用户地址空间。
- 在用户地址空间中映射内核段、`.user.text` 和当前任务的 `.user.stack`。
- 新增 `MemorySet::self_check_user(app_id)`，验证内核代码不可被用户访问、用户代码可执行、用户栈可读写。
- 在 `mm::init()` 中对所有临时用户任务执行用户地址空间自检。

### 结论

用户地址空间结构自检通过；当前任务仍运行在内核页表下，下一步需要把 `MemorySet` 保存到 `TaskControlBlock` 并在任务运行时切换到对应页表。

## 2026-06-04 任务绑定用户地址空间

### 今日目标

让每个任务保存自己的用户地址空间和 `satp` token。

### 修改内容

- 将 `MEMORY_END` 暴露给内存映射模块。
- 在内核地址空间和用户地址空间中映射 `ekernel..MEMORY_END`，保证分页开启后页帧分配区可访问。
- 在 `TaskControlBlock` 中新增 `memory_set` 和 `satp_token`。
- 初始化任务时创建 `MemorySet::new_user(app_id)`，并保存对应 `satp_token`。
- 运行任务时打印任务对应的用户页表 token。

### 结论

任务已经绑定独立用户地址空间；当前仍未实际切换到任务页表，下一步进入任务前写入对应 `satp`。

## 2026-06-04 按任务切换页表

### 今日目标

进入用户任务前切换到该任务自己的用户页表，并验证 syscall、trap 和任务轮转仍然正常。

### 修改内容

- 在 `mm` 模块新增 `activate_satp(token)`，用于切换到指定页表。
- 在 `MemorySet::new_user()` 中临时映射完整 `.user.stack`，避免当前 trap 栈在切换页表时失效。
- 在 `run_task()` 中进入用户态前写入当前任务的 `satp_token`。
- 切换页表前提前取出 `trap_cx_addr`，避免切换后再次依赖任务数组字段访问。

### 结论

每个临时用户任务已经可以在自己的用户页表下运行，并且 `SYS_TEST`、`SYS_YIELD`、`SYS_EXIT` 和任务轮转仍然正常。

## 2026-06-04 用户程序加载准备

### 今日目标

从 `user.rs` 中拆出用户程序入口，建立 `loader` 模块边界。

### 修改内容

- 新增 `kernel/src/loader.rs`。
- 将临时用户入口 `user_entry_0` 和 `user_entry_1` 移入 `loader`。
- 新增 `loader::APP_NUM` 和 `loader::app_entry(app_id)`。
- `user.rs` 改为只负责用户栈和 `TrapContext` 构造。
- `main.rs` 接入 `loader` 模块。

### 结论

用户程序入口已经由 `loader` 模块管理；当前行为不变，下一步可以将内核内置入口替换为独立用户程序编译产物。

## 2026-06-04 独立用户程序构建与嵌入自检

### 今日目标

建立独立 `user/` 用户程序目录，并确认内核能嵌入用户程序二进制。

### 修改内容

- 新增 `user/` 裸机 Rust 工程。
- 新增 `app0` 和 `app1` 两个用户程序。
- 新增用户态 syscall wrapper。
- 新增用户程序 linker script 和 Makefile。
- 修改 `kernel/Makefile`，在构建内核前先构建用户程序。
- 在 `loader` 中通过 `include_bytes!` 嵌入 `app0.bin` 和 `app1.bin`。
- 启动时打印用户程序二进制大小和入口地址。

### 结论

独立用户程序已经可以单独编译成二进制，并被内核嵌入和识别；下一步需要把二进制内容复制到用户地址空间，并让任务从用户程序入口运行。

## 下一组任务

| 顺序 | 任务 | 完成标准 | 状态 |
|---|---|---|---|
| 1 | 设计内核地址空间结构 | `MemorySet::new_kernel()` 能完成内核段恒等映射自检 | 已完成 |
| 2 | 开启内核分页 | 写入 `satp` 后内核仍能启动、打印、处理中断 | 已完成 |
| 3 | 设计用户地址空间 | 用户入口和用户栈权限映射能通过自检 | 已完成 |
| 4 | 任务绑定用户地址空间 | 每个任务保存自己的 `MemorySet` 并能取得 `satp` token | 已完成 |
| 5 | 按任务切换页表 | 进入用户任务前切换到对应用户页表，syscall/trap 后仍正常 | 已完成 |
| 6 | 用户程序加载准备 | 用户入口由 `loader` 模块统一管理 | 已完成 |
| 7 | 独立用户程序构建 | 建立 `user/` 目录并生成用户程序二进制 | 已完成 |
| 8 | 用户程序二进制嵌入 | 内核能读取 `app0.bin` 和 `app1.bin` | 已完成 |
| 9 | 用户程序加载 | 将内嵌二进制复制到用户地址空间并从入口运行 | 下一步 |
| 10 | 基础 syscall 兼容 | 实现 `write`、`exit`、`yield`、`getpid` 等基础 syscall | 未开始 |
| 11 | 测试矩阵 | 建立官方测例通过情况记录 | 未开始 |

## 提交计划

| 次数 | 提交内容 | 状态 |
|---|---|---|
| 1 | 初始化项目 | 已完成 |
| 2 | 引入 rCore baseline 与基础文档 | 已完成 |
| 3 | 自建最小内核启动与 timer interrupt | 已完成 |
| 4 | trap 结构整理与 `TrapContext` | 已完成 |
| 5 | 最小 syscall 与用户态闭环 | 已完成 |
| 6 | `SYS_EXIT` 与最小任务模型 | 已完成 |
| 7 | `TaskContext`、`SYS_YIELD` 与多任务轮转 | 已完成 |
| 8 | 物理页帧分配器 | 已完成 |
| 9 | 地址类型、页表项、最小页表 | 已完成 |
| 10 | 页表区间映射 | 已完成 |
| 11 | 内核地址空间结构 | 已验证，待提交 |
| 12 | 开启内核分页 | 已验证，待提交 |
| 13 | 用户地址空间结构 | 已验证，待提交 |
| 14 | 任务绑定用户地址空间 | 已验证，待提交 |
| 15 | 按任务切换页表 | 已验证，待提交 |
| 16 | 用户程序加载准备 | 已验证，待提交 |
| 17 | 独立用户程序构建 | 已验证，待提交 |
| 18 | 用户程序二进制嵌入自检 | 已验证，待提交 |
| 19 | 用户程序加载到地址空间 | 未开始 |
