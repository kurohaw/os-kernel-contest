# 2026-06-18 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-18 09:16:08，`Accepted / 320.0` |
| 线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9` |
| 当前修复方向 | 新增 libcbench 脚本型测试组 staging，保持 basic/BusyBox/Lua 不回退 |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，真实 libc-bench 运行待线上确认 |
| 当前已知边界 | LoongArch 占位 ELF；libcbench、lmbench、ltp、网络/性能测试仍未得分 |

这轮目标是保持 RISC-V `320.0` 基线，同时让 libcbench 组开始产生线上反馈或得分。

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
- libcbench glibc-rv 或 musl-rv 至少开始输出官方 `libcbench` START/END。
- 如果 libcbench 不得分，串口应能看到 `oscomp: found official libcbench script ...`，
  用于区分 staging 问题和 libc-bench 运行期 syscall/线程问题。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若 libcbench 仍为 0 分，先保存完整串口日志并按以下顺序定位：

1. 若没有 `found official libcbench script`，检查官方目录结构和 EXT4 path 探测。
2. 若脚本已暂存但没有 START marker，检查 `/busybox sh libcbench_testcode.sh` 的 execve。
3. 若 `libc-bench` 启动后失败，按首个 benchmark 输出定位 pthread、time、regex 或 stdio 缺口。
4. 若 libcbench 卡住或 panic，先收窄到单个源码 benchmark，再做最小 syscall 修复。

## 后续提分顺序

1. 根据下一次官方 libcbench 日志补首个阻塞 syscall/ABI，保持 `320.0` 基线。
2. 若 libcbench 能稳定得分，继续评估 lmbench 或 libctest。
3. 再推进 ltp、iozone、iperf、netperf 等更容易暴露文件系统、网络或多进程语义的问题。
4. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不处理网络、性能、多核和 LoongArch。
- 不为 libcbench 之外的测试组做大范围架构重构。
