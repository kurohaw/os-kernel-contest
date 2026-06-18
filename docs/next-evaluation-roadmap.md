# 2026-06-18 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-18 09:33:47，`Accepted / 326.0` |
| 线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `6` |
| 当前修复方向 | 补 futex bitset 和未知 op 降级，继续推进 libcbench pthread/futex 段 |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，真实 libc-bench 运行待线上确认 |
| 当前已知边界 | LoongArch 占位 ELF；libcbench、lmbench、ltp、网络/性能测试仍未得分 |

这轮目标是保持 RISC-V `326.0` 基线，同时让 libcbench 继续进分，并尽量让 musl-rv libcbench 也跑到。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. 官方完整参数下，无盘、单组 glibc、双组、动态 glibc、未知扩展 header 和
   BusyBox 外部探针均无 panic、无超时并主动关机。
5. 双组 basic 官方解析器复跑得到 `102/102`。
6. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench glibc-rv 不低于 `6`。
- libcbench musl-rv 尽量不再是 `0`；若仍为 0，串口应能显示 glibc 组后续失败点或 musl 组入口。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若 libcbench 不再进分，先保存完整串口日志并按以下顺序定位：

1. 若只到第 6 个 benchmark，优先看 `b_malloc_thread_stress` 的 clone/futex 日志。
2. 若 glibc 组跑完但 musl 仍为 0，检查 musl `libc-bench` 的 syscall 差异。
3. 若出现 unsupported futex op，按日志补最小 op 语义。
4. 若 libcbench 卡住或 panic，先收窄到单个源码 benchmark，再做最小 syscall 修复。

## 后续提分顺序

1. 根据下一次官方 libcbench 日志补首个阻塞 syscall/ABI，保持 `326.0` 基线。
2. 若 libcbench 能稳定得分，继续评估 lmbench 或 libctest。
3. 再推进 ltp、iozone、iperf、netperf 等更容易暴露文件系统、网络或多进程语义的问题。
4. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不处理网络、性能、多核和 LoongArch。
- 不为 libcbench 之外的测试组做大范围架构重构。
