# 2026-06-14 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-13 19:30:50，`Accpted / 0.0` |
| 线上直接失败原因 | musl 首个动态 ELF 缺少解释器，`memory_space/mod.rs:871` 的 `unwrap()` 触发 panic |
| 本地 glibc basic | 官方解析器复跑 `91/102` |
| 本地双组镜像 | 同一次启动依次执行 glibc、musl，共 60 个命令并主动关机 |
| 本地动态探针 | RISC-V glibc 动态 ELF 通过私有 loader/libc 成功进入 `main` |
| 当前已知边界 | LoongArch 占位 ELF、`getdents 4/5`、主动跳过 `mount/umount` |

这轮目标不是扩展更多测试组，而是把已经能编译的提交变成稳定的 RISC-V 非零分。
必须先保证 glibc 得分不会被后续 musl 动态运行失败拖垮。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. 官方完整参数下，无盘、单组 glibc、双组、动态 glibc 探针和 BusyBox 外部探针
   均无 panic、无超时并主动关机。
5. 单组 glibc basic 官方解析器复跑得到 `91/102`。
6. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- glibc-rv basic 得分非零。
- musl-rv 至少开始执行真实 basic 测试。
- 即使 musl 单项 `execve` 失败，也继续后续记录并输出组 END。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若仍为 0 分，先保存完整串口日志并按以下顺序定位：

1. 检查 glibc 组是否完成 START/END；若未完成，优先修复 glibc 基线。
2. 检查 musl 组是否因缺失运行时被整体跳过，或已进入首个真实 ELF。
3. 若 musl 已进入用户态，按首个缺失 syscall/ABI 日志做最小修复。
4. 若出现新的 kernel panic，优先消除 panic，不并行开发其他功能。

## 后续提分顺序

1. 根据下一次官方 musl 日志补首个阻塞 ABI，保持 glibc 得分基线。
2. 修复 `getdents` 最后 1 项及 pipe 串口输出交错的偶发计分波动。
3. 单独处理 `mount/umount` 未实现路径，确认不再 panic 后再取消跳过。
4. BusyBox、lua、libctest 按真实日志逐组推进。
5. LoongArch 作为独立里程碑，不与当前 RISC-V 稳定得分混合提交。

## 本轮暂缓

- 不新增 BusyBox 测试入口。
- 不处理 `getdents`、`mount/umount`。
- 不处理网络、性能、多核和 LoongArch。
- 不进行与当前 panic、双组隔离或动态运行时无关的架构重构。
