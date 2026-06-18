# 2026-06-18 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-18 09:46:55，`Accepted / 377.3228370332187` |
| 线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `30.15271484677692`、musl-rv `27.170122186441827` |
| 当前修复方向 | 保持 libcbench 新增分，接入 iozone staging |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 iozone staging | glibc/musl iozone 脚本、`busybox` 和 `iozone` 可从 EXT4 暂存到 tmpfs，动态运行时按组复制 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未得分 |

这轮目标是保持 RISC-V `377.3228370332187` 基线，同时让 iozone 开始得分。

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
- libcbench glibc-rv 不低于 `30.15271484677692`。
- libcbench musl-rv 不低于 `27.170122186441827`。
- iozone glibc-rv 或 musl-rv 尽量开始得分；若仍为 0，串口应能显示 iozone 组入口或首个失败点。
- 若遇到未支持 syscall/op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若 iozone 不进分，先保存完整串口日志并按以下顺序定位：

1. 若脚本未进入，检查 `iozone_testcode.sh` 的工作目录、`busybox` 和 `iozone`
   暂存路径。
2. 若 `execve` 失败，检查 iozone 的 `PT_INTERP`、loader/libc/libm 暂存路径。
3. 若进入 iozone 后失败，优先看文件创建、truncate、lseek、fsync、statfs 等
   VFS/syscall 差异。
4. 若 iozone 卡住或 panic，先收窄到脚本中的第一个 iozone 参数组合，再做最小修复。

## 后续提分顺序

1. 根据下一次官方 iozone 日志补首个阻塞 syscall/VFS 语义，保持 `377.3228370332187` 基线。
2. 若 iozone 能稳定得分，继续评估 lmbench 或 libctest。
3. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
4. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不处理网络、多核和 LoongArch。
- 不为 iozone 之外的测试组做大范围架构重构。
