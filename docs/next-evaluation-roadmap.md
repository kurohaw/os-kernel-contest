# 2026-06-25 下一次评测路线

## 2026-06-25 Compile Error 修复

最新官方编译错误不是 RISC-V 代码失败，而是默认 `make all` 强制进入
LoongArch 严格工具链检查。评测机缺少 `nightly-2025-02-18` 的
`loongarch64-unknown-none` target，导致 `check-la-tools` 退出 1。
本轮把默认 `build-la` 改为 best-effort：工具链齐全时构建真实 `SWTC-la`，
不可用时复制 `kernel-rv` 为占位 `kernel-la`；真实 LA 构建保留在
`build-la-strict`。

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-23 18:05:27，`Accepted / 484.32498298746674`；basic=204、BusyBox=98、Lua=18、libcbench=57.32498298746679、libctest=107 |
| 最新稳定官方结果 | 2026-06-23 18:05:27，`Accepted / 484.32498298746674`；libcbench 约 57.32 分、libctest-musl=107 |
| 最新高分结果 | 2026-06-21 13:15:41，`Accepted / 484.26735406790885`；iozone-lite 撤回后已恢复 |
| 已止血问题 | `4602678` 扩容 libctest 后曾在 libcbench-glibc 阶段触发 `src/process/thread/exit.rs:74` 父进程 weak unwrap panic；14:43 结果已恢复且无 panic |
| 上一条通过基线 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 通过基线得分构成 | RISC-V basic `204`、BusyBox `98`、Lua `18`、libcbench `57.255157002759375`、libctest `107` |
| 上一条编译错误 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮通过移除 `hashbrown` 依赖链修复 |
| 上一条高分结果 | 2026-06-21 12:05:08，`Accepted / 484.2551570027594` |
| 最新线上得分 | basic `204`、BusyBox `98`、Lua `18`、libcbench `0`、libctest `0`、lmbench `0` |
| 当前修复方向 | 先恢复 Compile 阶段，LoongArch 默认 fallback 不再阻塞 RISC-V 基线 |
| 本轮代码基线 | 在 `d6746eb fix: add lmbench runtime skeleton` 基础上，删除全局 `/bin/sh`、loader/lib、`/etc/passwd`、`/tmp/memfd` staging |
| 本轮新增门禁修复 | `64fe8b4` 已撤回 `8690e03 feat: add minimal iozone probe` |
| 本地双组 basic | 官方解析器复跑 `102/102` |
| 本地 libcbench staging | glibc/musl libcbench 脚本和静态 ELF 可从 EXT4 暂存到 tmpfs，线上已证明能得分 |
| 当前已知边界 | LoongArch 当前只启用 basic；cyclictest、iozone、lmbench、网络/性能测试仍未稳定得分 |

## 2026-06-24 快速增长四路线

1. **musl dynamic libctest**：已完成本地官方镜像验证。static `107/107`、
   dynamic `110/110`，预计线上在 484 稳定基线上增加约 110 分。
2. **LTP 批量队列**：首批已完成，官方镜像本地 22/22 返回 0；继续扩容前先等待
   线上确认，并依据首个失败项补 syscall/VFS，不恢复会 panic 的宽泛 256 项队列。
3. **LoongArch**：已完成第一阶段。真实 `kernel-la` 已离线构建，官方 LA 镜像
   本地 musl/glibc basic 64/64 通过；下一阶段扩展 BusyBox、Lua、libctest 和 LTP。
4. **性能测试**：在 1 GiB 物理帧范围修复后重新评估 libcbench/lmbench，再逐项
   推进 cyclictest、iozone、iperf/netperf；不得恢复曾导致 320 回退的全局 staging。

第一路线的完整官方镜像运行已经证明此前后半程归零的关键原因之一是 submit
模式仅管理 128 MiB 物理内存。当前已改为匹配官方 `-m 1G`，并通过 256 MiB
无盘兼容回归。新一轮线上结果出来前，仍以 484.32498298746674 作为线上稳定基线。

2026-06-24 已确认 `d500180` 编译通过但线上回退到 320：基础组仍在，
libcbench/libctest 全部丢分，cyclictest 也没有得分。当前已完整撤回该提交；
恢复 480 基线前不再扩大测试组或改全局 VFS 环境。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. 默认 `make all` 必须生成 `kernel-la`；若 LA 工具链不可用，允许占位 ELF。
   真实 LoongArch ELF 与官方 LA 镜像 basic 64/64 只作为 `build-la-strict` 门禁。
5. `SWTC/kernel/Cargo.lock` 中不再出现 `hashbrown`、`ahash` 或
   `allocator-api2`；`managed` 仍不记录 registry source/checksum。
6. 官方完整参数下，无盘、basic 和 lmbench 外部探针均无 panic、无全局超时并主动关机。
7. `readlinkat` 行为与 `e8d1b48` 保持一致，不保留 `b433976` 的真实路径尝试；
   lmbench staging 必须提供 `/lmbench_all` 根路径别名。
8. submit 默认构建 feature 应为 `submit tmpfs`，不再带 `stack_trace`；需要诊断时手动传 `STACK_TRACE=1`。
9. fake lmbench EXT4 盘应暂存 2 组共 18 条命令并主动关机。
10. `C/A` 队列记录服务 musl libctest；107 个 static case 已线上通过，110 个
   dynamic case 已在官方镜像本地通过，后续不得改动其顺序和隔离运行时。
11. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- Compile 阶段通过，不再出现 `no matching package named managed found` 或
  `no matching package found: ahash`。
- RISC-V basic 保持 `204/204`。
- RISC-V BusyBox 保持 `98/98`。
- RISC-V Lua 保持 `18/18`。
- libcbench 恢复约 `57` 分区间。
- `libctest-musl` 应由 `107/107` 提升到 static + dynamic `217/217`。
- lmbench 若仍为 0，但总分保持 480 以上，才继续看串口日志研究真实 benchmark 输出。
- 若总分仍低于 480，先核对评测提交哈希和完整串口日志，不再盲目扩大 staging。
- cyclictest 探针已证明会造成 320 回退，不再启用。
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

1. 提交已本地验证的 musl dynamic libctest，目标先从约 484 提升到约 594。
2. dynamic libctest 与 LTP 22 项线上确认后冻结，下一主线转向真实 LoongArch。
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
