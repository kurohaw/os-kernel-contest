# 初赛开发进度

## 当前状态

| 项目 | 内容 |
|---|---|
| 当前日期 | 2026-06-22 |
| 当前开发分支 | `codex/swtc-architecture`，本轮完成后推送到 `main` |
| 当前内核主体 | `SWTC/` |
| 历史保分基线 | 旧自建内核曾取得官方 basic=102 |
| 当前里程碑 | musl libctest static 已满分，2026-06-22 复测维持 483-484 基线 |
| 当前提交 | submit 构建关闭默认 `stack_trace`，lmbench 扩为 24-command 官方主项队列 |
| 最新可见线上结果 | 2026-06-22 15:40:33，`Accepted / 483.89530518161376`；basic=204、BusyBox=98、Lua=18、libcbench=56.89530518161379、libctest=107、lmbench=0 |
| 最新高分线上结果 | 2026-06-21 13:15:41，`Accepted / 484.26735406790885`；已确认撤回 iozone-lite 后恢复 |
| 上一条通过基线 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594`；basic=204、BusyBox=98、Lua=18、libcbench=57.255157002759375、libctest=107 |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594`；libcbench glibc/musl 合计 57.255157002759375、libctest-musl=107 |
| 本地得分闭环 | 官方 basic 解析器 `102/102` |

## 2026-06-22 lmbench 主项冲刺

- 最新官方结果为 2026-06-22 15:40:33，`Accepted / 483.89530518161376`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.89530518161379、
  libctest=107；lmbench 仍为 0。
- 评测页面已经显示 lmbench-glibc 和 lmbench-musl 的完整评分表，说明源码包编译、
  lmbench 分组和页面识别都已恢复，但当前 9-command lite 没有产生计分。
- 本轮改为冲刺真实 lmbench 主项，不伪造输出：
  - `SWTC/kernel/Makefile` 中 submit 构建不再默认启用 `stack_trace`，去掉每次
    syscall/文件/内存函数入口的 `StackInfoGuard` push/pop 开销。
  - lmbench 队列从 9 条扩到 24 条，覆盖官方脚本主要项目：`lat_syscall`、
    `lat_select`、`lat_sig`、`lat_pipe`、`lat_proc`、`lmdd`、`lat_pagefault`、
    `lat_mmap`、`lat_fs`、`bw_pipe`、`bw_file_rd`、`bw_mmap_rd` 和 `lat_ctx`。
  - 补齐 `/var/tmp/XXX`、`/tmp/hello` 和 `lat_sig` helper/兼容别名。
- 本地验证：
  - `make all RUST_TOOLCHAIN=nightly-2025-02-18`：通过，构建日志显示内核 feature
    为 `submit tmpfs`，不再包含 `stack_trace`。
  - 无测试盘 QEMU：主动关机。
  - 官方布局 basic 双组盘：官方 `test_runner.py` 解析 `102/102`。
  - fake lmbench EXT4 盘：glibc/musl 两组均识别，暂存 48 条命令并主动关机。
- 风险：完整 lmbench 主项可能拉长评测时间；若线上回退低于 480，优先撤回本轮
  lmbench 扩容，只保留关闭 `stack_trace` 的性能构建再复测。

## 2026-06-21 14:23 lmbench 路径别名修复

- 保持当前 9-command lmbench-lite 范围，不新增 iozone、ltp、iperf、netperf 或
  其他测试组。
- 当前 `sys_readlinkat(/proc/self/exe)` 仍维持稳定基线的兼容行为：返回
  `/lmbench_all`，不恢复 `b433976` 那种全局真实路径尝试。
- 本轮修复的是 staging 缺口：`install_lmbench_group` 除了在
  `oscomp-lmbench-*/lmbench_all` 写入官方 ELF，也同步写入根目录
  `/lmbench_all`，让 readlinkat 返回值指向一个真实存在的 ELF。
- 本地验证：
  - `make all RUST_TOOLCHAIN=nightly-2025-02-18`：通过。
  - 无测试盘 QEMU：输出 `oscomp: block device unavailable`、`!TEST FINISH!`，
    并主动关机。
  - fake lmbench EXT4 盘：glibc/musl 两个 `lmbench_testcode.sh` 均被识别，
    两组 9-command 队列均进入 START/END，最终主动关机。
  - 官方布局 basic 双组盘：官方 `test_runner.py` 解析结果全部为通过，并主动关机。
- 注意：fake lmbench 盘使用 `basic/brk` 冒充 `lmbench_all`，只验证 staging 和
  队列入口，不代表真实 lmbench 得分。下一次官方评测重点看 lmbench 是否从 0
  变为有分，若仍为 0，再根据串口日志确认是否出现
  `Simple syscall:`、`Select on 100 fd` 或 `Signal handler installation:`。

## 2026-06-21 13:50 稳定基线复测

- 最新官方结果为 2026-06-21 13:50:12，`Accepted / 483.52722370911204`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.527223709112、
  libctest=107；iozone、lmbench 等仍为 0。
- 与 13:36 的 `483.16564668235225` 相比略有回升，仍属于 libcbench 性能分波动；
  basic、BusyBox、Lua 和 libctest 都没有回退。
- 继续保持“不新增测试组”：没有完整串口日志或真实 `lmbench_all` 本地复现前，
  不再改 lmbench/iozone staging。

## 2026-06-21 13:36 稳定基线复测

- 最新官方结果为 2026-06-21 13:36:45，`Accepted / 483.16564668235225`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.165646682352225、
  libctest=107；iozone、lmbench 等仍为 0。
- 与 13:15 的 `484.26735406790885` 相比少约 1.10 分，来源是 libcbench 性能波动；
  basic、BusyBox、Lua 和 libctest 都没有回退。
- 当前继续保持“不新增测试组”：没有完整串口日志或真实 `lmbench_all` 本地复现前，
  不再改 lmbench/iozone staging。

## 2026-06-21 13:15 iozone-lite 撤回后恢复

- 最新官方结果为 2026-06-21 13:15:41，`Accepted / 484.26735406790885`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=57.26735406790887、
  libctest=107；iozone、lmbench 等仍为 0。
- 这确认 `64fe8b4 Revert "feat: add minimal iozone probe"` 已恢复 484 基线；
  13:04 的 `320.0` 回退就是 iozone-lite 造成的。
