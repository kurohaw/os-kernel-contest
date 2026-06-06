# 初赛开发进度记录

## 当前状态

| 项目 | 内容 |
|---|---|
| 阶段 | 初赛开发期 |
| 当前日期 | 2026-06-06 |
| 当前仓库 | GitHub: `kurohaw/os-kernel-contest`；GitLab: `gitlab.eduxiji.net/T2026102569910192/oskernel2025-sudo_win_the_cscc` |
| 当前基础版本 | `rCore-Tutorial-v3-main` |
| 主参考作品 | 2024 Phoenix |
| 当前目标 | 接入官方测试磁盘扫描 |
| 当前完成度 | 已完成最小启动、trap、syscall、两任务轮转、物理页帧分配、Sv39 页表基础、区间映射、内核地址空间结构、临时用户段权限映射、Sv39 内核分页开启、用户地址空间自检、任务绑定用户地址空间、按任务切换页表、用户程序 loader 边界、独立用户程序构建、用户程序二进制嵌入自检、用户程序加载运行、`write` syscall、`getpid` syscall、最小 `read` syscall、最小 `brk` syscall、基础文件描述符层、最小 `close` syscall、最小 `fstat` syscall、最小 `openat` syscall、基础文件描述符表、基础文件读取、测试矩阵、官方 RISC-V 提交入口适配、virtio-blk 扇区读取、EXT4 根目录测试脚本扫描 |

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

## 2026-06-04 用户程序加载运行

### 今日目标

将内嵌用户程序二进制复制到用户地址空间，并从用户入口地址运行。

### 修改内容

- 新增 `MemorySet::load_user_app(app_id)`。
- 为用户程序分配物理页帧。
- 将用户程序二进制复制到新分配的物理页。
- 建立 `USER_APP_BASE -> app frame` 的用户页表映射。
- `loader::app_entry()` 改为返回统一用户入口 `0x10000`。
- 更新用户地址空间自检，检查 `USER_APP_BASE` 可读、可执行且带 `U` 权限。
- 移除旧 `.user.text` 自检依赖。

### 结论

独立用户程序二进制已经能加载到用户地址空间运行，当前 `app0` 和 `app1` 仍可通过 `SYS_TEST`、`SYS_YIELD`、`SYS_EXIT` 完成轮转和退出。

## 2026-06-04 write syscall

### 今日目标

实现最小 `write(fd, buf, len)`，让独立用户程序可以通过 syscall 主动输出字符串。

### 修改内容

- 在内核 syscall 分发表中新增 `SYS_WRITE = 64`。
- 新增 `sys_write(fd, buf, len)`，支持 `fd = 1` 和 `fd = 2` 输出到 SBI console。
- 在用户库中新增 `write(fd, &str)` 封装。
- 修改 `app0` 和 `app1`，通过 `write` 输出用户态字符串。

### 结论

用户程序已经可以通过 `write` 输出内容；下一步实现 `getpid`，让用户程序能取得当前任务编号。

## 2026-06-04 getpid syscall

### 今日目标

实现 `getpid()`，让用户程序可以取得当前任务编号，方便后续测试和日志定位。

### 修改内容

- 在内核 syscall 分发表中新增 `SYS_GETPID = 172`。
- 在 `task` 模块新增 `current_task_id()`，返回当前运行任务编号。
- 新增 `sys_getpid()`，将当前任务编号作为 pid 返回给用户态。
- 在用户库中新增 `getpid()` 封装。
- 修改 `app0` 和 `app1`，分别验证返回值为 `0` 和 `1`。

### 结论

用户程序已经可以获得当前任务编号；下一步实现最小 `read` syscall，为后续 stdin 和文件描述符表做接口准备。

## 2026-06-05 read syscall 与 yield 恢复修复

### 今日目标

实现最小 `read(fd, buf, len)` 接口，并验证用户任务在 `read` 后 `yield` 再恢复仍然正确。

### 修改内容

- 在内核 syscall 分发表中新增 `SYS_READ = 63`。
- 新增 `sys_read()`，当前先支持 `fd = 0` 返回 `0`。
- 在用户库中新增 `read(fd, &mut [u8])` 封装。
- 修改 `app0` 和 `app1`，验证 `read(0, buf)` 返回稳定结果。
- 修复 `SYS_YIELD` 路径：yield 时保存当前 trap 产生的新 `TrapContext` 地址。

### 结论

用户程序已经可以调用最小 `read`，并且任务在 yield 后可以从正确用户态位置恢复；下一步实现最小 `brk`，为用户态堆边界和后续分配器适配做准备。

