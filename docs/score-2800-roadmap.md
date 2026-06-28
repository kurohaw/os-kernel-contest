# 2800 分冲刺路线

本文用于等待 2026-06-28 当前评测结果期间统一后续提分方向。目标不是每轮
十几分微调，而是按大分区推进，争取每次有效提交至少带来 200 分级别的增量。

配套执行文档：

- [`post-evaluation-triage.md`](post-evaluation-triage.md)：当前评测结果出来后先按
  分数区间和串口日志分叉，避免盲目继续扩大测试。
- [`ltp-next-candidates.md`](ltp-next-candidates.md)：下一轮 LTP 大批量候选池，
  仅在 983 基线恢复后按批次启用。

## 第一名分布参考

用户提供的第一名截图显示，总分约 `2877.9425`。按当前表格列顺序拆分后，
关键结构如下：

| 测点 | 第一名得分特征 | 我们当前稳定基线 | 主要缺口 |
|---|---:|---:|---:|
| basic | 四列全 `102`，合计 `408` | `408` | 已满，不再投入 |
| busybox | 四列约 `54`，合计 `216` | `208` | 小缺口，低优先级 |
| lua | 四列约 `9`，合计 `36` | `35` | 小缺口，低优先级 |
| libcbench | 四列约 `54`，合计 `216` | 约 `87` | 中等缺口，需补 LA 与稳定性 |
| libctest | RV/LA 均 `217`，合计 `434` | `217` | 高价值，补 LA 可直接 +217 |
| lmbench | 四列约 `72`，合计 `288` | `0` | 高价值，需专门修运行环境 |
| ltp | RV/LA 均约 `30525` 原始，折算约 `1000` | `155` | 最大缺口，主攻方向 |
| iozone | 四列约 `40`，合计 `160` | `0` | 高价值但曾回退，后置推进 |
| iperf | 四列约 `8`，合计 `32` | `0` | 网络路径，后置推进 |
| netperf | 四列约 `12`，合计 `48` | `0` | 网络路径，后置推进 |
| cyclictest | 四列约 `8`，合计 `32` | `0` | 小中收益，需防止回退 |

结论：`basic/busybox/lua` 已不是瓶颈。要接近 2800，必须吃下
`LTP + libctest + lmbench + iozone/network`，其中 LTP 是最大单点。

## 当前等待结果的判定

当前已提交 `442e9ba1 fix: isolate LoongArch BusyBox before lmbench`。该提交
修复上一轮 `838` 的已知硬伤：LoongArch `busybox-musl du` 从 `/musl` 扫入
`/musl/ltp/testcases` 大树后触发 page fault，导致后续 LA 组归零。

评测结果出来后按以下分叉处理：

| 结果 | 判断 | 下一步 |
|---|---|---|
| `< 900` | 仍有截断或 panic | 先看完整串口日志，撤掉后置 lmbench 或继续收窄 LA BusyBox 沙箱 |
| `900-1100` | 983 基线基本恢复 | 进入 LTP + LA libctest 大批量路线 |
| `1100-1400` | LA functional 有新增分 | 冻结当前 functional，扩大 LTP，开始修 lmbench |
| `> 1400` | 后置 lmbench 或 LA libctest 已起效 | 立即把本轮成功项固化，再开 iozone/network 的隔离探针 |

不要只看总分。必须同时记录每个测点四列明细，尤其是：

- LA BusyBox 是否恢复到约 `108`；
- LA Lua/libcbench 是否不再归零；
- LA libctest 是否从 `0` 开始计分；
- LA/RV lmbench 是否出现任意非零；
- LTP 是否保持 RV `155`，并观察 LA 是否开始计分。

## 四条最快增长主线

### 1. LTP 大批量推进

目标：从当前 `155` 提升到 `400-600`，最终冲 `800+`。

执行方法：

1. 不再一轮只加十几个 case；本地先筛 80-150 个候选。
2. 每个候选必须满足：有 `TPASS`、无 `TFAIL/TBROK`、退出状态为 0、无 timeout、
   无 kernel panic。