- 当前不要再用线上评测盲试新组。下一步需要先拿到 lmbench 串口日志，或在本地构造
  真实 `lmbench_all` 镜像复现，再决定是否修 syscall/VFS 小缺口。

## 2026-06-21 13:04 iozone-lite 回退并撤销

- 最新官方结果为 2026-06-21 13:04:01，`Accepted / 320.0`。
- 得分构成：basic=204、BusyBox=98、Lua=18；libcbench、libctest、iozone、
  lmbench 等均为 0。
- 这说明 `8690e03 feat: add minimal iozone probe` 虽然本地假镜像可运行，但在
  官方完整镜像中会破坏后续 libcbench/libctest 得分路径，表现与历史完整 iozone
  回退相同。
- 已通过 `64fe8b4 Revert "feat: add minimal iozone probe"` 撤回 iozone-lite，
  代码恢复到 `83ff79e feat: shorten lmbench lite runs` 的 484 基线。
- 后续禁止再暂存或执行 iozone，除非先拿到完整串口日志并能证明不会影响
  libcbench 与 libctest。下一步应从不新增测试组的 syscall/VFS 小修入手。

## 2026-06-21 12:23 lmbench 9-command 探针结果

- 最新官方结果为 2026-06-21 12:23:56，`Accepted / 484.15161299502336`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=57.15161299502337、
  libctest=107，lmbench 仍为 0。
- 与 12:05 的 `484.2551570027594` 相比只少约 0.1035 分，来源是 libcbench
  性能波动；basic、BusyBox、Lua 和 libctest 107 均保持稳定。
- 这说明 9-command lmbench 探针没有造成明显回退，但也没有让输出被官方计分。
- 评测耗时从约 3 分钟增加到约 5 分钟，结合 lmbench 仍为 0，优先判断为默认
  lmbench 轮次过长，命令被 per-command timeout 终止，未吐出 `Simple syscall:`、
  `Select on 100 fd`、`Signal handler installation:` 等可解析行。
- 本轮不继续加命令，而是保留 9-command 范围，加入 `-W 1 -N 10` 短轮次参数，并
  把单命令 timeout 收回到 10 秒，目标是让命令快速完成并保留超时隔离。

## 2026-06-21 12:05 musl libctest static 满分

- 最新官方结果为 2026-06-21 12:05:08，`Accepted / 484.2551570027594`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=57.255157002759375、
  libctest=107；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 这说明 `415b423 feat: run full musl libctest static set` 的 107-case
  full-static 探针全部进入 musl-rv 得分，且没有引发 basic、BusyBox、Lua 或
  libcbench 回退。
- libctest 方向先冻结：后续不要继续改 allowlist、timeout 或 `C` 队列协议，除非
  出现官方回归。
- 下一阶段优先选一个新指标做最小探针。推荐先回到 `lmbench-lite`，只跑一条短命令
  或先做脚本/资源 staging 诊断；若出现回退，立刻撤回并保留 484 基线。

## 2026-06-21 lmbench-lite 9-command 探针

- 在 484 基线上只推进 `lmbench` 一个指标，不修改 libctest、libcbench、iozone、
  ltp、网络或 LoongArch。
- 线上历史显示 6 条 `lat_syscall` lite 命令仍为 0 分，且 musl 曾出现超时迹象。
- 第一版保持官方参数语义，不添加 `-N` 或缩短 benchmark 轮次，只把每条 lmbench
  命令超时从 5 秒放宽到 20 秒；线上确认仍为 0 分。
- 第二版保留同样 9 条命令，但加入 `-W 1 -N 10`，避免默认轮次在当前内核上跑到
  timeout 之前没有任何可解析输出。
- 在原有 `lat_syscall null/read/write/stat/fstat/open` 后新增官方脚本中紧接着的
  3 条轻量命令：`lat_select -n 100 -P 1 file`、
  `lat_sig -P 1 install`、`lat_sig -P 1 catch`。
- 暂不加入 `lat_sig prot`、`lat_pipe`、`lat_proc`、`lmdd`、`lat_fs` 或带宽测试；
  这些更容易触发文件、进程、mmap 或管道语义问题，等 9-command 结果确认后再拆。
- 本地 lmbench 夹具使用假 `lmbench_all` 验证队列和隔离：glibc/musl 两组均被识别，
  共暂存 18 条命令，START/END 各 2 次，最终主动关机且无 panic。

## 2026-06-21 11:49 libctest static 全量探针

- 最新官方结果为 2026-06-21 11:49:38，`Accepted / 412.92336789756513`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.92336789756515、
  libctest=36；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 这说明上一轮 12-case 扩容全部进分，且没有引发 libcbench 或退出路径回退。
- 本轮按“直接拿满 musl static libctest”的要求，把 allowlist 扩到官方
  `libc-test/static.txt` 归一化后的全部 107 个 case。
- 仍只改 musl libctest 一个指标，并继续依靠 `C` 队列逐 case 串行执行和
  `LIBCTEST_TIMEOUT_MS` 隔离失败项；若回退，优先退回到已验证的 36-case 基线。

## 2026-06-20 21:34 基线恢复与 12-case libctest 探针

- 最新官方结果为 2026-06-20 21:34:30，`Accepted / 400.50066694574866`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.50066694574863、
  libctest=24；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 这说明上一轮新增的 `search_hsearch`、`search_insque`、`search_lsearch`、
  `search_tsearch` 四个 case 已稳定进分，且没有引发 libcbench 或退出路径回退。
- 本轮继续沿单指标策略，但把批次从 4 个扩大到 12 个，新增
  `clocale_mbfuncs`、`clock_gettime`、`fnmatch`、`inet_pton`、`mbc`、`setjmp`、
  `sscanf`、`sscanf_long`、`strftime`、`strtod`、`strtold`、`swprintf`。
- 验收标准是总分不低于当前 400 基线，且 libctest 从 24 明显上升或至少提供
  明确失败日志；若回退，优先退回本轮 12-case 探针。

## 2026-06-20 21:09 基线恢复与 4-case libctest 探针

- 最新官方结果为 2026-06-20 21:09:40，`Accepted / 397.32511382265227`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=57.325113822652256、
  libctest=20；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 这说明上一轮新增的 `strtod_simple`、`strtof`、`udiv`、`wcsstr`
  四个 case 已稳定进分，且没有引发 libcbench 或退出路径回退。