## 2026-06-05 brk syscall

### 今日目标

实现最小 `brk(addr)`，让用户程序可以查询和设置当前任务的堆边界。

### 修改内容

- 在 `loader` 中新增 `USER_HEAP_BASE` 和 `USER_HEAP_SIZE`。
- 在 `TaskControlBlock` 中新增 `heap_bottom` 和 `heap_end`。
- 新增 `set_current_brk()`，支持 `brk(0)` 查询和有限范围内设置堆边界。
- 在内核 syscall 分发表中新增 `SYS_BRK = 214`。
- 在用户库中新增 `brk(addr)` 封装。
- 修改 `app0` 和 `app1`，验证 `brk` 查询和设置结果。

### 结论

最小 `brk` 状态管理已经完成；当前尚未真实映射新增堆页，下一步先建立基础文件描述符层，把 `read/write` 从 syscall 模块中拆出，为 `close/fstat/open` 做准备。

## 2026-06-05 基础文件描述符层

### 今日目标

建立最小 `fs` 模块，把 `read/write` 的 stdin/stdout/stderr 处理从 syscall 分发层拆出。

### 修改内容

- 新增 `kernel/src/fs.rs`。
- 定义 `STDIN`、`STDOUT`、`STDERR`。
- 将 `read(fd, buf, len)` 移入 `fs` 模块，当前 `STDIN` 返回 `0`。
- 将 `write(fd, buf, len)` 移入 `fs` 模块，当前 `STDOUT/STDERR` 输出到 SBI console。
- 在 `main.rs` 中接入 `mod fs;`。
- `syscall.rs` 中的 `sys_read()` 和 `sys_write()` 改为调用 `crate::fs`。

### 结论

基础文件描述符层已经建立；下一步实现最小 `close` syscall，为后续 `fstat/open` 和文件描述符表扩展做准备。

## 2026-06-05 close syscall

### 今日目标

实现最小 `close(fd)`，让标准文件描述符具备可调用的关闭接口。

### 修改内容

- 在内核 syscall 分发表中新增 `SYS_CLOSE = 57`。
- 在 `fs` 模块中新增 `close(fd)`。
- 当前 `close(0/1/2)` 返回 `0`，非法 fd 返回 `-1`。
- 在用户库中新增 `close(fd)` 封装。
- 修改 `app0` 和 `app1`，验证标准 fd 和非法 fd 的返回值。

### 结论

最小 `close` syscall 已经完成；下一步实现最小 `fstat`，让标准 fd 可以返回基础状态信息，适配更多 libc/测例路径。

## 2026-06-05 fstat syscall

### 今日目标

实现最小 `fstat(fd, stat)`，让标准文件描述符可以返回基础状态信息。

### 修改内容

- 在 `fs` 模块中新增最小 stat buffer 大小和 mode 字段写入。
- 新增 `fs::fstat(fd, stat_buf)`，标准 fd 返回 `0`，非法 fd 返回 `-1`。
- 在内核 syscall 分发表中新增 `SYS_FSTAT = 80`。
- 新增 `sys_fstat()`，转发到 `fs` 模块。
- 在用户库中新增 `STAT_SIZE` 和 `fstat(fd, buf)` 封装。
- 修改 `app0` 和 `app1`，验证 stdout 和非法 fd 的 `fstat` 结果。

### 结论

最小 `fstat` 已经完成，标准 fd 可以返回基础 stat 信息；下一步可以开始做最小 `open` 或正式建立文件描述符表。

## 2026-06-05 openat syscall

### 今日目标

实现最小 `openat(dirfd, path, flags, mode)`，让用户程序可以打开基础路径并得到稳定 fd。

### 修改内容

- 将 syscall 参数从 3 个扩展为 4 个，支持 `a0-a3`。
- 在内核 syscall 分发表中新增 `SYS_OPENAT = 56`。
- 在 `fs` 模块中新增 `/dev/null` 最小路径识别。
- `openat("/dev/null")` 返回固定 fd `3`，不存在路径返回 `-1`。
- `read/write/close/fstat` 支持 `/dev/null` 对应 fd。
- 在用户库中新增 `open()` 和 `openat()` 封装。
- 修改 `app0` 和 `app1`，验证打开 `/dev/null`、关闭 fd 和打开不存在路径。

### 结论

最小 `openat` 已经完成，当前可以打开 `/dev/null` 并返回稳定 fd；下一步需要从固定 fd 过渡到基础文件描述符表，为真实文件读取做准备。

## 2026-06-05 基础文件描述符表

### 今日目标

