# OS Kernel Contest

当前主线基于往届 GPLv3 开源作品 Titanix，目标是在其完整的进程、内存、VFS、
异步执行器、网络和驱动架构上完成 2026 官方评测适配。

旧自建内核曾取得官方线上 `basic=102`。当前 `main` 使用 Titanix 重写路线，
已跑通首个真实 basic ELF。

## 当前进度

- 根目录 `make all` 可完全离线构建 Titanix。
- 生成官方要求的 RISC-V executable ELF `kernel-rv`。
- 使用官方风格 `256M`、单核 QEMU 命令启动并主动关机。
- 从 x0 virtio-blk 测试盘识别 EXT4。
- 命中 `musl/basic_testcode.sh`、`glibc/basic_testcode.sh` 或根目录 fixed path。
- 读取 basic 脚本和嵌套 `run-all.sh`，解析并串行执行 basic 测试队列。
- 将 30 个安全 basic ELF、`test_echo` 和 `text.txt` 暂存到 tmpfs。
- 主动跳过当前会触发内核 panic 的 `mount`、`umount`。
- 本地官方 `test_runner.py` 对完整队列的解析结果为 `91/102`。
- 官方镜像同版本工具链 `nightly-2025-02-01` 下完成隐藏文件过滤、强制离线构建验证。

官方页面最后可见结果仍是
2026-06-11 19:44:39 的旧提交：`0.00 / Compile Error`；该结果尚未覆盖
2026-06-12 已推送到 `gitlab/main` 的离线构建修复提交 `bab4cd0`，也尚未覆盖
当前未提交的完整 basic 队列实现。

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
