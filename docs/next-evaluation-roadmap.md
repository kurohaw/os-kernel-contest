# 2026-06-21 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 最新稳定结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594`；libctest-musl 已进 107 分 |
| 已止血问题 | `4602678` 扩容 libctest 后曾在 libcbench-glibc 阶段触发 `src/process/thread/exit.rs:74` 父进程 weak unwrap panic；14:43 结果已恢复且无 panic |
| 上一条通过基线 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.255157002759375`、libctest `107` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 最新线上得分 | basic `204`、BusyBox `98`、Lua `18`、libcbench `57.255157002759375`、libctest `107` |
| 当前修复方向 | 冻结 musl libctest full-static，锁定 484 基线后推进 lmbench-lite 9-command 探针 |
| 本轮代码基线 | 已基于 GitHub/GitLab `main` 的 `415b423 feat: run full musl libctest static set` |
| 本轮新增门禁修复 | 刷新 `SWTC/vendor/allocator-api2-0.2.21/cargo-checksum.json`，消除 22 个 stale checksum |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

这轮在 12:05 已确认 musl libctest static 全量进分：不再改 allowlist、timeout
或 `C` 队列协议。下一次只推进 lmbench-lite：原有 6 条 `lat_syscall` 加上
`lat_select file`、`lat_sig install`、`lat_sig catch`，并把单命令超时从 5 秒
放宽到 20 秒。核心要求是保持 basic、BusyBox、Lua、libcbench 与 107 个
libctest case，不因 lmbench 探针回退。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
5. 官方完整参数下，无盘、basic、libctest 和 lmbench 外部探针均无 panic、无全局超时并主动关机。
6. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试。
7. `C` 队列记录只服务 musl libctest，且 107 个 static case 已线上通过；后续不再扩容。
8. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench 维持约 `57` 分区间。
- `libctest-musl` 保持 `107/107`。
- lmbench 探针若失败，必须只表现为单命令 timeout 或无得分，不能引发 kernel panic。
- 不再出现 `src/process/thread/exit.rs:74` panic。
- 若遇到未支持 futex op，应返回 errno 或输出 warn，不应 kernel panic。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

后续新增测试组必须按以下顺序推进：

1. 先只识别脚本和资源，不执行正式命令，确认不影响 libcbench。
2. 再只暂存最小二进制和脚本，避免启动时一次性占用大块 tmpfs。
3. 最后用 `A` 或 `C` 记录执行最短命令，确认有 START/END、timeout 生效且主动关机后
   扩展完整脚本。
4. 若出现回退，立即撤回该测试组 staging，保留 484 基线。

## 后续提分顺序

1. 下一次先确认 484 基线不回退；若低于 480，优先撤回本轮 lmbench 9-command 探针。
2. `libctest` 已满分，除非官方回归，不再修改 allowlist、timeout 或 `C` 队列协议。
3. 若 lmbench 仍为 0，先从串口日志确认是超时、缺 syscall 还是 parser 未识别，再决定
   是否补 `select/sigaction` 语义或回退到 6-command。
4. iozone 先补齐安全返回路径，再只执行小文件 direct 命令，禁止恢复完整脚本。
5. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
6. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 iozone、iperf、netperf 或 ltp 正式执行组。
- 不处理网络、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