将 `/dev/null` 从固定 fd 过渡到基础文件描述符表，支持动态分配和释放 fd。

### 修改内容

- 在 `fs` 模块中新增动态 fd 表。
- 新增 `FileKind::DevNull`，用 fd 表记录打开的文件对象类型。
- `openat("/dev/null")` 改为从 fd 表中分配空闲 fd。
- `close(fd)` 对动态 fd 执行释放，重复关闭已释放 fd 返回失败。
- `read/write/fstat` 通过 fd 表识别动态 fd。
- 修改 `app0` 和 `app1`，验证连续分配、关闭和重复关闭动态 fd。

### 结论

基础文件描述符表已经完成，fd 不再只依赖固定编号；下一步可以开始加入一个只读内嵌文件，验证 `openat + read + close` 的真实文件读取路径。

## 2026-06-05 基础文件读取

### 今日目标

加入一个内嵌只读文件，验证 `openat + read + close` 的真实文件读取路径。

### 修改内容

- 在 `fs` 模块中新增内嵌文件 `/hello.txt`。
- 将 fd 表项从单纯 `FileKind` 扩展为 `FileDescriptor`，保存文件类型和读取偏移。
- 新增 `FileKind::Hello`。
- `openat("/hello.txt")` 可以分配动态 fd。
- `read(fd, buf, len)` 可以从内嵌文件复制内容到用户缓冲区，并更新读取偏移。
- 第二次读取到文件末尾时返回 `0`。
- 修改 `app0` 和 `app1`，验证打开文件、读取内容、EOF 和关闭文件。

### 结论

基础文件读取已经完成，当前可以通过 `openat + read + close` 读取内嵌只读文件；下一步建议建立测试矩阵，把已支持 syscall 和验证结果系统记录下来。

## 2026-06-06 测试矩阵

### 今日目标

建立 `docs/test-matrix.md`，记录当前 syscall、用户程序验证点、文件系统验证点和已知限制。

### 修改内容

- 新增当前验证命令。
- 记录已支持 syscall：`test`、`exit`、`yield`、`openat`、`close`、`read`、`write`、`fstat`、`getpid`、`brk`。
- 记录 `app0/app1` 当前验证点。
- 记录 `/dev/null`、`/hello.txt`、`/missing` 的文件系统行为。
- 记录当前限制和下一步待测方向。

### 结论

测试矩阵已经建立；后续接入官方测例时，可以继续在该文档中追加测试名、结果和阻塞 syscall。

## 2026-06-06 官方提交入口适配

### 今日目标

根据官方初赛提交说明，先把项目从本地 RustSBI loader 流程切到评测机要求的根目录构建和 RISC-V QEMU 启动形式。

### 修改内容

- 根目录 `make all` 生成 ELF 格式 `kernel-rv`，并临时生成 `kernel-la` 占位文件。
- 保留旧 `exec.out` 裸二进制产物，避免破坏已有本地流程。
- 将 `make run` 调整为 `qemu-system-riscv64 -kernel kernel-rv -bios default` 风格，旧 RustSBI loader 流程保留为 `run-loader`。
- 将 Cargo target/linker 配置复制到非隐藏 `cargo-config/`，构建前再恢复到 `.cargo/config.toml`，适配评测系统过滤隐藏目录的行为。
- 增加 SBI shutdown，所有内嵌自测任务退出后主动关闭 QEMU。

### 验证结果

`make all` 通过；`kernel-rv` 为 RISC-V ELF。使用官方风格 QEMU 命令可启动到 `kernel started`，完成 `app0/app1` 自测，并在 `all tasks exited` 后退出 QEMU。

### 结论

RISC-V 提交入口已经与官方说明对齐到第一层。下一步不应继续只补自测 syscall，而应优先接入 virtio-blk 与 EXT4 测试磁盘扫描，找到并运行 `xxxxx_testcode.sh`。

## 2026-06-06 virtio-blk 与 EXT4 测试脚本扫描

### 今日目标

继续打通官方测试入口：在官方风格 QEMU 挂载测试盘后，内核能从 virtio-blk 设备读取无分区 EXT4 镜像，并扫描根目录中的 `*_testcode.sh`。

### 修改内容

