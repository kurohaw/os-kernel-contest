# SWTC

SWTC 是面向 2026 操作系统大赛评测开发的 RISC-V 操作系统。项目参考往届
GPLv3 开源作品 Titanix 的架构进行开发，在其进程、内存、VFS、异步执行器、
网络和驱动等设计基础上，持续完成本队自己的官方评测适配、功能补全和稳定性修复。

旧自建内核曾取得官方线上 `basic=102`。当前 `main` 使用 SWTC 主线，已能在
同一次 RISC-V 启动中串行运行 glibc、musl basic、BusyBox、Lua、libcbench
和 musl libctest 等测试组。

## 当前进度

- 根目录 `make all` 可完全离线构建 SWTC。
- 使用官方镜像预装的 Rust `nightly-2025-02-18`，评测时不依赖联网下载。
- 生成官方要求的 RISC-V executable ELF `kernel-rv`。
- 使用官方完整 `1G`、单核、网络设备与 RTC 参数启动并主动关机。
- 从 x0 virtio-blk 测试盘识别 EXT4。
- 固定按 `glibc`、`musl` 顺序收集 basic；两者均不存在时才使用根目录 fixed path。
- 读取 basic 脚本和嵌套 `run-all.sh`，解析并串行执行 basic 测试队列。
- 将每组 basic ELF、资源和动态运行时暂存到独立 tmpfs 工作目录。
- 动态解释器缺失或无效时向 `execve` 返回错误，不再触发 loader panic。
- basic 的 `mount`、`umount` 已恢复执行，线上 RISC-V basic 为 `102 + 102`。
- BusyBox 已按官方脚本暂存并执行，线上 RISC-V BusyBox 为 `49 + 49`。
- Lua 官方脚本、`lua`、`busybox` 和 `.lua` 资源已接入 tmpfs staging，线上 RISC-V
  Lua 为 `9 + 9`。
- libcbench 官方脚本、`busybox` 和静态 `libc-bench` 已接入 tmpfs staging，线上
  glibc-rv、musl-rv 均已开始得分。
- musl `libctest` static 全量已线上通过；当前只识别官方
  `/musl/libctest_testcode.sh`，读取 `run-static.sh`，暂存
  `entry-static.exe` 和可选 `runtest.exe`。官方 `static.txt` 归一化后的 107 个
  case 已全部进入 musl-rv 得分。
- futex 已补 `WAIT_BITSET`/`WAKE_BITSET`，未知 futex op 返回错误而不是 panic，
  用于推进 libcbench pthread 段。
- 两次 iozone staging 尝试均导致线上分数回退到 320，当前已撤回该接入并暂停
  iozone 方向，优先恢复 484 得分基线。
- 本地官方 `test_runner.py` 对双组 basic 的解析结果为 `102/102`。
- 官方镜像同版本工具链 `nightly-2025-02-18` 下完成隐藏文件过滤、强制离线构建验证。

最新可见结果为 2026-06-21 13:50:12：编译状态 `Accepted`，总分
`483.52722370911204`。其中 RISC-V basic 为 `204`，BusyBox 为 `98`，Lua 为
`18`，libcbench 为 `56.527223709112`，musl libctest 为 `107`。相比 13:36 的
`483.16564668235225` 略有回升；basic、BusyBox、Lua 和 libctest 均保持稳定，
当前波动仍主要来自 libcbench 性能项。

上一条稳定复测结果为 2026-06-21 13:36:45：编译状态 `Accepted`，总分
`483.16564668235225`。其中 RISC-V basic 为 `204`，BusyBox 为 `98`，Lua 为
`18`，libcbench 为 `56.165646682352225`，musl libctest 为 `107`。

上一条高分恢复结果为 2026-06-21 13:15:41：编译状态 `Accepted`，总分
`484.26735406790885`。这说明撤回 `8690e03` 的 iozone-lite 后已经恢复 484
基线，且当前分数波动主要来自 libcbench 性能项。

上一条回归结果为 2026-06-21 13:04:01：编译状态 `Accepted`，总分 `320.0`。
该结果来自 `8690e03` 的 iozone-lite 尝试，basic、BusyBox、Lua 保持得分，但
libcbench 和 libctest 均回到 0；当前已通过 revert 撤回该接入。

下一轮在没有完整官方串口日志前，不再新增或暂存 iozone，也不再盲目增加新测试组。
后续提分改从现有稳定组暴露的小 syscall/VFS 语义缺口入手。

上一条高分结果为 2026-06-21 11:49:38：编译状态 `Accepted`，总分
`412.92336789756513`。其中 RISC-V basic 为 `204`，BusyBox 为 `98`，Lua 为
`18`，libcbench 为 `56.92336789756515`，musl libctest 为 `36`。

上一条高分可见结果为 2026-06-18 09:46:55：编译状态 `Accepted`，总分
`377.3228370332187`。其中 RISC-V basic 为 glibc `102`、musl `102`，BusyBox 为
glibc `49`、musl `49`，Lua 为 glibc `9`、musl `9`，libcbench 为 glibc
`30.15271484677692`、musl `27.170122186441827`。

上一轮 iozone 完整脚本回归结果为 2026-06-18 16:00:21：编译状态 `Accepted`，
总分 `320.0`。其中 RISC-V basic 为 glibc `102`、musl `102`，BusyBox 为 glibc
`49`、musl `49`，Lua 为 glibc `9`、musl `9`，libcbench 和 iozone 均为 0。
该结果来自 iozone staging 尝试后的回归，已撤回 iozone 接入。

更早一轮可见结果为 2026-06-18 09:33:47：编译状态 `Accepted`，总分
`326.0`。其中 RISC-V basic 为 glibc `102`、musl `102`，BusyBox 为 glibc
`49`、musl `49`，Lua 为 glibc `9`、musl `9`，libcbench glibc 为 `6`。当前修复
futex bitset 兼容性，下一次评测重点看 libcbench 是否继续进分并保持已有分数不回退。

上一轮可见结果为 2026-06-18 09:16:08：编译状态 `Accepted`，总分
`320.0`。其中 RISC-V basic 为 glibc `102`、musl `102`，BusyBox 为 glibc
`49`、musl `49`，Lua 为 glibc `9`、musl `9`。

上一轮可见结果为 2026-06-18 08:55:11：编译状态 `Accepted`，总分
`302.0`。其中 RISC-V basic 为 glibc `102`、musl `102`，BusyBox 为 glibc
`49`、musl `49`。

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

- `SWTC/kernel/`：SWTC 当前内核主体。
- `SWTC/user/`：SWTC 内置用户程序，包含官方提交模式使用的 `runtestcase`。
- `SWTC/vendor/`：SWTC 离线 Cargo 依赖，官方评测构建依赖它。
- `SWTC/dependencies/`：SWTC 使用的本地底层依赖。
- `SWTC/LICENSE`：上游 GPLv3 许可证和来源信息，必须保留。
- `docs/`：路线、进度、测试矩阵和来源说明。

开发前请先阅读 `AGENTS.md`。

不要删除 vendor 中的非隐藏 `cargo-checksum.json`。官方提交会过滤隐藏路径，
根 Makefile 依靠这些文件恢复 Cargo 所需的 `.cargo-checksum.json`。
