# 2026-06-18 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-18 08:55:11，`Accepted / 302.0` |
| 线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49` |
| 当前修复方向 | 新增 Lua 脚本型测试组 staging，保持 basic/BusyBox 不回退 |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 Lua staging | glibc/musl Lua 脚本和资源可从 EXT4 暂存到 tmpfs，真实 Lua 二进制运行待线上确认 |
| 当前已知边界 | LoongArch 占位 ELF；Lua、libcbench、lmbench、ltp、网络/性能测试仍未得分 |

这轮目标是保持 RISC-V `302.0` 基线，同时让 Lua 组开始产生线上反馈或得分。

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
- Lua glibc-rv 或 musl-rv 至少开始输出官方 `lua` START/END 或 testcase 行。
- 如果 Lua 不得分，串口应能看到 `oscomp: found official lua script ...`，用于区分
  staging 问题和 Lua 运行期 syscall/脚本问题。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若 Lua 仍为 0 分，先保存完整串口日志并按以下顺序定位：

1. 若没有 `found official lua script`，检查官方目录结构和 EXT4 path 探测。
2. 若脚本已暂存但没有 START marker，检查 `/busybox sh lua_testcode.sh` 的 execve。
3. 若 Lua testcase 输出 fail，按首个失败脚本补文件、时间、随机数或 math 相关 syscall/ABI。
4. 若 Lua 卡住或 panic，先收窄到单个脚本资源，再做最小 syscall 修复。

## 后续提分顺序

1. 根据下一次官方 Lua 日志补首个阻塞 syscall/ABI，保持 `302.0` 基线。
2. 若 Lua 能稳定得分，继续评估 libcbench；它也是静态脚本组，但运行压力更高。
3. 再推进 libctest、lmbench、ltp 等更容易暴露动态运行时或多进程语义的问题。
4. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不处理网络、性能、多核和 LoongArch。
- 不为 Lua 之外的测试组做大范围架构重构。
