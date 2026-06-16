# 2026-06-16 下一次评测路线

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 最新可见官方结果 | 2026-06-15 19:24:27，`Accpted / 91.0` |
| 结果时序 | 该 HTML 早于 `gitlab/main` 的 `0bc0dc9`，不是最新远端提交的评测结果 |
| 线上得分 | glibc-rv basic `91/102`；musl-rv、两项 LoongArch 均为 0 |
| 当前直接失败原因 | musl 组正常开始和结束，但 30 个 ELF 全部 `execve ... failed: -2`，即 `ENOENT` |
| 当前修复方向 | 从 musl ELF 的 `PT_INTERP` 提取真实解释器路径，并在 `/oscomp-musl` 下完整匹配暂存 |
| 本地 glibc basic | 官方解析器复跑 `91/102` |
| 本地双组镜像 | 同一次启动依次执行 glibc、musl，共 60 个命令并主动关机 |
| 本地动态探针 | RISC-V glibc 动态 ELF 通过私有 loader/libc 成功进入 `main` |
| 当前已知边界 | LoongArch 占位 ELF、`getdents 4/5`、主动跳过 `mount/umount` |

这轮目标是保持 glibc-rv 91 分，同时恢复至少一个 musl-rv 真实 basic 测试。

## 本轮提交门禁

1. 强制离线 `make all`，vendor checksum 保持 `53/0`。
2. 隐藏文件过滤后的干净导出仍能恢复 Cargo 配置并构建。
3. `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
4. 官方完整参数下，无盘、单组 glibc、双组、动态 glibc、未知扩展 header 和
   BusyBox 外部探针均无 panic、无超时并主动关机。
5. 单组 glibc basic 官方解析器复跑得到 `91/102`。
6. Git 状态不包含本地说明、镜像、日志、验证夹具或构建产物。

## 下一次官方评测验收

- glibc-rv basic 保持至少 91 分。
- musl-rv 不再出现 30 个 ELF 全部 `execve failed: -2`。
- musl-rv 至少开始执行一个真实 basic 测试；如果仍失败，串口必须输出真实
  `PT_INTERP`、负 errno 和 loader 失败阶段。
- RISC-V 输出中没有 `Panicked`，最终输出 `!TEST FINISH!` 并主动关机。

若仍为 0 分，先保存完整串口日志并按以下顺序定位：

1. 根据首个 musl `execve` errno 与阶段日志定位共同失败点。
2. 若仍为 `ENOENT`，对照日志中的真实 `PT_INTERP` 检查组内路径暂存。
3. 若为 `ENOEXEC`，对照官方 musl ELF/loader 的 program headers 修复兼容性。
4. 若 musl 已进入用户态，按首个缺失 syscall/ABI 日志做最小修复。

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
