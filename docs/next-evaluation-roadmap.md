# 2026-06-23 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-22 18:46:51，`Accepted / 484.1693353980349`；libctest-musl 仍为 107 分，lmbench 仍为 0 |
| 最新高分结果 | 2026-06-21 13:15:41，`Accepted / 484.26735406790885`；iozone-lite 撤回后已恢复 |
| 已止血问题 | `4602678` 扩容 libctest 后曾在 libcbench-glibc 阶段触发 `src/process/thread/exit.rs:74` 父进程 weak unwrap panic；14:43 结果已恢复且无 panic |
| 上一条通过基线 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.255157002759375`、libctest `107` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 最新线上得分 | basic `204`、BusyBox `98`、Lua `18`、libcbench `57.16933539803484`、libctest `107`、lmbench `0` |
| 当前修复方向 | 保留 submit 默认关闭 `stack_trace`，撤回 24-command lmbench 扩张，补官方运行环境骨架后复测 9-command lite |
| 本轮代码基线 | 在 `c941356 feat: expand lmbench main queue` 基础上，回到短 lmbench 队列并补 `/bin/sh`、全局 loader/lib、`/etc/passwd`、`/tmp/memfd` |
| 本轮新增门禁修复 | `64fe8b4` 已撤回 `8690e03 feat: add minimal iozone probe` |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 占位 ELF；iozone、lmbench、ltp、网络/性能测试仍未稳定得分 |

18:46 已确认 `c941356` 编译通过且保持 483-484 基线，但 24-command lmbench
仍为 0，并把评测时间拉长到约 17 分钟。下一轮不伪造 lmbench 输出，先补官方脚本会
准备的运行环境骨架，并把 lmbench 恢复到短探针；若仍为 0，再看串口日志判断是否
进入真实 benchmark 输出路径。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
5. 官方完整参数下，无盘、basic、libctest 和 lmbench 外部探针均无 panic、无全局超时并主动关机。
6. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试；
   lmbench staging 必须提供 `/lmbench_all` 根路径别名。
7. submit 默认构建 feature 应为 `submit tmpfs`，不再带 `stack_trace`；需要诊断时手动传 `STACK_TRACE=1`。
8. fake lmbench EXT4 盘应暂存 2 组共 18 条命令并主动关机。
9. `C` 队列记录只服务 musl libctest，且 107 个 static case 已线上通过；后续不再扩容。
10. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench 维持约 `57` 分区间。
- `libctest-musl` 保持 `107/107`。
- lmbench 若仍为 0，但总分保持 480 以上，下一轮转向 pipe/select/fork 热路径优化。
- 若总分低于 480，优先撤回运行环境骨架，只保留关闭 `stack_trace` 的性能构建单独复测。
- 不再暂存或执行 iozone；完整脚本和 lite 探针都已证明会导致 320 回退。
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

1. 483-484 基线已恢复；下一轮如果低于 480，先确认是否有人重新引入 iozone 或新组 staging。
2. `libctest` 已满分，除非官方回归，不再修改 allowlist、timeout 或 `C` 队列协议。
3. 若 lmbench 仍为 0，必须看串口日志确认是否出现 `Simple syscall:`、
   `Select on 100 fd` 或 `Signal handler installation:`；没有这些行就继续修
   执行/超时路径，有这些行但不计分再查 parser 分组。
4. iozone 暂停；没有完整官方串口日志前，不再做任何 iozone staging。
5. 再推进 ltp、iperf、netperf 等更容易暴露网络或多进程语义的问题。
6. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 iozone、iperf、netperf 或 ltp 正式执行组。
- 不处理网络、多核和 LoongArch。
- 不一次性重新合入完整 iozone。