3. 按类型分批，而不是混合提交：
   - 文件/目录：`openat`、`mkdir`、`rmdir`、`rename`、`statx`、`truncate`；
   - fd/io：`pipe`、`readv/writev`、`preadv/pwritev`、`fcntl`、`flock`；
   - 进程/信号：`waitpid`、`kill`、`alarm`、`nanosleep`、`timerfd`；
   - 时间/系统信息：`gettimeofday`、`clock_gettime`、`uname`、`getrandom`。
4. 每批目标不是“全过”，而是至少转化 150-300 分增量。

风险控制：

- `readv02` 已知会触发 panic，继续排除。
- 所有 LTP 仍保持单项 timeout，不允许一个 case 卡住整轮。
- 新 case 必须放在已验证稳定 case 后面，便于从日志定位断点。

### 2. LoongArch libctest 补齐

目标：从 LA `0` 拉到 `217`，这是最清晰的一块整分。

执行方法：

1. 如果本轮 LA libctest 仍是 `0`，下一轮只围绕 LA libctest 修：
   `runtest.exe`、`entry-static.exe`、`entry-dynamic.exe`、cwd、动态库路径、
   timeout 和 marker。
2. 先拿 static，再拿 dynamic；不要同时混入 iozone/network。
3. 输出必须保留官方 `#### OS COMP TEST GROUP START libctest-musl ####`
   和 `END`，每 case 失败不能截断后续 case。

预期收益：

- static 约 `107`；
- dynamic 约 `110`；
- 合计可补 `217`。

### 3. lmbench 专项修复

目标：从 `0` 到 `150-288`。

当前策略：

- `442e9ba1` 已将 lmbench 放在所有 functional 组之后；
- 每条命令单独 timeout；
- 不允许 lmbench 失败影响 basic/BusyBox/libctest/LTP。

下一步根据结果修：

| 日志现象 | 修复方向 |
|---|---|
| 没有 `lmbench-* START` | init 顺序或文件路径错误 |
| 有 START 但全 0 | 输出格式不匹配，需贴近官方脚本原文 |
| 卡在 `lat_proc shell` | `/bin/sh`、`hello`、`/tmp`、fork/exec/wait |
| 卡在 `lat_ctx` | 调度/pipe/进程回收，缩小参数或单独修 |
| 卡在 mmap/pagefault | `mmap/munmap`、文件页 fault、VFS 权限 |

### 4. iozone/network/cyclictest 后置冲刺

目标：合计补 `160 + 80 + 32` 左右。

执行顺序：

1. iozone 只用最小参数单项探针，放在全部 functional/LTP/lmbench 后面。
2. iperf/netperf 先验证 loopback server/client 能完整退出，再接官方 marker。
3. cyclictest 只在 LTP/lmbench 稳定后重开，禁止恢复曾导致 `320` 回退的旧入口。

这些组的共同规则：

- 每条命令必须 timeout；
- server 类进程必须在组结束前 kill；
- 任何一组出现回退，立即撤回该组，不影响前三条主线。

## 三轮评测目标

| 轮次 | 主要提交内容 | 目标总分 | 判断标准 |
|---|---|---:|---|
| 当前等待 | BusyBox 沙箱 + 后置 lmbench | `983-1400` | 修复 838 回退，不再 panic |
| 下一轮 | LTP 大批量 + LA libctest | `1400-1800` | 至少 +300，低于 1200 视为失败 |
| 再下一轮 | lmbench 专项 + iozone 最小探针 | `1900-2300` | lmbench 或 iozone 至少一组非零 |
| 冲刺轮 | LTP 扩到 800+，补 network/cyclictest | `2600-2800+` | 大分区覆盖接近第一名 |

## 操作纪律

- 每轮只允许一个高风险大分区进入正式提交，其余高风险组必须后置。
- 不因为一个小 case 得分就马上提交；攒成 200 分级别再评测。
- 每次评测结果必须保存完整日志，不只截总表。
- 若出现 `320` 或 `838` 型回退，优先看第一个 panic 或最后一个 START marker。
- `basic=408`、BusyBox/Lua 现有分数是保底盘，不为小收益冒险。
- 不再把 `iozone/cyclictest/network` 和 LTP 大扩容混在同一轮首次提交。