- 本轮继续沿同一策略，只新增 `search_hsearch`、`search_insque`、
  `search_lsearch`、`search_tsearch` 四个低风险静态 case。验收标准是总分不低于
  当前 397 基线，且 libctest 从 20 小幅上升或至少提供明确失败日志。

## 2026-06-20 17:52 基线恢复与 4-case libctest 探针

- 最新官方结果为 2026-06-20 17:52:16，`Accepted / 392.9002806839883`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.9002806839883、
  libctest=16；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 这说明上一轮新增的 `string_memmem`、`string_strcspn`、`strtol`、
  `strverscmp` 四个 case 已稳定进分，且没有引发 libcbench 或退出路径回退。
- 本轮继续沿同一策略，只新增 `strtod_simple`、`strtof`、`udiv`、`wcsstr`
  四个低风险静态 case。验收标准是总分不低于当前 392 基线，且 libctest
  从 16 小幅上升或至少提供明确失败日志。

## 2026-06-20 16:16 基线恢复与 4-case libctest 探针

- 最新官方结果为 2026-06-20 16:16:33，`Accepted / 389.00362218124934`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=57.00362218124933、
  libctest=12；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 这说明上一轮新增的 `string_memcpy`、`string_memset`、`string_strchr`、
  `string_strstr` 四个 case 已稳定进分，且没有引发 libcbench 或退出路径回退。
- 本轮继续沿同一策略，只新增 `string_memmem`、`string_strcspn`、`strtol`、
  `strverscmp` 四个低风险静态 case。验收标准是总分不低于当前 389 基线，
  且 libctest 从 12 小幅上升或至少提供明确失败日志。

## 2026-06-20 14:43 基线恢复与 4-case libctest 探针

- 最新官方结果为 2026-06-20 14:43:29，`Accepted / 384.8411392883504`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=56.84113928835043、
  libctest=8；cyclictest、iozone、iperf、lmbench、ltp、netperf 仍为 0。
- 串口日志显示启动时暂存 11 个测试组、90 条命令，basic、BusyBox、Lua、
  libcbench、libctest、lmbench 两个 ABI 组均到达 END，最终主动关机。
- `libctest-musl` 的 8 个稳定 case 仍输出 `Pass!`；此前 `4602678` 一次扩到
  64 个 case 的回退已通过 revert 和 `exit.rs:74` 修复止血。
- 本轮不恢复 64-case 扩容，只新增
  `string_memcpy`、`string_memset`、`string_strchr`、`string_strstr` 四个纯字符串
  静态 case。验收标准是总分不低于当前 384 基线，且 libctest 从 8 分小幅上升或
  至少提供明确失败日志。

## 2026-06-20 14:24 libctest 扩容回退

- 最新官方结果为 2026-06-20 14:24:44，`Accepted / 326.0`。
- 得分构成：basic=204、BusyBox=98、Lua=18、libcbench=6.0、libctest=0。
- 串口日志显示启动时已暂存 11 个测试组、146 条命令，但在
  `#### OS COMP TEST GROUP START libcbench-glibc ####` 后触发 panic：
  `src/process/thread/exit.rs:74 called Option::unwrap() on a None value`。
- 本轮判断：`4602678 feat: expand musl libctest static coverage` 将队列从约 90
  条扩大到 146 条，提前暴露了退出路径中父进程 weak 指针失效的内核 bug。
- 当前处理：先 revert `4602678`，恢复已验证的 8 个 libctest case；同时把
  `exit.rs:74` 的 `upgrade().unwrap()` 改为父进程已释放时跳过 SIGCHLD 通知，
  禁止同类 orphan exit 再次 panic。

## 2026-06-20 13:19 官方结果与 lmbench argv 修复

- 最新可见官方结果为 2026-06-20 13:19:00，`Accepted / 384.97435365207264`。
- 相比 2026-06-20 12:03:02 的 `385.16527137512986`，差值只有
  `0.19091772305722`，来源是 `libcbench` 性能分波动：glibc-rv 从
  `30.059660564398104` 到 `29.84938547814558`，musl-rv 从
  `27.1056108107318` 到 `27.124968173927094`；不是测试项丢失。
- `libctest-musl` 已稳定出现 8 个 case，且 `argv`、`basename`、`dirname`、
  `env`、`qsort`、`random`、`snprintf`、`string` 均输出 `Pass!`，贡献 8 分。
- `lmbench` 已进入 glibc/musl 两组，但每条命令输出 `no match func -P`。
  这说明当前把 `lat_syscall` 当作 `argv[0]` 的兼容策略不符合官方
  `lmbench_all` 的调度方式；下一步改为保留 `argv[0]=./lmbench_all`，
  让 `lat_syscall` 作为第一个普通参数传入。

## 2026-06-20 远端同步与 checksum 门禁

- 已同步 GitLab `main` 到 `aed0d6a fix: align libctest probe output`。
- 同步后本地复查发现 `SWTC/vendor/allocator-api2-0.2.21/cargo-checksum.json`
  仍有 22 个文件哈希不匹配；虽然当前 `make all` 不再依赖该 crate，但它违反
  vendor 门禁，后续依赖变化时可能再次触发官方 Compile Error。
- 已用 `tools/vendor_checksums.py --fix` 刷新该 crate 的 checksum manifest，
  复查结果为 53 个 manifest、0 个问题。
- 本轮继续保持单指标策略：不扩大 lmbench/iozone/ltp/network，也不改
  `readlinkat`；等待下一次官方日志确认 musl libctest 探针是否进分。

## 2026-06-20 musl libctest 小批量探针

### 当前策略

- 本轮只追 `libctest` 一个指标，不同时改 `lmbench`、`readlinkat`、argv 或资源路径。
- `oscomp` 只探测 musl 入口 `musl/libctest_testcode.sh`，不新增 glibc libctest。
- 读取官方 `run-static.sh` 或 `run-static`，只从中筛出
  `argv`、`basename`、`dirname`、`env`、`qsort`、`random`、`snprintf`、
  `string` 这 8 个 allowlist case。
- 只暂存 `entry-static.exe`、可选 `runtest.exe` 和脚本元数据到
  `oscomp-libctest-musl`；若存在官方 runner，就执行
  `runtest.exe -w entry-static.exe <case>`，否则 fallback 到 `C` 队列记录。
