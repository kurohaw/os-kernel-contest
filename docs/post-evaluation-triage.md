# 评测结果出来后的分叉处理清单

当前评测排队时间很长，结果出来后不能只看总分。按本文先提取关键信息，再决定
下一轮代码是否启用 LTP 大批量、修 LA libctest，或收窄 lmbench。

## 必须记录的分数列

结果页面或 JSON 中先记录这些字段：

```text
basic-glibc-rv basic-glibc-la basic-musl-rv basic-musl-la
busybox-glibc-rv busybox-glibc-la busybox-musl-rv busybox-musl-la
lua-glibc-rv lua-glibc-la lua-musl-rv lua-musl-la
libcbench-glibc-rv libcbench-glibc-la libcbench-musl-rv libcbench-musl-la
libctest-musl-rv libctest-musl-la
ltp-musl-rv ltp-musl-la
lmbench-glibc-rv lmbench-glibc-la lmbench-musl-rv lmbench-musl-la
```

同时保存完整串口日志，至少搜索：

```text
panic
Panicked
Unhandled PLV0
OS COMP TEST GROUP START
OS COMP TEST GROUP END
RUN LTP CASE
FAIL LTP CASE
lmbench
libctest
```

## 分叉 1：总分低于 900

含义：本轮 `442e9ba1` 没有成功止血，仍有截断或 panic。

优先排查：

1. 是否仍在 `busybox-musl` 中 panic。
2. `busybox-musl` 是否已经进入 `/tmp/swtc-busybox-musl`，还是仍在 `/musl`。
3. 是否因为后置 lmbench 导致最终没有主动关机。

动作：

- 不扩 LTP；
- 不修 iozone/network；
- 先撤掉或收窄后置 lmbench；
- 若仍是 BusyBox `du/find` 扫大目录，则继续强化 BusyBox 沙箱。

## 分叉 2：总分 900-1100

含义：983 基线基本恢复，但新增大分还没有明显吃到。

下一步：

1. 保留当前 BusyBox 沙箱。
2. 若 LA libctest 仍为 0，优先修 LA libctest。
3. 启用 `docs/ltp-next-candidates.md` 中批次 A 的前 40-80 个，作为下一轮大分尝试。

不要做：

- 不同时打开 iozone/network/cyclictest；
- 不扩大 lmbench 参数；
- 不一次加入所有 LTP 候选。

## 分叉 3：总分 1100-1400

含义：LA functional 或后置 lmbench 已产生部分增量。

下一步：

1. 固化成功项，不改动已经计分的 START/END 和执行顺序。
2. 若 LA libctest 非零但未到 217，优先补完整 static/dynamic。
3. LTP 扩到批次 A 全量，并从批次 B 中选 pipe/writev/poll 子集。
4. lmbench 若非零，开始按输出缺口修对应命令。

目标：下一轮至少 +300。

## 分叉 4：总分超过 1400

含义：当前架构已经突破 983 基线，进入真正大分区阶段。

下一步：

1. LTP 继续扩容到 A/B/C 批次组合，但每次必须本地日志过 `analyze_ltp_log.py`。
2. lmbench 开始做专项修复，目标从非零推到 150+。
3. iozone 只开最小单项探针，并放在所有 functional/LTP/lmbench 后。
4. network/cyclictest 继续等待，不和 iozone 同轮首次启用。

## LA libctest 专项判断

如果 `libctest-musl-la=0`：

- 检查是否出现 `#### OS COMP TEST GROUP START libctest-musl ####`。
- 若没有 START，说明 init 路径或文件位置错误。
- 若有 START 但没有 case 输出，检查 `runtest.exe`、`entry-static.exe`、
  `entry-dynamic.exe` 是否存在于当前目录。
- 若 static 有分 dynamic 无分，优先检查动态 loader 和 `/lib` 链接。
- 若某个 case 卡住，必须单项 timeout 后继续，不允许整组归零。

预期收益：

```text
static 107
dynamic 110
total 217
```

## lmbench 专项判断

如果四列 lmbench 仍为 0：

| 日志现象 | 下一步 |
|---|---|
| 没有 START | 修 init 执行顺序或路径 |
| 有 START，无有效表格 | 输出格式贴回官方脚本，减少自定义文字 |
| 卡 `lat_proc shell` | 检查 `/bin/sh`、`hello`、`/tmp/hello`、fork/exec/wait |
| 卡 `lat_ctx` | 缩小参数或单项隔离，查 pipe/scheduler |
| 卡 mmap/pagefault | 查文件 mmap、MAP_SHARED、fault handler |
| 能跑但分低 | 按第一名分布逐项补，而不是一口气全开 |

如果 lmbench 导致主动关机失败：

- 先只保留 `lat_syscall null/read/write/stat/fstat/open` 六项；
- 其余命令全部后移；
- 等六项非零后再恢复 pipe/proc/mmap/ctx。

## LTP 下一轮启用规则

使用 `docs/ltp-next-candidates.md`：

1. `< 900`：不启用。
2. `900-1100`：启用批次 A 的前 40-80 个。
3. `1100-1400`：批次 A 全量 + 批次 B 子集。
4. `> 1400`：A/B/C 本地筛过后合并。

每次启用后必须确认：

- RV basic/BusyBox/Lua/libctest 没有回退；
- `ltp-musl-rv` 不低于当前 `155`；
- LA basic/BusyBox 不因 LTP 扩容被截断；
- 日志最终有 `#### OS COMP TEST GROUP END ltp-musl ####` 和主动关机。
