# 2026-06-28 下一次评测路线

## 983 分后的主线

2026-06-28 08:12:33 提交的官方结果为 `Accepted / 983.2675500541894`：
basic 四列满分 `408`，BusyBox 合计 `208`，Lua 合计 `35`，libcbench 合计
约 `86.91`，musl-rv libctest `217`，musl-rv LTP `155`。真实 LoongArch
已线上生效，当前最快增长点从“生成 LA ELF”切换为“让 LA 侧多个仍为 0 的
大分区同时产生有效输出”。

用户反馈官方评测一次排队约 2-3 小时，因此本轮不再做小步频繁提交。当前提交
采用激进但隔离的 LA 脚本路线：在已稳定得分的 basic、BusyBox、Lua、
libcbench、libctest、LTP 之后，批量启动 musl/glibc 的 `lmbench`、`iozone`、
`iperf`、`netperf` 和 `cyclictest` no-stress 子集。每个命令都加 timeout，
网络测试使用独立端口并清理 server 进程，目标是用一次评测同时观察多个大分区
是否开始计分。

本轮同时继续扩展 LTP allowlist，加入更多偏 syscall/file/time 的低风险项。
如果下一次结果中任一大分区从 0 变成非 0，下一轮优先沿该组补完整脚本或修
对应 syscall；如果某组导致明显回退，则只撤回该组入口，保留其他已经验证的
得分路径。

## 607 分后的主线

2026-06-27 16:06:27 提交的官方结果为 `Accepted / 607.8318219303549`：
basic=204、BusyBox=100、Lua=18、libcbench=55.565189668706445、
libctest=217、LTP=70。官网四个 LA 汇总列仍全部为 0，当前最高收益方向仍是
让官方环境生成并启动真实 LoongArch ELF。

根构建已改用官方镜像预装的 `nightly-2025-05-20`。本地补齐同源 GCC 13.2
musl 工具链和 Rust LA target 后，`build-la-strict` 已带 lwext4、virtio-blk、
网络和 FP/SIMD 完整通过；若真实 LA 构建仍失败，默认构建继续 fallback 到 RV
占位，避免 RISC-V 基线变成 Compile Error。

真实 LA init 在原 basic 64/64 路径之后按文件存在性增加 BusyBox、Lua、
glibc/musl libcbench、musl libctest 和 42 个受限 LTP case；libcbench、
libctest、LTP 的超时分别为 180 秒、300 秒和单项 5 秒。iozone、cyclictest、
iperf 和 netperf 仍不进入本轮。

RISC-V 第三批从 37 个候选中只保留 9 个双跑通过项，本地合计 91 TPASS，
零 TFAIL/TBROK/timeout/panic。下一次提交的验收顺序是：Compile Accepted、
RISC-V 不低于 607、`kernel-la` 被官方识别为 LoongArch、LA basic 首先非零，
再观察 BusyBox/Lua/libcbench/libctest/LTP。下一轮现实目标为 1100–1400；
2800 仍需后续多轮推进 lmbench/iozone、网络组和性能优化。

源码构建的 QEMU 9.2.1 已能按官方参数启动真实 `kernel-la`。官方 `pre-2025`
源码构建的静态 musl BusyBox 最小盘复跑为 55/55，START/END 完整，零 panic、
零 unsupported syscall，并主动退出；日志暴露的 `/bin/ls` 缺失已修复。
Lua、libcbench、libctest 和 LTP 的 LA 路径仍只完成构建与脚本接入，不能提前
视为线上通过。

## 2026-06-25 Compile Error 修复

