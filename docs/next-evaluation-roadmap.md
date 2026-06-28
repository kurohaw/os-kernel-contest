# 2026-06-28 下一次评测路线

2800 分分区缺口、三轮冲刺目标和评测结果分叉处理见
[`score-2800-roadmap.md`](score-2800-roadmap.md)。
当前排队评测出结果后，先按
[`post-evaluation-triage.md`](post-evaluation-triage.md) 提取关键字段；若
983 基线恢复，再从 [`ltp-next-candidates.md`](ltp-next-candidates.md) 选择
下一批 LTP 候选。

## 2026-06-29 983 分后的截断修复

2026-06-29 03:06:43 官方日志内嵌 JSON 为 `Accepted / 983.6805892892955`。
页面 `JSON格式错误` 仍由平台侧 `gzip: sdcard-rv.img already exists; not
overwritten` 污染 JSON 输出导致，不代表本仓库编译失败。

本轮真正阻塞提分的是 LoongArch 执行顺序：LA 已跑完 basic、BusyBox、Lua 和
`libcbench-musl`，随后在 `libcbench-glibc` 触发
`Unhandled trap Exception(MemoryAccessAddressError)`。由于 libctest、LTP 和
lmbench 放在它后面，LA 侧大分区被截断，导致 `libctest-musl-la=0`、
`ltp-musl-la=0`。

下一次提交先跳过 LA `libcbench-glibc`：

1. 保留已计分的 LA `libcbench-musl`。
2. 先运行 LA libctest、LTP 和 lmbench，争取释放 `+217` libctest 和 LTP 增量。
3. glibc libcbench 当前线上为 `0` 且会 panic，等定位 LoongArch
   `MemoryAccessAddressError` 后再恢复。

## 2026-06-28 982 分后的本轮动作

2026-06-28 22:55:36 开始的官方结果为 `Accepted / 982.3134986891687`：
basic 四列满分 `408`，BusyBox 合计 `208`，Lua 合计 `36`，libcbench 合计
约 `84.78`，musl-rv libctest `217`，musl-rv LTP `156`。LoongArch
basic/BusyBox/Lua/libcbench 仍在计分，但 LA libctest 和 LA LTP 仍为 `0`。

按图二要求，本轮不再做十几分微调，进入大分路线：

1. LTP 优先：在已验证 43 个 case 的基础上，追加批次 A 中有现有 syscall
   路径支撑的 55 个候选；暂不加入 eventfd 和已知风险项。
2. LA libctest 补分：保持逐 case 执行和 START/END marker，把 LoongArch
   `runtest.exe -w entry-*.exe case` 单项 timeout 从 3 秒放宽到 8 秒，降低
   慢启动导致 217 分整组为 0 的风险。
3. lmbench、iozone、network、cyclictest 继续后置，不和本轮 LTP/libctest
   首次大扩容混在一起。

## 2026-06-29 继续压 LTP 大分区

上一轮已经把批次 A 主体放入队列。本轮继续走图二的 LTP 大分路线，不等下一次
评测再做十几分小步：

1. RISC-V 补齐 `preadv/pwritev/preadv2/pwritev2` syscall 入口，支撑批次 B
   中的 vector positioned I/O case。
2. RV/LA 同步追加批次 B 子集：`fcntl*`、`pipe*`、`pipe2_*`、`writev*`、
   `preadv*`、`pwritev*`、`pwrite02*`、`poll01/02`、`pselect*`、`select01-04`。
3. 继续排除 `readv02`、eventfd、flock、stat/rename/sendfile、iozone 和网络组。
4. 下一次评测若 LTP 明显上涨，继续向 C/D 批推进；若 LTP 不涨但不回退，则看
   串口日志拆 poll/select 或 fcntl；若总分回退，优先回滚本轮 B 子集而不是动
   basic/BusyBox/Lua/libctest。

## 2026-06-29 批次 C 子集

在批次 B 基础上继续推进进程/信号/时间类 LTP，但不打开 timerfd：

1. 修复 RISC-V `nanosleep(req, rem)`：syscall 表传入第二参数，校验
   `tv_nsec < 1e9`，被信号打断时可安全写回 `rem`。
2. RV/LA 同步追加 `alarm03/05/06/07`、`nanosleep01/02/04`、
   `kill02/03/05-13`、`waitpid01/03/04/06-13`、`fork04/09/13/14`。
3. `timerfd_create/gettime/settime` 继续暂缓，原因是 RV 侧尚无 timerfd fd
   对象；贸然加入只会增加确定失败项。
4. 若这批导致 LTP 截断，优先拆回 `kill*` 和 `alarm*`，保留
   `nanosleep/waitpid/fork` 观察。

## 838 回退后的主线

`301e9717 feat: batch LoongArch big test probes` 的官方结果内嵌 JSON 显示
`838.5995587579628`。页面的 `JSON格式错误` 来自平台脚本在 JSON 前输出
`gzip: sdcard-rv.img already exists; not overwritten` 和线程异常，不代表内核
没有得分，也不是 Compile Error。

真正需要修的是 LA 侧运行回退：basic 四列仍满分，但 LA BusyBox、Lua、
libcbench、libctest 和 LTP 被 `busybox-musl` 阶段的 panic 截断。日志显示
`busybox du` 从 `/musl` 递归扫入 `/musl/ltp/testcases` 大树，随后触发
LoongArch `Unhandled PLV0 Page Fault`。

本轮路线先撤回激进大分区批量入口，恢复 983 基线；同时让 LA BusyBox 在
`/tmp/swtc-busybox-*` 沙箱目录中执行，只放 `busybox`、脚本、命令清单和
`ls` 链接，避免 `du/find` 扫整个 `/musl` 或 `/glibc`。