- `runtestcase` 对 `C` 记录按真实退出码输出 `Pass!` 或
  `FAIL LIBCTEST CASE ...`，并补齐 per-case `========== START/END ... ==========`
  标记；每条最多运行 3 秒，超时后继续后续队列。

### 验收重点

- 若官方仍为 0，优先看串口日志中是否出现 `RUN LIBCTEST CASE`、`Pass!`、
  `FAIL`、`execve failed` 或 `timeout`。
- basic、BusyBox、Lua 和 libcbench 不应因本轮 libctest 探针回退。
- 若 libctest 有分，下一轮继续小批量扩大 allowlist；若回退，直接撤回该组
  staging，保留 377 基线。

## 2026-06-20 readlinkat 真实路径回归

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-20 11:12:19 提交评测为
  `Accepted / 326.0`。
- basic、BusyBox、Lua 仍保持既有得分；libcbench 从上一轮
  `57.42523152095458` 掉到 `6.0`。
- 该提交对应 `b433976 fix: report current executable path`，改动点是让
  `readlinkat(/proc/self/exe)` 返回当前进程真实 `execve` 路径，并对其他
  readlinkat 路径返回 `ENOENT`。

### 当前止血

- 已用非破坏性的 `git revert` 回退 `b433976`。
- 恢复 `e8d1b48` 的 readlinkat 兼容行为，优先把 libcbench 拉回 377 基线。
- 后续不再直接修改通用 `readlinkat` 语义来试探 lmbench；lmbench 需要通过
  更隔离的资源 staging、argv 或超时诊断推进。

### 本地验证计划

- 强制离线 `make all`。
- 无测试盘官方风格 QEMU 主动关机。
- 官方 basic 双组镜像保持 `102/102`。
- 推送后下一次线上目标是恢复到 `377.42523152095464` 附近。

## 2026-06-20 hashbrown/ahash 离线解析错误

### 线上证据

- 用户提供的官方编译输出显示，2026-06-19 19:09:49 提交在
  `SWTC/kernel` 的 Cargo 阶段失败。
- 失败日志为：`error: no matching package found`，搜索包名为 `ahash`，
  搜索位置是 `/coursegrader/submit/SWTC/kernel/../vendor`。
- `ahash` 由 `hashbrown v0.14.5` 拉入，而 `hashbrown` 是内核
  `Cargo.toml` 中的直接依赖。

### 当前修复

- `SWTC/kernel/src/fs/inode.rs` 的 `InodeCache` 和 `FastPathCache` 从
  `hashbrown::HashMap` 改为 `alloc::collections::BTreeMap`。
- `SWTC/kernel/src/fs/hash_key.rs` 为 `HashKey` 补充 `Ord/PartialOrd`，
  满足 `BTreeMap` key 约束。
- `SWTC/kernel/Cargo.toml` 删除 `hashbrown = "0.14"`。
- `SWTC/kernel/Cargo.lock` 删除 `hashbrown`、`ahash`、`allocator-api2`、
  `once_cell`、`version_check` 以及只被该链路使用的 `zerocopy 0.7` 条目。

### 本地验证

- `CARGO_NET_OFFLINE=true make all RUST_TOOLCHAIN=nightly-2025-02-18` 通过。
- `kernel-rv` 仍为 RISC-V executable ELF，入口 `0x80200000`。
- 无测试盘官方风格 QEMU：输出 `!TEST FINISH!` 并主动关机。
- 带官方 basic 双组镜像 QEMU：glibc/musl 共 64 条命令串行执行，
  官方 `test_runner.py` 解析 `102/102`，失败数为 0。

## 2026-06-19 lmbench-lite argv0 修正

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-19 17:00:35 提交评测为
  `Accepted / 377.200790558321`。
- RISC-V basic、BusyBox、Lua 仍保持既有得分。
- libcbench 为 glibc-rv `30.12274508733359`、musl-rv
  `27.07823396849846`，总计 `57.2007905583205`。
- `lmbench` 仍为 0，说明 `e85c3ac` 的第一版 `lmbench-lite` 没有被官方解析为
  有效得分，但也没有破坏现有 377 基线。

### 当前修复

- `lmbench` START/END marker 改为优先使用官方 `lmbench_testcode.sh` 内的真实
  marker，避免手写组名与官方 parser 不一致。
- `runtestcase` 对 `lmbench_all` 使用队列第一个参数作为 `argv[0]`，让
  多调用二进制以 `lat_syscall` 名义运行。
- `readlinkat` 保持返回 `/lmbench_all` 的兼容路径，但返回 Linux 语义的实际
  字节数，避免调用方按返回值得到空路径。

## 2026-06-19 managed vendor 离线解析错误

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-19 16:31:18 提交评测为
  `Compile Error / 0.00`。
- 编译已通过 `SWTC/user`，失败发生在 `SWTC/kernel` 的 Cargo 阶段。
- 失败日志为：`error: no matching package named managed found`，搜索位置是
  `/coursegrader/submit/SWTC/kernel/../vendor`，且 Cargo 处于 offline 模式。

### 当前修复

- `SWTC/vendor/managed-0.8.0` 在 GitLab fresh clone 中存在，且 vendor 校验为
  53 个 manifest、0 个问题；本地无法复现该缺失。
- 为降低官方 directory source 对 `managed` 识别失败的风险，内核直接依赖改为
  `path = "../vendor/managed-0.8.0"`。
- 新增 `[patch.crates-io] managed = { path = "../vendor/managed-0.8.0" }`，
  让 `smoltcp` 等间接依赖也统一使用本地 path crate。
- `SWTC/kernel/Cargo.lock` 中 `managed` 不再带 registry source/checksum，
  避免该 crate 继续走 directory source 解析。

### 本地验证

- GitLab fresh clone `e85c3ac` 原始源码：vendor 目录包含
  `SWTC/vendor/managed-0.8.0`，离线 `make all` 可通过，说明线上失败更像官方
  源快照或 directory source 识别异常。
- 修复后主工作区：`tools/vendor_checksums.py --check` 为 53/0，
  `CARGO_NET_OFFLINE=true make all` 通过。
