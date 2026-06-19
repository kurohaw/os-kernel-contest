# 2026-06-19 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-19 14:05:15，`Accepted / 377.02594320298937` |
| 上一条高分结果 | 2026-06-18 09:46:55，`Accepted / 377.3228370332187` |
| 最新线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `29.86218129302594`、musl-rv `27.163761909963373` |
| 当前修复方向 | 保持 377 基线，后续只用小探针推进 iozone/lmbench/libctest |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮目标是在守住 `377.02594320298937` 基线的前提下，小步探测下一个可得分测试组。

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
- libcbench glibc-rv 不低于 `29.86218129302594` 附近。
- libcbench musl-rv 不低于 `27.163761909963373` 附近。
- 新测试组若不能起分，必须保证不会把 libcbench 拉回 0。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后再执行单个最短命令，确认有 START/END 且主动关机后扩展完整脚本。
4. 若出现回退，立即撤回该测试组 staging，保留 377 基线。

## 后续提分顺序

1. 基于 2025 multiarch 脚本分析 iozone、lmbench、libctest 的最小资源集合。
2. 优先尝试不会一次性暂存大文件、不会长时间运行的探针提交。
3. 若探针稳定，再决定是推进 iozone 文件系统路径，还是转向 lmbench/libctest。
4. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
5. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不处理网络、性能、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
