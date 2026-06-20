# 2026-06-19 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 当前直接失败原因 | `lmbench-lite` 仍为 0；怀疑真实 `lmbench_all` 通过 `/proc/self/exe` 找自身时拿到了不存在的 `/lmbench_all` |
| 上一条通过基线 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.42523152095458` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 最新线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `30.237213649762825`、musl-rv `27.18801787119176` |
| 当前修复方向 | 保持 377 基线，修正 `/proc/self/exe` 为当前进程真实执行路径，继续低风险探测 lmbench |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮目标是在官方 Accepted 回归后继续小步推进 `lmbench-lite`：不增加新测试组，
只把 `readlinkat(/proc/self/exe)` 从硬编码 `/lmbench_all` 改为当前进程的真实
`execve` 路径，避免真实 `lmbench_all` 依赖自身路径时直接失败。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
5. 官方完整参数下，无盘、basic 和 BusyBox 外部探针均无 panic、无全局超时并主动关机。
6. `readlinkat(/proc/self/exe)` 返回当前进程 `execve` 的绝对路径。
7. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench glibc-rv 不低于 `30.07126205049758` 附近。
- libcbench musl-rv 不低于 `27.166961800614022` 附近。
- `lmbench-lite` 若仍为 0，下一步优先查看是否还有资源路径、argv0 或 timeout
  线索；不要一次性扩大 lmbench 命令集。
- `lmbench-lite` 不能把现有 8 组或 libcbench 拉回 0。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后用 `A` 记录执行最短命令，确认有 START/END、timeout 生效且主动关机后
   扩展完整脚本。
4. 若出现回退，立即撤回该测试组 staging，保留 377 基线。

## 后续提分顺序

1. 观察 `lmbench-lite` 官方结果：若有得分，逐步加入 `lat_select file`、
   `lat_sig install/catch`；若 0 分，先根据 errno/timeout 修资源路径或 syscall。
2. 以同一 `A` 协议尝试 musl `libctest` allowlist，小批量执行字符串、数学、
   stdio、stdlib、argv/env 和基础时间类。
3. iozone 先补齐安全返回路径，再只执行小文件 direct 命令，禁止恢复完整脚本。
4. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
5. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 libctest、iozone、iperf、netperf 或 ltp 正式执行组。
- 不处理网络、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