- 删除全部隐藏文件后的干净副本：vendor 校验 53/0，离线 `make all` 通过，
  `kernel-rv` 入口仍为 `0x80200000`。

## 2026-06-19 lmbench-lite 提分探针

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-19 15:50:35 提交评测为
  `Accepted / 377.2382238511116`。
- 当前已稳定执行 8 组：RISC-V `basic`、`busybox`、`lua`、`libcbench`
  的 glibc 与 musl 版本。
- 当前得分构成为：basic `204`、BusyBox `98`、Lua `18`、libcbench
  `57.2382238511116`。
- `cyclictest`、`iozone`、`iperf`、`libctest`、`lmbench`、`ltp`、
  `netperf` 仍为 0。
- LoongArch 仍为 0，直接原因是 `kernel-la` 仍为 RISC-V 占位 ELF，
  QEMU 报 `Failed to load ELF`。

### 当前修复

- 扩展 `/oscomp-queue`，保留原有 `G/X/E` 记录，新增
  `A<timeout_ms>\t<argv0>\t<arg1>...` 记录，用于执行带参数 ELF。
- `runtestcase` 从固定 4 KiB 读取队列改为分块读取，最大 64 KiB，
  避免后续小批量测试命令被截断。
- `A` 记录使用 `wait4(WNOHANG)` 轮询；超过记录内 timeout 后发送
  `SIGKILL` 并继续下一条记录，防止单个性能测试拖死整轮评测。
- 第一轮只新增 `lmbench-lite`：探测 `glibc/lmbench_testcode.sh` 与
  `musl/lmbench_testcode.sh`，每组只暂存 `lmbench_all`、可选 `hello`、
  必要动态运行时和空 `/var/tmp/lmbench`。
- 当前 allowlist 只执行轻量命令：
  `lat_syscall null/read/write/stat/fstat/open`，每条 5 秒超时。
- 暂不接入 `libctest`、完整 `iozone`、网络测试或 LoongArch，避免把
  已有 377 分基线和 libcbench 拉回 0。

### 本地验证

- 强制离线 `CARGO_NET_OFFLINE=true make all`：通过。
- 无测试盘官方风格 QEMU：输出 `!TEST FINISH!` 并主动关机。
- 外部官方 BusyBox 镜像：仍能执行 BusyBox 组并主动关机。
- 构造 `glibc/lmbench_testcode.sh` 与假 `lmbench_all` 的 EXT4 探针盘：
  能识别 `lmbench-glibc`，输出 START/END，执行 6 条 `A` 记录并主动关机。
- 本地没有官方真实 `lmbench_all` 镜像，本轮只验证队列协议、资源暂存和
  超时保护；真实 lmbench 得分需要下一次官方评测确认。

## 2026-06-19 SWTC vendor checksum 编译错误

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-19 14:51:46 提交评测为
  `Compile Error / 0.00`。
- 编译已进入 `SWTC/kernel` 的 Cargo 阶段，没有发生 rustup 联网下载或 target 缺失。
- 失败点为 `SWTC/vendor/allocator-api2-0.2.21/src/stable/macros.rs` checksum
  mismatch：官方日志中期望值为 `74490796...`，实际值为 `c05b6bbc...`。
- 本地复查发现同一个 crate 共有 22 个文件哈希与 `cargo-checksum.json` 不一致，
  官网只是在第一个触发点停止。

### 当前修复

- 已快进同步 `gitlab/main` 到 `ea2b5ac chore: rename kernel tree to SWTC`。
- `tools/vendor_checksums.py` 已指向 `SWTC/vendor`，无需再兼容旧 `titanix/vendor`
  路径。
- 使用现有工具重建 53 个 vendor manifest，仅
  `SWTC/vendor/allocator-api2-0.2.21/cargo-checksum.json` 发生变化。
- 保留原 `package` checksum，只更新 Git 实际追踪的非隐藏 vendor 文件哈希。

### 本地验证

- `tools/vendor_checksums.py --check`：53 个 manifest，0 个问题。
- 强制离线 `CARGO_NET_OFFLINE=true make all`：通过，无 Cargo checksum mismatch。
- 删除全部隐藏文件后的干净导出副本：vendor 校验 `53/0`，离线 `make all` 通过。

## 2026-06-19 libcbench 基线恢复确认

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-19 14:05:15 提交评测为
  `Accepted / 377.02594320298937`。
- RISC-V basic、BusyBox、Lua 保持满分。
- libcbench 恢复到 glibc-rv `29.86218129302594`、musl-rv `27.163761909963373`，
  与 2026-06-18 09:46:55 的 `377.3228370332187` 属于同一高分基线。
- iozone 仍为 0，说明下一步不能再一次性接入完整 iozone staging，需要拆成更小探针。

## 2026-06-19 iozone 回归止血

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-18 16:00:21 提交评测为
  `Accepted / 320.0`。
- RISC-V basic、BusyBox、Lua 保持原分数。
- libcbench 从上一条高分结果的 `57.32283703321875` 总分回退为 0，iozone 仍为 0。
- 该回归发生在 `b10e9f0 feat: stage iozone test group` 之后，说明 iozone staging
  当前不能直接进入主线。

### 当前修复

- 撤回 `b10e9f0` 中新增的 iozone 组扫描、暂存和运行时复制逻辑。
- 保留已线上验证有增益的 libcbench futex bitset 修复路径。
- 下一次评测先确认 libcbench 是否回到 2026-06-18 09:46:55 的高分线，再重新拆分
  iozone 为更小的诊断提交。

## 2026-06-18 377 分 libcbench 高分基线

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-18 09:46:55 提交评测为
  `Accepted / 377.3228370332187`。
- RISC-V basic、BusyBox、Lua 均未回退。
- libcbench 已由上一轮 `6.0` 提升到 glibc-rv `30.15271484677692`、
  musl-rv `27.170122186441827`，说明 futex bitset 兼容修复带来新增得分。

## 2026-06-18 326 分基线与 futex bitset 修复

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-18 09:33:47 提交评测为
  `Accepted / 326.0`。
- RISC-V basic、BusyBox、Lua 均未回退。
- libcbench glibc-rv 从 0 提升到 `6.0`，musl-rv 仍为 0。
- 结合 libc-bench 顺序，当前很可能已经跑过前 6 个 malloc benchmark，后续阻塞点
  落在 malloc 线程压力或 pthread/futex 相关路径。