- 新增 `kernel/src/drivers/block.rs`，支持 QEMU virtio-mmio block 设备探测。
- 支持 legacy virtio-mmio v1 的队列初始化，关键修复是写入 `GuestPageSize`，否则 QEMU 不消费队列。
- 暴露 `block::read_sector(sector, buffer)`，为文件系统层提供 512 字节扇区读取接口。
- 映射 QEMU virtio-mmio 区间 `0x10000000..0x10009000`。
- 新增 `kernel/src/drivers/ext4.rs`，实现最小只读 EXT4 子集：
  - 读取 superblock。
  - 读取 group descriptor 和 root inode。
  - 支持 direct block 和 depth=0/1 的 extent tree。
  - 遍历根目录 dirent，识别 `*_testcode.sh`。
- `drivers::init()` 现在先初始化 virtio-blk，再尝试扫描 EXT4 测试盘。

### 验证结果

`make all` 通过。

无测试盘回归：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default -no-reboot
```

可以正常输出 `virtio-blk: device not found`，继续运行内嵌 `app0/app1`，并在 `all tasks exited` 后关机。

本地 EXT4 测试盘验证：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default \
  -drive file=/tmp/oskernel-ext4.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot \
  -device virtio-net-device,netdev=net -netdev user,id=net -rtc base=utc
```

关键输出：

```text
virtio-blk: device found at 0x10001000, version=1
virtio-blk: ready
oscomp: found test script basic_testcode.sh
oscomp: found test script lua_testcode.sh
ext4: found 2 test script(s)
```

### 结论

官方测试入口已经推进到“能识别挂载测试盘并列出根目录测试脚本”。下一步需要读取脚本文件内容，按官方要求输出测试组起止标记；随后从脚本中定位 ELF 测试程序，接入 ELF loader、用户栈参数和进程模型。

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
| 9 | 用户程序加载 | 将内嵌二进制复制到用户地址空间并从入口运行 | 已完成 |
| 10 | `write` syscall | 用户程序能通过 `write` 输出字符串 | 已完成 |
| 11 | `getpid` syscall | 用户程序能取得当前任务编号 | 已完成 |
| 12 | 最小 `read` syscall | 用户程序能调用 `read(0, buf, len)` 并得到稳定返回值 | 已完成 |
| 13 | 最小 `brk` syscall | 用户程序能查询当前堆边界并得到稳定返回值 | 已完成 |
| 14 | 基础文件描述符层 | `read/write` 通过 fd 模块处理 stdin/stdout/stderr | 已完成 |
| 15 | 最小 `close` syscall | `close(0/1/2)` 返回成功，非法 fd 返回失败 | 已完成 |
| 16 | 最小 `fstat` syscall | `fstat(0/1/2, stat)` 返回成功并清零 stat buffer | 已完成 |
| 17 | 最小 `openat` syscall | 能处理基础路径并返回稳定 fd 或错误 | 已完成 |
| 18 | 基础文件描述符表 | fd 不再只依赖固定编号，支持后续文件对象管理 | 已完成 |
| 19 | 基础文件读取 | 打开内嵌只读文件后可以通过 `read` 读取内容 | 已完成 |
| 20 | 测试矩阵 | 建立当前 syscall 和文件读取验证记录 | 已完成 |
| 21 | 官方 RISC-V 提交入口 | `make all` 生成 `kernel-rv`，官方风格 QEMU 能启动并主动退出 | 已完成 |
| 22 | 测试磁盘扫描 | 识别官方挂载的 virtio-blk EXT4 测试盘并列出测试脚本 | 已完成 |
| 23 | 官方脚本入口 | 串行运行或按格式跳过 `xxxxx_testcode.sh` 测试点 | 下一步 |
| 24 | 真实堆页映射 | `brk` 增长后能访问新映射用户页 | 未开始 |
| 25 | 官方测例矩阵 | 接入比赛测例并记录通过情况 | 未开始 |

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
| 19 | 用户程序加载到地址空间 | 已验证，待提交 |
| 20 | `write` syscall | 已完成 |
| 21 | `getpid` syscall | 已验证，待提交 |
| 22 | 最小 `read` syscall 与 yield 恢复修复 | 已验证，待提交 |
| 23 | 最小 `brk` syscall | 已验证，待提交 |
| 24 | 基础文件描述符层 | 已验证，待提交 |
| 25 | 最小 `close` syscall | 已完成 |
| 26 | 最小 `fstat` syscall | 已验证，待提交 |
| 27 | 最小 `openat` syscall | 已验证，待提交 |
| 28 | 基础文件描述符表 | 已验证，待提交 |
| 29 | 基础文件读取 | 已验证，待提交 |
| 30 | 测试矩阵 | 已完成 |
| 31 | 真实堆页映射 | 暂缓 |
| 32 | virtio-blk 扇区读取 | 已完成 |
| 33 | EXT4 根目录测试脚本扫描 | 已完成 |
