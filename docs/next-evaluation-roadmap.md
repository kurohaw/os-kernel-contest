# 2026-06-12 今日提分计划

## 当前可信基线

| 证据 | 结论 |
|---|---|
| 官方页面最后可见结果 | 2026-06-11 19:44:39，`0.00 / Compile Error` |
| 旧线上失败原因 | 下载 `nightly-2025-02-18` 失败；隐藏 `Cargo.lock` 导致 vendor checksum 失败 |
| 当前 `gitlab/main` | `bab4cd0`，已修复离线工具链和 vendor checksum，但尚无对应线上结果 |
| 当前本地运行能力 | 串行执行 30 个 basic 测试，官方解析器确认 `91/102` |
| 当前未得分项 | `getdents` 少 1 项；主动跳过 `mount/umount` 共 10 项 |
| 当前主要瓶颈 | 线上结果尚未验证；挂载路径会触发 kernel panic |
| 历史参考上限 | 旧自建内核曾取得官方 basic=102；Titanix 上游也记录了完整 basic/BusyBox/libc 能力 |

今天不能把本地 `brk=3/3` 或 Titanix 上游能力当作线上得分。第一目标是消除
Compile 阶段的不确定性，第二目标是把本地 `91/102` 转化为稳定线上分数。

## 从全部 Markdown 得出的优先级

- `README.md`、`docs/progress.md` 和 `docs/test-matrix.md` 表明：构建、启动、
  EXT4 读取和完整 basic 串行队列已闭环，当前线上验证和剩余语义是主要瓶颈。
- `titanix/README.md` 表明上游曾运行 BusyBox、libc 和性能测试，说明现有内核
  可能已经具备大量 syscall 能力；今天应先让官方 basic 测试真正执行起来。
- `titanix/docs/preliminary.md`、`fs_syscall.md`、`syscall.md` 和 `thread.md`
  表明进程、fd、wait、pipe 等路径已有实现经验，而目录、文件、mount 和 mmap
  是更高风险区域。
- `titanix/docs/bugfix.md` 提醒后续重点观察 ELF 段对齐、waitpid 返回值、TLS、
  信号上下文和阻塞 syscall；这些问题应由真实失败日志驱动，不提前重构。
- `titanix/docs/rtld.md`、`libc.md`、`signal.md` 表明动态链接和 libc 需要
  auxv、mmap、TLS、信号等组合能力，不适合作为今天第一轮提分入口。
- `titanix/docs/net.md`、`redis.md`、`vi.md` 和 `todo.md` 中的网络、性能、多核、
  调度和交互优化不会解决当前 Compile/basic 得分瓶颈，今天暂缓。
- `docs/boot-notes.md` 是旧路线阅读笔记，只作为启动机制参考，不作为当前计划。

## 今日目标

### 必达

1. 触发 `bab4cd0` 的官方评测，取得新的 Compile 结果并保存完整日志。
2. 若 Compile 通过，确认完整 basic 队列的线上得分接近本地 `91/102`。
3. 每次修改保持离线 `make all`、完整 basic `91/102`、无盘主动关机和
   BusyBox 探针通过。

### 冲刺

- 修复 `getdents` 最后 1 项。
- 隔离并修复 `mount/umount` 未实现路径，向 basic `102/102` 靠近。

## 时间表

### 12:45-13:15：线上编译门禁

1. 在官方页面触发当前 `gitlab/main` 的新评测。
2. 确认评测对应提交不早于 `bab4cd0`。
3. 保存 Compile 输出、开始时间、最终分数和运行日志。

停止条件：若仍为 Compile Error，暂停后续功能开发，只根据新日志修复；修复后
先重复隐藏文件过滤和强制离线构建，再重新评测。

### 已完成：完整 basic 串行队列

- 解析完整测试列表并暂存 30 个安全 ELF。
- 暂存 `test_echo`、`text.txt`，创建 `mnt`。
- 串行 `fork/execve/waitpid`，全部完成后统一关机。
- 跳过会 panic 的 `mount/umount`。
- 本地官方解析器结果：`91/102`。

### 下一阶段：按真实日志修复剩余项

优先顺序：

1. 先触发线上评测，确认 Compile 和实际 basic 得分。
2. 修复 `getdents` 输出或 ABI，使其从 `4/5` 提升到 `5/5`。
3. 单独定位 `src/fs/file_system.rs:65` 的 mount 未实现路径。
4. 仅在 `mount/umount` 不再 panic 后取消跳过。

每次修复都运行完整 basic。若修改导致分数低于 `91/102`、panic 或卡死，立即
回退该项并保留稳定基线。

### 18:30-19:30：回归与第二次线上评测

1. 运行 vendor checksum、隐藏文件过滤导出和强制离线 `make all`。
2. 检查两个 ELF、无盘启动、完整 basic、首个 `brk=3/3` 和 BusyBox 外部探针。
3. 提交并推送稳定版本，触发第二次线上评测。
4. 将线上分数、首个失败项和日志摘要回填到 `progress.md` 与 `test-matrix.md`。

## 今日暂缓

- BusyBox 测试入口、动态链接和 libc 全量测试。
- mount/umount 的 `/dev/vda2` 与 FAT 挂载，除非它们是 basic 队列中唯一剩余阻塞。
- 网络、Redis、交互终端、性能、多核、LoongArch 和架构重构。

这些方向具备长期价值，但今天的边际收益低于“线上编译通过 + 完整 basic 队列”。

## 每轮提交门禁

- `python tools/vendor_checksums.py --check`：0 issues。
- 隐藏文件过滤后的干净导出仍可构建。
- 构建日志无 rustup 下载、`cargo install` 或 crates.io 请求。
- `kernel-rv` 为 RISC-V executable ELF，入口 `0x80200000`。
- 无盘、basic 和 BusyBox 探针均无 panic，并主动关机。
- Git 状态不包含本地说明文件、镜像、日志和构建产物。