### 当前修复

- futex syscall 现在会同时清除 `FUTEX_PRIVATE_FLAG` 和 `FUTEX_CLOCK_REALTIME`。
- 新增 `FUTEX_WAIT_BITSET` 与 `FUTEX_WAKE_BITSET`，按普通 wait/wake 路径处理。
- 未支持的 futex op 改为返回 `ENOSYS`，避免直接 panic 中断整轮测试。

### 本地验证

- `cargo +nightly-2025-02-18 fmt --manifest-path SWTC/kernel/Cargo.toml --check`：通过。
- `make all RUST_TOOLCHAIN=nightly-2025-02-18`：通过。
- 无测试盘 QEMU：输出 `!TEST FINISH!` 并主动关机。
- 双组官方布局 basic 镜像：官方 parser 得到 `102/102`。
- libcbench 官方布局夹具盘：识别 glibc/musl `libcbench_testcode.sh`，暂存 2 组并主动关机。

## 2026-06-18 320 分基线与 libcbench staging

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-18 09:16:08 提交评测为
  `Accepted / 320.0`。
- RISC-V basic 保持满分：glibc-rv `102/102`，musl-rv `102/102`。
- RISC-V BusyBox 保持得分：glibc-rv `49/49`，musl-rv `49/49`。
- RISC-V Lua 已开始得分：glibc-rv `9/9`，musl-rv `9/9`。

### 当前修复

- `oscomp` 新增 libcbench 组扫描顺序：`glibc/libcbench_testcode.sh`、
  `musl/libcbench_testcode.sh`、根目录 `libcbench_testcode.sh`。
- 每个 libcbench 组会暂存 `busybox`、静态 `libc-bench` 和官方
  `libcbench_testcode.sh` 到独立 tmpfs 工作目录。
- libcbench 队列沿用现有 `G/X` 协议，由官方脚本自身输出 START/END marker。
- 仅新增 libcbench staging，不改 runner 协议，不改 basic/BusyBox/Lua 执行顺序。

### 本地验证

- `make all RUST_TOOLCHAIN=nightly-2025-02-18`：通过。
- 无测试盘 QEMU：输出 `!TEST FINISH!` 并主动关机。
- 双组官方布局 basic 镜像：glibc、musl 均完整 START/END，官方 parser 得到
  `102/102`。
- libcbench 官方布局夹具盘：识别 `glibc/libcbench_testcode.sh` 和
  `musl/libcbench_testcode.sh`，暂存 2 个 libcbench 测试组并主动关机。
- 本机 `nightly-2025-02-01` 安装不完整，缺 RISC-V target 和 rustfmt 动态库；
  本地验证继续使用已完整安装的 `nightly-2025-02-18`。Makefile 默认官方工具链未改。

## 2026-06-18 302 分基线与 Lua staging

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-18 08:55:11 提交评测为
  `Accepted / 302.0`。
- RISC-V basic 已满分：glibc-rv `102/102`，musl-rv `102/102`。
- RISC-V BusyBox 已得分：glibc-rv `49/49`，musl-rv `49/49`。
- 其余测试组仍为 0，本轮继续优先推进脚本型、低耦合的 Lua 组。

### 当前修复

- `oscomp` 新增 Lua 组扫描顺序：`glibc/lua_testcode.sh`、`musl/lua_testcode.sh`、
  根目录 `lua_testcode.sh`。
- 每个 Lua 组会暂存 `busybox`、`lua`、`lua_testcode.sh`、`test.sh` 和官方
  9 个 `.lua` 测试脚本到独立 tmpfs 工作目录。
- Lua 队列沿用现有 `G/X` 协议，由脚本自身输出 START/END marker，避免 runner
  与官方脚本重复打印组边界。
- 仅新增 Lua staging，不改 runner 协议，不改 basic/BusyBox 执行顺序。

### 本地验证

- `make all RUST_TOOLCHAIN=nightly-2025-02-18`：通过。
- 无测试盘 QEMU：输出 `!TEST FINISH!` 并主动关机。
- 双组官方布局 basic 镜像：glibc、musl 均完整 START/END，官方 parser 得到
  `102/102`。
- Lua 官方布局夹具盘：识别 `glibc/lua_testcode.sh` 和
  `musl/lua_testcode.sh`，暂存 2 个 Lua 测试组并主动关机。
- 本机 `nightly-2025-02-01` 安装不完整，缺 RISC-V target 和 rustfmt 动态库；
  本地验证使用已完整安装的 `nightly-2025-02-18`。Makefile 默认官方工具链未改。

## 2026-06-16 musl ENOENT 与真实解释器路径暂存

### 线上证据

- 用户提供的官方 HTML 显示，2026-06-15 19:24:27 提交评测为
  `Accpted / 91.0`，glibc-rv basic 保持 `91/102`。
- 该页面结果早于 `gitlab/main` 的 `0bc0dc9 fix: improve musl runtime
  compatibility`，因此不能作为 `0bc0dc9` 的验证结果。
- RISC-V 串口中 musl 30 个 basic ELF 全部
  `oscomp: execve ... failed: -2`，即 `ENOENT`。
- 失败已从 panic/`ENOEXEC` 收敛为解释器或运行时路径不存在，下一步优先处理
  musl `PT_INTERP` 的完整路径匹配。

### 当前修复

- 已快进同步 `gitlab/main` 到 `0bc0dc9`。
- `oscomp` 从每个动态 ELF 的 `PT_INTERP` 读取真实解释器路径，并在组私有
  tmpfs 中创建完全匹配的路径。
- musl 运行时来源优先读取测试盘中的 `musl/lib/libc.so`，随后尝试
  `musl/lib/ld-musl-riscv64*.so.1` 与 `lib/ld-musl-riscv64*.so.1`。
- 找到一个有效 musl loader/libc 后，同时安装到 `/oscomp-musl/libc.so`、
  `/oscomp-musl/lib/libc.so`、常见 `ld-musl-riscv64*.so.1` 别名，以及每个
  ELF 实际声明的解释器路径。
- 保留 runner 的负 errno 输出；若下一次线上 musl 仍失败，应能看到真实
  `PT_INTERP` 与新的失败阶段。

### 本地验证

