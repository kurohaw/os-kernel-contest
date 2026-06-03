# 初赛开发进度记录

## 当前状态

| 项目 | 内容 |
|---|---|
| 阶段 | 初赛开发期 |
| 当前日期 | 2026-06-03 |
| 当前仓库 | GitHub: `kurohaw/os-kernel-contest`；GitLab: `gitlab.eduxiji.net/T2026102569910192/oskernel2025-sudo_win_the_cscc` |
| 当前基础版本 | `rCore-Tutorial-v3-main` |
| 主参考作品 | 2024 Phoenix |
| 当前目标 | 开启内核分页，并确认用户态临时代码和用户栈在分页下仍可运行 |
| 当前完成度 | 已完成最小启动、trap、syscall、两任务轮转、物理页帧分配、Sv39 页表基础、区间映射、内核地址空间结构和临时用户段权限映射 |

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

## 下一组任务

| 顺序 | 任务 | 完成标准 | 状态 |
|---|---|---|---|
| 1 | 设计内核地址空间结构 | `MemorySet::new_kernel()` 能完成内核段恒等映射自检 | 已完成 |
| 2 | 开启内核分页 | 写入 `satp` 后内核仍能启动、打印、处理中断 | 下一步 |
| 3 | 设计用户地址空间 | 用户入口和用户栈通过页表映射后仍能 syscall | 未开始 |
| 4 | 用户程序加载 | 从内嵌用户程序或 ELF 构造用户地址空间 | 未开始 |
| 5 | 基础 syscall 兼容 | 实现 `write`、`exit`、`yield`、`getpid` 等基础 syscall | 未开始 |
| 6 | 测试矩阵 | 建立官方测例通过情况记录 | 未开始 |

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
| 12 | 开启内核分页 | 未开始 |
