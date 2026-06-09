# OS Kernel Contest

当前主线基于往届 GPLv3 开源作品 Titanix，目标是在其完整的进程、内存、VFS、
异步执行器、网络和驱动架构上完成 2026 官方评测适配。

旧自建内核的官方 `basic=102` 版本保存在 `codex/basic-102-archive`。当前
`codex/titanix-architecture` 是重写开发分支，尚未恢复真实 basic ELF 执行。

## 当前进度

- 根目录 `make all` 可完全离线构建 Titanix。
- 生成官方要求的 RISC-V executable ELF `kernel-rv`。
- 使用官方风格 `256M`、单核 QEMU 命令启动并主动关机。
- 从 x0 virtio-blk 测试盘识别 EXT4。
- 命中 `musl/basic_testcode.sh`、`glibc/basic_testcode.sh` 或根目录 fixed path。
- 输出 basic START/END，证明已经进入官方 basic 测试入口。

当前还没有从 EXT4 读取脚本内容、加载 basic ELF 或恢复官方分数。

## 构建

```bash
cd /mnt/d/os-kernel-contest
make all
```

无测试盘启动：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic \
  -smp 1 -bios default -no-reboot
```

带官方风格 EXT4 测试盘：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic \
  -smp 1 -bios default \
  -drive file=/path/to/test.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot
```

## 目录

- `titanix/kernel/`：当前内核主体。
- `titanix/user/`：内置用户程序。
- `titanix/vendor/`：离线 Cargo 依赖。
- `titanix/dependencies/`：本地底层依赖。
- `docs/`：路线、进度、测试矩阵和来源说明。

开发前请先阅读 `AGENTS.md`。