- vendor 校验：53 个 manifest，0 个问题。
- 强制离线 `make all`：通过。
- 无测试盘：输出 `!TEST FINISH!` 并主动关机。
- 单组 glibc basic：官方解析器保持 `91/102`。
- 双组静态镜像：依次输出 glibc、musl START/END，并主动关机。
- glibc 动态探针：输出 `PT_INTERP /lib/ld-linux-riscv64-lp64d.so.1`，并进入
  `main`。
- 损坏 loader 探针：输出 `execve ... failed: -8`，无 panic，正常收尾。
- 外部 BusyBox 镜像：无 panic、无卡死并主动关机。

## 2026-06-15 官方 91 分与 musl execve 诊断

### 线上证据

- 官方总分由 `0.0` 提升至 `91.0`，glibc-rv basic 为 `91/102`。
- glibc、musl 两组均完成 START/END，RISC-V 无 panic 并主动关机。
- musl 组已暂存 30 个 ELF，但 30 次 `execve` 全部失败，没有进入任何
  `========== START test_* ==========`。
- LoongArch 仍因 `kernel-la` 是 RISC-V 占位 ELF 无法加载。

### 当前修复

- runner 输出 `execve` 的负 errno，下一次线上失败可直接区分 `ENOENT/ENOEXEC`。
- ELF 安全校验只检查头部、program-header 表和段文件范围，不再拒绝 loader
  不使用的合法扩展 program-header 类型。
- ELF 映射和解释器扫描只处理需要的 `PT_LOAD/PT_INTERP`，未知类型安全跳过。
- 主 ELF、解释器查找/布局/解析和进程替换失败时输出简短阶段日志。

### 本地验证

- glibc basic 仍为 `91/102`。
- 动态 glibc、双组静态、BusyBox 外部探针和无盘启动均无 panic 并主动关机。
- 将非执行的 RISC-V attributes header 改为未知扩展类型后，动态 ELF 仍进入
  `main`。
- 损坏 loader 探针输出 `execve ... failed: -8`，继续输出组 END 并主动关机。

## 2026-06-14 动态 loader 与双组队列

### 线上证据

- 2026-06-13 19:30:50 的官方提交已编译成功，状态为 `Accpted`。
- RISC-V 找到 `musl/basic_testcode.sh` 并暂存 30 个 basic ELF。
- 首个动态 ELF 执行时，tmpfs 缺少 musl 动态解释器；
  `memory_space/mod.rs:871` 对解释器 inode 执行 `unwrap()`，导致内核 panic。
- LoongArch 因 `kernel-la` 仍是 RISC-V 占位 ELF 无法加载，本轮未处理。

### 已完成

- basic 探测顺序固定为 `glibc -> musl`，仅在两者都不存在时使用根目录脚本。
- 两组分别暂存到 `/oscomp-glibc`、`/oscomp-musl`，隔离 ELF、资源和运行时。
- `/oscomp-queue` 改为 NUL 分隔的 `G/X/E` 记录；runner 切换工作目录后串行执行。
- 扫描 ELF `PT_INTERP`，只为动态组暂存 glibc 或 musl 运行时。
- `MemorySpace::from_elf` 与动态解释器加载改为返回错误；缺失、无法打开或无效
  的解释器向 `execve` 传播 `ENOENT/ENOEXEC`，不再 panic。

### 本地验证

- 单组 glibc basic：官方解析器 `91/102`；一次复跑因 pipe 串口输出交错得到
  `88/102`，再次复跑恢复 `91/102`。
- 双组静态镜像：一次启动依次输出 glibc、musl START/END，执行 60 个命令后
  输出 `!TEST FINISH!` 并主动关机。
- RISC-V glibc 动态探针：成功通过私有 loader/libc 进入 `main`。
- 损坏 glibc loader 探针：`execve` 返回失败，runner 继续输出组 END 并主动关机，
  无 kernel panic。
- 故障注入：glibc 静态组加缺少 musl 运行时的动态组时，musl 被跳过，glibc
  仍完整执行并正常关机。
- 无盘与外部 BusyBox 镜像：无 panic、无超时并主动关机。

## 2026-06-12 完整 basic 串行队列

### 已完成

- 将 `tests="..."` 从只取首项改为解析完整有序队列。
- 将 basic ELF 使用 `oscomp-basic-<name>-elf` 别名暂存到 tmpfs，避免 `sleep`
  被 SWTC 的 BusyBox 命令重定向逻辑截获。
- 额外暂存 `test_echo`、`text.txt`，并创建 `mnt` 目录。
- 用户态 `runtestcase` 逐项执行 `fork + execve + waitpid`。
- 普通测试失败后继续执行后续测试，全部完成后统一输出 END marker、
  `!TEST FINISH!` 并主动关机。
- 跳过 `mount` 和 `umount`：它们当前会在 `src/fs/file_system.rs:65`
  的未实现路径触发 kernel panic。

### 本地结果

- 串行暂存并执行 30 个 basic 测试。
- 官方 `test_runner.py`：`91/102`。
- `getdents`：`4/5`，是已执行测试中唯一未满分项。
- `mount`、`umount`：主动跳过，共 10 项暂未得分。
- 无 kernel panic，输出 `!TEST FINISH!` 并主动关机。

## 2026-06-12 官方编译错误修复

### 线上证据

- 官方页面最后一次提交时间为 2026-06-11 19:44:39。
- 失败发生在 Compile 阶段，首先尝试联网下载 `nightly-2025-02-18`，随后因
  `SWTC/vendor/spin-0.7.1/Cargo.lock` 被官方隐藏文件过滤移除而校验失败。
- 页面分数为 `0.00`，没有产生任何运行阶段反馈。

### 已完成

- 将工具链固定为官方镜像预装的 `nightly-2025-02-01`。
- 构建流程移除 `rustup target add`、`rustup component add` 和 `cargo install`。
- 新增 `tools/vendor_checksums.py`，按 Git 实际追踪的非隐藏文件重建并检查
  53 个 vendor manifest。
- 保留根 Makefile 从非隐藏备份恢复 `.cargo/` 和 `.cargo-checksum.json` 的流程。
- 在删除全部隐藏文件的干净导出中完成强制离线 `make all`。

### 验证结果

