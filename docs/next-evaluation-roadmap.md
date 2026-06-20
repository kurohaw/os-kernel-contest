# 2026-06-20 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-20 13:57:28，`Accepted / 385.00317485255493` |
| 最新已提交结果 | 2026-06-20 13:57:28，`Accepted / 385.00317485255493`；libctest-musl 已进 8 分 |
| 当前直接瓶颈 | 已得分组稳定；lmbench/iozone/network/ltp 仍为 0，短期优先扩大已验证的 libctest 静态覆盖 |
| 上一条通过基线 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.42523152095458` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 最新线上得分 | basic `204`、BusyBox `98`、Lua `18`、libcbench `57.003174852554935`、libctest `8`；lmbench/iozone/network/ltp 仍为 0 |
| 当前修复方向 | 扩大 musl `libctest` 静态低风险 case；`lmbench` 已打印部分耗时但官方仍计 0，本轮不继续扩大 |
| 本轮远端同步 | 当前本地基于 GitLab `main` 的 `02ac1f3 fix: pass lmbench function as argument` |
| 本轮新增门禁修复 | 刷新 `SWTC/vendor/allocator-api2-0.2.21/cargo-checksum.json`，消除 22 个 stale checksum |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮继续执行单指标策略：只扩大 `libctest-musl` 静态 allowlist。已经通过
官方日志确认 8 个静态 case 可得分，因此下一步优先把更多低风险 case 放进
同一条带超时的 `C` 队列；pthread、socket、dlopen、fork 压力项仍不放行。
若导致 385 基线回退，立即缩小 allowlist，保留已验证的 8 个 case。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
5. 官方完整参数下，无盘、basic 和 BusyBox 外部探针均无 panic、无全局超时并主动关机。
6. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试。
7. `C` 队列记录只服务 musl libctest，不能改变既有 `G/X/A/E` 队列语义。
8. 本轮只扩大 `LIBCTEST_ALLOWLIST` 和 `MAX_LIBCTEST_CASES`，不混入 iozone、
   network、ltp、LoongArch 或动态 libctest。
9. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench glibc-rv 保持约 `29` 到 `30` 分区间。
- libcbench musl-rv 保持约 `27` 分区间。
- `libctest-musl` 至少保持 8 分；若新增 case 与官方表匹配，应从 8 分继续上升。
- `lmbench` 可以继续为 0，但不能因为本轮改动影响 basic、BusyBox、Lua、
  libcbench 或 libctest 已有得分。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后用 `A` 或 `C` 记录执行最短命令，确认有 START/END、timeout 生效且主动关机后
   扩展完整脚本。
4. 若出现回退，立即撤回该测试组 staging，保留 377 基线。

## 后续提分顺序

1. 观察本轮 musl `libctest` 扩容结果：若有新增得分，继续按官方表分批加入
   剩余静态纯 libc case；若回退，缩回已验证的 8 个 case。
2. `libctest` 静态覆盖稳定后，再回到 `lmbench-lite`，逐步加入 `lat_select file`、
   `lat_sig install/catch`；若 0 分，只根据 errno/timeout 修隔离 staging。
3. iozone 先补齐安全返回路径，再只执行小文件 direct 命令，禁止恢复完整脚本。
4. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
5. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 iozone、iperf、netperf 或 ltp 正式执行组。
- 不处理网络、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
