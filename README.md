# SWTC

SWTC 是面向 2026 操作系统大赛评测开发的双架构操作系统。RISC-V 主线参考往届
GPLv3 开源作品 Titanix 的架构进行开发；LoongArch 主线参考 2025 一等奖开源作品
StarryX 及其 ArceOS 基础。两条主线均保留上游许可证、作者与来源记录，并持续完成
本队自己的官方评测适配、兼容修复和离线构建。

旧自建内核曾取得官方线上 `basic=102`。当前 `main` 的 RISC-V 内核已能串行运行
glibc、musl basic、BusyBox、Lua、libcbench、musl libctest 和稳定 LTP 子集。
根构建优先生成真实 LoongArch `kernel-la`；工具链或真实构建失败时仍复制
RISC-V 占位 ELF，避免阻塞已稳定的 RISC-V 评测。

最新可见线上结果为 2026-06-27 16:06:27 提交：
`Accepted / 607.8318219303549`。RISC-V basic=204、BusyBox=100、Lua=18、
libcbench=55.565189668706445、libctest=217、LTP=70；LoongArch 仍为 0。

## 2026-06-25 双架构里程碑

- `make all` 同时生成 RISC-V `kernel-rv` 和 `kernel-la`；后者可能是保分占位。
- `kernel-rv`：RISC-V executable ELF，入口 `0x80200000`。
- `build-la-strict` 成功时，`kernel-la` 为 LoongArch executable ELF，入口
  `0x80000000`。
- LoongArch 使用官方 `pre-20250615` 测试镜像本地验证：musl basic `32/32`、
  glibc basic `32/32`，共 64 个 START/END 完整匹配，无 panic 或 loader 错误。
- LoongArch 构建依赖已全部 vendor，`axconfig-gen` 也从仓库离线构建。
- 上述 LoongArch 结果是本地官方镜像证据，尚不能写成线上官方分数。

## 当前进度

- 根目录 `make all` 可完全离线构建 SWTC。
- RISC-V 使用官方镜像预装的 `nightly-2025-02-01`；LoongArch 独立使用
  `nightly-2025-05-20`，根构建不会再用一个全局工具链覆盖两套架构。
- 生成官方要求的 RISC-V `kernel-rv` 和 `kernel-la`；缺预编译 LA target
  但有 `rust-src` 时尝试 build-std，失败时安全 fallback。
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
- lmbench 保留 9-command lite 探针，并保留 `/lmbench_all`、`lat_sig`、
  `/var/tmp/lmbench`、`/var/tmp/XXX` 和 `/tmp/hello` 等兼容资源；submit 构建
  默认关闭 `stack_trace`，减少 syscall/文件热路径的调试栈记录开销。上一轮
  全局 `/bin/sh`、`/lib`、`/etc/passwd` 等运行环境骨架导致线上回退到 320，
  当前已撤回。
- futex 已补 `WAIT_BITSET`/`WAKE_BITSET`，未知 futex op 返回错误而不是 panic，
  用于推进 libcbench pthread 段。
- 当前转向稳定组内部优化：修复 futex wait 超时和 stale waiter、requeue 计数、
  无唤醒时的多余 yield，补 `set/get_robust_list` 最小语义，并给
  `clock_gettime/getres` 常见 clock id 增加免锁快路径。
- 继续补 libc/lmbench 常见 syscall 兼容：`getrusage` 支持
  `RUSAGE_THREAD/RUSAGE_CHILDREN` 且不再 panic，`times` 返回真实 tick，
  `sched_getaffinity(0, ...)` 指向当前线程，`fcntl` 文件锁探针不再触发
  `todo!()`，`lseek` 支持只写 fd 并修正负偏移边界。
- 两次 iozone staging 尝试均导致线上分数回退到 320，当前已撤回该接入并暂停
  iozone 方向，优先恢复 484 得分基线。
- 本地官方 `test_runner.py` 对双组 basic 的解析结果为 `102/102`。
- 两套架构按各自固定工具链完成隐藏文件过滤、强制离线构建验证。

本轮 LoongArch 冲刺已用官方同源 GCC 13.2 musl 工具链完成严格构建，生成入口
`0x80000000` 的真实 LoongArch ELF；basic 后按文件存在性执行 BusyBox、Lua、
glibc/musl libcbench、musl libctest 和 42 个受限 LTP case。RISC-V 同时新增
9 个双跑通过的 LTP case，共产生 91 个 TPASS。以上增量仍待线上确认，不应提前
计入官方分数。

最近可信线上结果为 2026-06-23 18:05:27：编译状态 `Accepted`，总分
`484.32498298746674`。其中 RISC-V basic 为 `204`，BusyBox 为 `98`，Lua 为
`18`，libcbench 为 `57.32498298746679`，musl libctest 为 `107`。本轮新增的
dynamic libctest、LTP 22 项和 LoongArch basic 仍待新一轮线上确认。

上一条稳定复测结果为 2026-06-21 13:36:45：编译状态 `Accepted`，总分
`483.16564668235225`。其中 RISC-V basic 为 `204`，BusyBox 为 `98`，Lua 为
`18`，libcbench 为 `56.165646682352225`，musl libctest 为 `107`。

上一条高分恢复结果为 2026-06-21 13:15:41：编译状态 `Accepted`，总分
`484.26735406790885`。这说明撤回 `8690e03` 的 iozone-lite 后已经恢复 484
基线，且当前分数波动主要来自 libcbench 性能项。

上一条回归结果为 2026-06-21 13:04:01：编译状态 `Accepted`，总分 `320.0`。
该结果来自 `8690e03` 的 iozone-lite 尝试，basic、BusyBox、Lua 保持得分，但
libcbench 和 libctest 均回到 0；当前已通过 revert 撤回该接入。

下一轮重点先验证撤回运行环境骨架和 syscall 兼容修复后是否恢复 480 基线并小幅提分；
恢复后再继续研究 lmbench 真实串口输出，不再用全局 `/lib`、`/bin/sh` 或
`/etc/passwd` 盲试。

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

LoongArch 构建还需要官方镜像已提供的 `cmake`、
`loongarch64-linux-musl-gcc` 和 `libclang`。

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
- `SWTC-la/`：SWTC LoongArch 内核，包含 ArceOS/StarryX 适配和独立 vendor。
- `SWTC/LICENSE`：上游 GPLv3 许可证和来源信息，必须保留。
- `docs/`：路线、进度、测试矩阵和来源说明。

开发前请先阅读 `AGENTS.md`。

不要删除 vendor 中的非隐藏 `cargo-checksum.json`。官方提交会过滤隐藏路径，
根 Makefile 依靠这些文件恢复 Cargo 所需的 `.cargo-checksum.json`。
