# 2026-06-19 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-19 17:00:35，`Accepted / 377.200790558321` |
| 当前直接失败原因 | `lmbench-lite` 已不影响既有基线，但 lmbench 仍为 0，优先修 marker、argv0 和 readlinkat |
| 上一条通过基线 | 2026-06-19 17:00:35，`Accepted / 377.200790558321` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.2007905583205` |
| 上一条编译错误 | 2026-06-19 14:51:46，`Compile Error / 0.00`；vendor checksum mismatch，已由 `0acac92` 修复 |
| 上一条高分结果 | 2026-06-18 09:46:55，`Accepted / 377.3228370332187` |
| 最新线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `30.12274508733359`、musl-rv `27.07823396849846` |
| 当前修复方向 | 保持现有 377 基线，只修 `lmbench-lite` 执行入口与评分 marker |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮目标是在不扩大测试范围的前提下修正 `lmbench-lite`：继续只执行
`lat_syscall null/read/write/stat/fstat/open`，但改为使用官方脚本 marker、
`lat_syscall` argv0 和正确的 `readlinkat` 返回长度。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. `SWTC/kernel/Cargo.lock` 中 `managed` 不再记录 registry source/checksum。
5. 官方完整参数下，无盘、basic 和 BusyBox 外部探针均无 panic、无全局超时并主动关机。
6. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench glibc-rv 不低于 `30.07126205049758` 附近。
- libcbench musl-rv 不低于 `27.166961800614022` 附近。
- 新增 lmbench START/END 使用官方脚本 marker；若仍为 0，也必须有明确
  execve 失败、readlinkat 异常或 timeout 日志。
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