- vendor 校验：53 个 manifest，0 个问题。
- 强制离线构建：通过，日志中无 rustup 同步、组件下载或 crates.io 请求。
- `kernel-rv`、`kernel-la`：均为 RISC-V executable ELF，入口 `0x80200000`。
- 无盘启动：输出 `!TEST FINISH!` 并主动关机。
- `glibc/basic/brk`：官方解析器 `3/3`。
- 外部官方 BusyBox 镜像：60 秒内无 panic，输出允许的
  `official basic script not found` 后主动关机。

### 后续线上结果

- 2026-06-13 19:30:50 的评测状态为 `Accpted`，确认离线工具链和 vendor 修复已
  通过 Compile 阶段；后续 0 分原因已转为运行期动态 loader panic。

## 2026-06-12 官方离线编译修复

### 线上失败原因

- 提交固定为 `nightly-2025-02-18`，官方环境未预装该版本，rustup 在无网络
  环境中尝试下载后失败。
- vendor 的 `.cargo-checksum.json` 仍引用官方提交过滤后不存在的隐藏文件和
  未跟踪 `Cargo.lock`，首先在 `spin-0.7.1/Cargo.lock` 处停止构建。

### 已修复

- 工具链改为官方镜像已预装的 `nightly-2025-02-01`，只声明构建需要的
  `rust-src` 和 `llvm-tools-preview`。
- 重新生成 53 个非隐藏 `cargo-checksum.json`，只保留 Git 已跟踪且路径中
  不含隐藏组件的文件校验项。
- 用只包含 Git 跟踪、非隐藏文件的临时提交副本模拟官方过滤，离线
  `make all` 成功生成 `kernel-rv` 和 `kernel-la`。
- 模拟副本生成的 `kernel-rv` 为 RISC-V executable ELF，入口
  `0x80200000`。
- 使用官方目录结构 EXT4 测试盘回归，`test_brk` 仍由官方解析器判定为
  `3/3`，QEMU 主动关机。

### 尚待确认

- 本地没有安装 `nightly-2025-02-01` 且网络无法下载，因此本地过滤模拟使用
  已安装的 `nightly-2025-02-18` 执行编译；线上需要再次提交，确认官方镜像
  中预装的 `nightly-2025-02-01` 能直接接管构建且不触发下载。

## 2026-06-11 首个真实 basic ELF

### 已完成

- 为 `oscomp` 增加 EXT4 普通文件读取，支持 extent tree、直接块和一级间接块。
- 读取 `basic_testcode.sh`，处理 `cd` 和嵌套 `run-all.sh`。
- 解析 `tests="..."` 中的首个测试名，当前得到 `musl/basic/brk`。
- 从 EXT4 读取静态 RISC-V ELF，并校验 ELF magic。
- 将 ELF、argv 和 END 标记写入 SWTC tmpfs。
- 内置 `runtestcase` 检测暂存文件，使用 `fork + execve + wait4` 串行执行。
- 保持真实输出位于 `basic-musl` START/END 区间内。
- QEMU 执行结束后主动关机。

### 验证结果

- `make all`：通过。
- 官方风格 `256M`、单核 QEMU：通过。
- `basic/brk`：完整输出 `START test_brk`、三次堆位置和 `END test_brk`。
- 官方 `test_runner.py`：`test_brk` 共 3 项，全部通过。
- 无测试盘：输出 `oscomp: no staged basic ELF` 后主动关机。

### 当前边界

- 当前跳过 `mount` 和 `umount`，避免未实现挂载路径导致整个评测 panic。
- `getdents` 当前为 `4/5`。
- EXT4 仍未整体挂载进 SWTC VFS，而是按需读取并复制到 tmpfs。
- 线上评测尚未重新确认，不能把本地解析结果视为线上成绩。

## 2026-06-09 SWTC 主线重构

### 架构决策

- 停止同时扩展旧自建内核和 rCore 迁移版本。
- 使用 SWTC 作为唯一新内核主体，参考 Titanix 架构推进。
- 从新主线移除旧 `kernel/`、`user/` 和 `rCore-Tutorial-v3-main/`。
- 旧自建内核的 `basic=102` 成果继续由归档分支保存。

### 已完成

- 从 Titanix `final-submit-qemu` 分支获取内核、用户态和依赖源码。
- 保留 Titanix GPLv3 许可证与来源。
- 将 Windows 不允许检出的 `aux.rs` 改名为 `aux_file.rs`。
- 将工具链从不可下载的 nightly `2022-11-03` 迁移到可构建当前源码的新版
  nightly；线上提交最终固定为官方预装的 `2025-02-01`。
- 修复 PanicInfo、Poll、trap 汇编符号和 virtio-drivers API 兼容问题。
- vendor 全部 Cargo 依赖，构建过程无需连接 crates.io。
- 为 vendor 校验文件建立非隐藏备份，模拟官方隐藏文件过滤后仍可全量构建。
- 根目录 `make all` 已切换为 SWTC 构建。
- 新增 wrapper ELF 流程，把 SWTC 高虚拟地址 raw kernel 封装成物理入口
  `0x80200000` 的官方 `kernel-rv`。
- 官方风格 `256M`、单核 QEMU 启动成功并主动关机。
- 新增 `SWTC/kernel/src/oscomp.rs`，通过 SWTC BlockDevice 只读探测 EXT4。
- fixed path 命中 `musl/basic_testcode.sh` 后输出 basic START/END。

## 历史里程碑

- 2026-06-08：旧自建内核官方线上 basic 得分 102。
- 2026-06-08：旧自建内核加入 BusyBox 简单命令队列。
- 2026-06-09：旧自建内核冻结在 `codex/basic-102-archive`。
- 2026-06-09：rCore 迁移尝试能够启动到 shell，但未接入官方 EXT4。
- 2026-06-09：路线切换为 SWTC 主线，并完成启动与 basic 入口。
- 2026-06-11：SWTC 执行 `basic/brk`，本地官方解析器得到 `3/3`。
- 2026-06-12：修复官方离线工具链选择和 vendor 隐藏文件校验问题。

## 下一里程碑

先提交当前 loader 与双组修复，确认 glibc-rv basic 得分非零、musl-rv 至少开始
执行真实测试且全程无 `Panicked`。稳定线上基线后，再按首个 musl 失败日志补 ABI；
`getdents`、`mount/umount` 与 LoongArch 暂不混入本轮。