最新官方编译错误不是 RISC-V 代码失败，而是默认 `make all` 强制进入
LoongArch 严格工具链检查。评测机缺少 `nightly-2025-02-18` 的
`loongarch64-unknown-none` target，导致 `check-la-tools` 退出 1。
本轮把默认 `build-la` 改为 best-effort：工具链齐全时构建真实 `SWTC-la`，
不可用时复制 `kernel-rv` 为占位 `kernel-la`；真实 LA 构建保留在
`build-la-strict`。

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-28 08:12:33 提交，`Accepted / 983.2675500541894`；basic=408、BusyBox=208、Lua=35、libcbench=86.91038529599213、libctest=217、LTP=155 |
| 最新稳定官方结果 | 2026-06-28 08:12:33 提交，`Accepted / 983.2675500541894`；真实 LA 已上线，basic 四列满分，LA BusyBox/Lua/libcbench 已开始计分 |
| 最新高分结果 | 2026-06-21 13:15:41，`Accepted / 484.26735406790885`；iozone-lite 撤回后已恢复 |
| 已止血问题 | `4602678` 扩容 libctest 后曾在 libcbench-glibc 阶段触发 `src/process/thread/exit.rs:74` 父进程 weak unwrap panic；14:43 结果已恢复且无 panic |
| 上一条通过基线 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.255157002759375`、libctest `107` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 最新线上得分 | basic `204`、BusyBox `98`、Lua `18`、libcbench `0`、libctest `0`、lmbench `0` |
| 当前修复方向 | 先恢复 Compile 阶段，LoongArch 默认 fallback 不再阻塞 RISC-V 基线 |
| 本轮代码基线 | 在 `d6746eb fix: add lmbench runtime skeleton` 基础上，删除全局 `/bin/sh`、loader/lib、`/etc/passwd`、`/tmp/memfd` staging |
| 本轮新增门禁修复 | `64fe8b4` 已撤回 `8690e03 feat: add minimal iozone probe` |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | 本轮新增 LA 大分区只完成脚本语法、资源路径和 RISC-V 构建回归；LA strict 构建在本机因 `nightly-2025-05-20` manifest 损坏无法作为门禁，正式效果等官方评测 |

## 2026-06-24 快速增长四路线

1. **musl dynamic libctest**：static `107/107`、dynamic `110/110` 已线上拿满
   217 分，当前冻结。
2. **LTP 批量队列**：首批线上 44 分；第二批 13 个 case、26 个 TPASS 已本地
   双跑通过，等待线上确认，不恢复会 panic 的宽泛 256 项队列。
3. **LoongArch**：已完成第一阶段并线上生效。当前 LA basic 四列满分，BusyBox、
   Lua、libcbench 已开始得分，本轮继续向 libctest/LTP 和大分区扩展。
4. **性能/网络大分区**：不再在 RV staging 里扩张，改在 LA init 尾部隔离执行
   `lmbench`、`iozone`、`iperf`、`netperf` 和 no-stress `cyclictest`，以
   timeout 控制风险。

第一路线的完整官方镜像运行已经证明此前后半程归零的关键原因之一是 submit
模式仅管理 128 MiB 物理内存。当前已改为匹配官方 `-m 1G`，并通过 256 MiB
无盘兼容回归。新一轮线上结果出来前，以 `983.2675500541894` 作为线上稳定基线。

2026-06-24 已确认 `d500180` 编译通过但线上回退到 320：基础组仍在，
libcbench/libctest 全部丢分，cyclictest 也没有得分。当前已完整撤回该提交；
恢复 480 基线前不再扩大测试组或改全局 VFS 环境。该风险主要来自旧 RV staging
和全局 runtime 污染，本轮 LA init 尾部隔离执行仍需用官方结果确认是否安全。

## 本轮提交门禁

1. `SWTC-la/src/init.sh` 必须通过 `sh -n` 和 `bash -n`。
2. 官方 LA 测试盘中必须存在本轮启用的大分区二进制，避免提交后全部 miss path。
3. RISC-V `make build-rv` 必须通过，`kernel-rv` 为 RISC-V executable ELF，
   入口 `0x80200000`。
4. 默认 `make all` 必须继续生成 `kernel-la`；若本机 LA 工具链不可用，允许占位
   ELF。真实 LoongArch strict 构建以工具链完整环境和官方评测为准。
5. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
6. 官方完整参数下，无盘、稳定 functional 组和新增大分区都不能导致全局超时；
   若单项失败，必须由 timeout 继续推进到后续组。
7. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试；
   lmbench staging 必须提供 `/lmbench_all` 根路径别名。
8. submit 默认构建 feature 应为 `submit tmpfs`，不再带 `stack_trace`；需要诊断时手动传 `STACK_TRACE=1`。
9. LA 大分区 START/END marker 必须使用官方组名，如 `lmbench-musl`、
   `iozone-glibc`、`iperf-musl`、`netperf-glibc`。
10. musl libctest 的 107 个 static 和 110 个 dynamic case 已线上通过，后续
    不得改动其顺序和隔离运行时。
11. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- basic 四列保持 `408/408`。
- BusyBox 四列保持约 `208`。
- Lua 保持约 `35`。
- libcbench 保持约 `86` 分区间。
- `libctest-musl` 保持 `217/217`。
- LTP 至少保持线上 `155`，观察新增 allowlist 是否转化为增量。
- 观察 `lmbench`、`iozone`、`iperf`、`netperf`、`cyclictest` 是否从 0 变成非 0。
- 总分最低目标是不低于当前 `983`；现实目标是突破 `1100`，若网络或 iozone
  任一组命中，有机会一次增加数百。若回退，先按组撤回 LA init 尾部入口。
- 不再出现 `src/process/thread/exit.rs:74` panic。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后用 `A` 或 `C` 记录执行最短命令，确认有 START/END、timeout 生效且主动关机后
   扩展完整脚本。
4. 若出现回退，优先按新增大分区逐组撤回，保留 983 基线。

## 后续提分顺序

1. 本轮先提交 LA 大分区批量探测，减少 2-3 小时评测排队的浪费。
2. 若 LTP/libctest LA 开始计分，优先补 LA functional 缺口，因为这类分数最稳。
3. 若 `iperf` 或 `netperf` 有输出但不计分，下一轮优先修 loopback/socket 语义。
4. 若 `iozone` 有输出但不计分，下一轮优先修文件读写、pwritev/preadv 和 mmap/fsync。
5. 若 lmbench 仍为 0，必须看串口日志确认是否出现 `Simple syscall:`、
   `Select on 100 fd` 或 `Signal handler installation:`；没有这些行就继续修
   执行/超时路径，有这些行但不计分再查 parser 分组。
6. 若 no-stress cyclictest 卡住或拖分，只撤回 cyclictest，不影响其他大分区。

## 本轮暂缓

- 不改 RISC-V `oscomp` staging，不恢复旧的全局 runtime skeleton。
- 不做多核、调度器性能优化或大范围 VFS 重构。
- 不要求用户每个小改都排队评测；本轮完成后只提交一次有信息量的官方评测。
