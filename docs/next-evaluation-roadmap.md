# 2026-06-19 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-18 16:00:21，`Accepted / 320.0`，属于 iozone staging 后的回退 |
| 上一条高分结果 | 2026-06-18 09:46:55，`Accepted / 377.3228370332187` |
| 高分线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `30.15271484677692`、musl-rv `27.170122186441827` |
| 当前修复方向 | 撤回 iozone staging，先恢复 libcbench 高分基线 |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮目标是先让 RISC-V 回到 `377.3228370332187` 高分线；在确认恢复前，不继续合入 iozone。

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
- libcbench glibc-rv 尽量恢复到 `30.15271484677692`。
- libcbench musl-rv 尽量恢复到 `27.170122186441827`。
- iozone 继续为 0 可以接受；本轮核心是确认撤回后不再拖累 libcbench。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若撤回后 libcbench 仍未恢复，先保存完整串口日志并按以下顺序定位：

1. 确认线上提交源码中不再包含 `install_iozone_groups`。
2. 对比 2026-06-18 09:46:55 高分提交和当前提交的源码差异。
3. 若出现 unsupported futex op，按日志补最小 op 语义。
4. 若 libcbench 卡住或 panic，先收窄到单个源码 benchmark，再做最小 syscall 修复。

## 后续提分顺序

1. 先确认撤回 iozone 后恢复 `377.3228370332187` 高分线。
2. 再基于完整日志将 iozone 拆成只探测脚本入口、只暂存静态资源、再执行正式命令的小提交。
3. 若 libcbench 能稳定得分，继续评估 lmbench 或 libctest。
4. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
5. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不处理网络、性能、多核和 LoongArch。
- 不重新合入 iozone，直到 libcbench 高分线确认恢复。
