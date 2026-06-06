# OS Kernel Contest

操作系统内核比赛开发仓库。

当前开发路线：

- 以 `rCore-Tutorial-v3-main` 作为学习和参考 baseline。
- 在 `kernel/` 中实现自建 Rust/RISC-V 内核。
- 在 `user/` 中维护独立用户程序，编译为二进制后由内核嵌入、加载和运行。

## 当前进度

截至 2026-06-06，`kernel/` 已完成：

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
- 基础 syscall：`write`、`read`、`openat`、`close`、`fstat`、`getpid`、`brk` 等最小路径。
- 官方 RISC-V 提交入口：根目录 `make all` 生成 `kernel-rv` 和临时 `kernel-la`。
- QEMU virtio-blk 扇区读取。
- 无分区 EXT4 测试盘根目录扫描，识别并读取 `*_testcode.sh`，输出官方测试组 START/END 标记。
- 从测试盘读取 RISC-V ELF，按 `PT_LOAD` segment 映射并进入用户态运行。
- 外部 ELF 最小启动栈：`argc=1`、`argv[0]`、空 `envp` 和基础 `auxv`。
- 外部 ELF 通过 `openat/read/fstat` 读取 EXT4 根目录普通文件。
- `brk` 增长时映射真实用户堆页，外部程序可以写入新增堆区。

当前已经可以用本地 EXT4 测试盘加载并运行放在盘上的 `app0` ELF，也可以让外部 ELF 读取同盘根目录普通文件、增长并写入用户堆。下一阶段目标是让官方 basic/busybox ELF 通过 libc 启动早期路径，重点补路径解析、进程模型和 Linux ABI syscall。

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
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default -no-reboot
```

根目录 `Makefile` 会先构建 `user/` 中的用户程序，再构建内核并生成官方要求的产物。

带 EXT4 测试盘的本地验证命令示例：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default \
  -drive file=/tmp/oskernel-ext4.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot \
  -device virtio-net-device,netdev=net -netdev user,id=net -rtc base=utc
```

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
