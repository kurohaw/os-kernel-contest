# OS Kernel Contest

当前主线基于往届 GPLv3 开源作品 Titanix，目标是在其完整的进程、内存、VFS、
异步执行器、网络和驱动架构上完成 2026 官方评测适配。

旧自建内核曾取得官方线上 `basic=102`。当前 `main` 使用 Titanix 重写路线，
已能在同一次 RISC-V 启动中串行运行 glibc、musl basic 和 BusyBox。

## 当前进度

- 根目录 `make all` 可完全离线构建 Titanix。
- 使用官方镜像预装的 Rust `nightly-2025-02-01`，评测时不依赖联网下载。
- 生成官方要求的 RISC-V executable ELF `kernel-rv`。
- 使用官方完整 `1G`、单核、网络设备与 RTC 参数启动并主动关机。
- 从 x0 virtio-blk 测试盘识别 EXT4。
- 固定按 `glibc`、`musl` 顺序收集 basic；两者均不存在时才使用根目录 fixed path。
- 读取 basic 脚本和嵌套 `run-all.sh`，解析并串行执行 basic 测试队列。
- 将每组 basic ELF、资源和动态运行时暂存到独立 tmpfs 工作目录。
- 动态解释器缺失或无效时向 `execve` 返回错误，不再触发 loader panic。
- basic 的 `mount`、`umount` 已恢复执行，线上 RISC-V basic 为 `102 + 102`。
- BusyBox 已按官方脚本暂存并执行，线上 RISC-V BusyBox 为 `49 + 49`。
- Lua 官方脚本、`lua`、`busybox` 和 `.lua` 资源已接入 tmpfs staging，等待下一次线上确认。
- 本地官方 `test_runner.py` 对双组 basic 的解析结果为 `102/102`。
- 官方镜像同版本工具链 `nightly-2025-02-01` 下完成隐藏文件过滤、强制离线构建验证。

官方页面最后可见结果为 2026-06-18 08:55:11：编译状态 `Accepted`，总分
`302.0`。其中 RISC-V basic 为 glibc `102`、musl `102`，BusyBox 为 glibc
`49`、musl `49`。当前新增 Lua staging，下一次评测重点看 Lua 是否开始得分并
保持 basic/BusyBox 不回退。

## 构建

```bash
cd /mnt/d/os-kernel-contest
make all
```

无测试盘启动：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 1G -nographic \
  -smp 1 -bios default -no-reboot \
  -device virtio-net-device,netdev=net -netdev user,id=net -rtc base=utc
```

带官方风格 EXT4 测试盘：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 1G -nographic \
  -smp 1 -bios default \
  -drive file=/path/to/test.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot \
  -device virtio-net-device,netdev=net -netdev user,id=net -rtc base=utc
```

## 目录

- `titanix/kernel/`：当前内核主体。
- `titanix/user/`：内置用户程序。
- `titanix/vendor/`：离线 Cargo 依赖。
- `titanix/dependencies/`：本地底层依赖。
- `docs/`：路线、进度、测试矩阵和来源说明。

开发前请先阅读 `AGENTS.md`。

不要删除 vendor 中的非隐藏 `cargo-checksum.json`。官方提交会过滤隐藏路径，
根 Makefile 依靠这些文件恢复 Cargo 所需的 `.cargo-checksum.json`。
