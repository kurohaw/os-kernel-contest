# 2026-06-20 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-20 11:12:19，`Accepted / 326.0` |
| 当前直接失败原因 | `b433976` 修改 `/proc/self/exe` readlinkat 后，libcbench 从 57.425 掉到 6.0 |
| 上一条通过基线 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.42523152095458` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-20 10:52:03，`Accepted / 377.42523152095464` |
| 最新线上得分 | basic glibc-rv `102/102`、musl-rv `102/102`；BusyBox glibc-rv `49/49`、musl-rv `49/49`；Lua glibc-rv `9/9`、musl-rv `9/9`；libcbench glibc-rv `6.0`、musl-rv `0.0` |
| 当前修复方向 | 在回退 `b433976` 后，只新增 musl `libctest` 小批量探针，目标是让 libctest 从 0 开始进分 |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮执行新的单指标策略：不再同时追 lmbench、readlinkat、argv 和资源路径，只把
musl `libctest` 做成最小 allowlist 探针。若仍为 0，依据日志判断是脚本布局、
execve、超时还是 case 返回非 0；若导致回退，立即撤回 libctest staging。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
5. 官方完整参数下，无盘、basic 和 BusyBox 外部探针均无 panic、无全局超时并主动关机。
6. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试。
7. 新增 `C` 队列记录只服务 musl libctest，不能改变既有 `G/X/A/E` 队列语义。
8. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench glibc-rv 恢复到 `30.237213649762825` 附近。
- libcbench musl-rv 恢复到 `27.18801787119176` 附近。
- 若官方镜像包含 `musl/libctest_testcode.sh` 和 `run-static.sh`，串口日志应出现
  `RUN LIBCTEST CASE string`、`stdlib`、`stdio` 以及对应 `Pass!` 或 `FAIL`。
- libctest 不能把现有 8 组或 libcbench 拉回 0。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后用 `A` 或 `C` 记录执行最短命令，确认有 START/END、timeout 生效且主动关机后
   扩展完整脚本。
4. 若出现回退，立即撤回该测试组 staging，保留 377 基线。

## 后续提分顺序

1. 观察本轮 musl `libctest` 官方结果：若有得分，继续按 allowlist 小批量加入
   `ctype`、`time`、`argv/env` 等基础 case；若 0 分，先看日志确认
   `entry-static.exe <case>` 是否真正执行。
2. `libctest` 稳定后，再回到 `lmbench-lite`，逐步加入 `lat_select file`、
   `lat_sig install/catch`；若 0 分，只根据 errno/timeout 修隔离 staging。
3. iozone 先补齐安全返回路径，再只执行小文件 direct 命令，禁止恢复完整脚本。
4. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
5. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 iozone、iperf、netperf 或 ltp 正式执行组。
- 不处理网络、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