为了不是只做十几分小步，本轮保留一个后置大分区探针：`lmbench` 使用官方命令
序列，但放在 basic、BusyBox、Lua、libcbench、libctest、LTP 之后，并且每条
命令单独 timeout。这样优先保护已知 983 functional 路径，同时给 lmbench 留出
一次大分增量机会。`iozone`、`cyclictest`、`iperf`、`netperf` 仍暂不启用，因为
它们目前没有线上增益证据，且曾造成 320 或 838 回退。

下一次评测首要目标是恢复到 `983` 以上，再观察 LA libctest/LTP 和后置
lmbench 是否开始计分。

## 983 分后的主线（保留但暂缓）

2026-06-28 08:12:33 提交的官方结果为 `Accepted / 983.2675500541894`：
basic 四列满分 `408`，BusyBox 合计 `208`，Lua 合计 `35`，libcbench 合计
约 `86.91`，musl-rv libctest `217`，musl-rv LTP `155`。真实 LoongArch
已线上生效，当前最快增长点仍是 LA functional，但必须先确保 BusyBox 不再
panic，暂不继续启用大分区批量入口。

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
| 最新可见官方结果 | 2026-06-28 16:47:31 内嵌 JSON，`838.5995587579628`；页面 JSON 错误由平台 gzip 异常污染输出导致 |
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
| 当前已知边界 | iozone/cyclictest/network 入口会拖低稳定组；BusyBox 必须沙箱运行；lmbench 只能放在 functional 之后逐命令限时探测 |

## 2026-06-24 快速增长四路线

1. **musl dynamic libctest**：static `107/107`、dynamic `110/110` 已线上拿满
   217 分，当前冻结。
2. **LTP 批量队列**：首批线上 44 分；第二批 13 个 case、26 个 TPASS 已本地
   双跑通过，等待线上确认，不恢复会 panic 的宽泛 256 项队列。
3. **LoongArch**：已完成第一阶段。真实 `kernel-la` 已离线构建，官方 LA 镜像
   本地 musl/glibc basic 64/64 通过；下一阶段扩展 BusyBox、Lua、libctest 和 LTP。
4. **性能测试**：在 1 GiB 物理帧范围修复后重新评估 libcbench/lmbench，再逐项
   推进 cyclictest、iozone、iperf/netperf；不得恢复曾导致 320 回退的全局 staging。

第一路线的完整官方镜像运行已经证明此前后半程归零的关键原因之一是 submit
模式仅管理 128 MiB 物理内存。当前已改为匹配官方 `-m 1G`，并通过 256 MiB
无盘兼容回归。新一轮线上结果出来前，以 604.3224084239476 作为线上稳定基线。

2026-06-24 已确认 `d500180` 编译通过但线上回退到 320：基础组仍在，
libcbench/libctest 全部丢分，cyclictest 也没有得分。当前已完整撤回该提交；
恢复 480 基线前不再扩大测试组或改全局 VFS 环境。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. 默认 `make all` 必须生成 `kernel-la`；若 LA 工具链不可用，允许占位 ELF。
   真实 LoongArch ELF 与官方 LA 镜像 basic 64/64 只作为 `build-la-strict` 门禁。
5. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
6. 官方完整参数下，无盘和第二批 LTP 探针均无 panic、无全局超时并主动关机。
7. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试；
   lmbench staging 必须提供 `/lmbench_all` 根路径别名。
8. submit 默认构建 feature 应为 `submit tmpfs`，不再带 `stack_trace`；需要诊断时手动传 `STACK_TRACE=1`。
9. fake lmbench EXT4 盘应暂存 2 组共 18 条命令并主动关机。
10. musl libctest 的 107 个 static 和 110 个 dynamic case 已线上通过，后续
    不得改动其顺序和隔离运行时。
11. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `100/100`。
- RISC-V Lua 保持 `18/18`。
- libcbench 恢复约 `57` 分区间。
- `libctest-musl` 保持 `217/217`。
- LTP 保持线上 70 分，并观察第三批 9 项的 91 个 TPASS 能否转化为增量。
- 总分目标为 1100–1400，最低不得低于当前 607；若回退，先核对提交哈希和
  完整串口日志，不再盲目扩大 staging。
- cyclictest 探针已证明会造成 320 回退，不再启用。
- 不再暂存或执行 iozone；完整脚本和 lite 探针都已证明会导致 320 回退。
- 不再出现 `src/process/thread/exit.rs:74` panic。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后用 `A` 或 `C` 记录执行最短命令，确认有 START/END、timeout 生效且主动关机后
   扩展完整脚本。
4. 若出现回退，立即撤回新增 case，保留 604 基线。

## 后续提分顺序

1. 提交第二批 13 个 musl LTP case，目标从约 604 提升到约 630。
2. 线上确认 LTP 达到约 70 分后冻结当前 34 个有效 case，再本地筛选 pipe、
   readv/writev 和目录变更测试。
3. 本地单独探测 libctest 的 `dynamic crypt`、`static crypt`、`static pleval`，
   只有不增加全局运行时且双跑通过时才提交。
4. 若 lmbench 仍为 0，必须看串口日志确认是否出现 `Simple syscall:`、
   `Select on 100 fd` 或 `Signal handler installation:`；没有这些行就继续修
   执行/超时路径，有这些行但不计分再查 parser 分组。
5. iozone 暂停；没有完整官方串口日志前，不再做任何 iozone staging。
6. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 iozone、iperf 或 netperf 执行组；LTP 不超过本轮已验证的 13 个新增项。
- 不处理网络、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
