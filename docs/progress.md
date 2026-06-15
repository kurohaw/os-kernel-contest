# 初赛开发进度

## 当前状态

| 项目 | 内容 |
|---|---|
| 当前日期 | 2026-06-15 |
| 当前分支 | `main` |
| 当前内核主体 | `titanix/` |
| 历史保分基线 | 旧自建内核曾取得官方 basic=102 |
| 当前里程碑 | 官方 glibc-rv basic 取得 91 分，正在恢复 musl-rv execve |
| 当前提交 | `9a7fdb9`，已推送至 `gitlab/main` |
| 最新可见线上结果 | 2026-06-15 15:43:27，`Accpted / 91.0`；glibc-rv=91，musl-rv=0 |
| 本地得分闭环 | 官方 basic 解析器 `91/102` |

## 2026-06-15 官方 91 分与 musl execve 诊断

### 线上证据

- 官方总分由 `0.0` 提升至 `91.0`，glibc-rv basic 为 `91/102`。
- glibc、musl 两组均完成 START/END，RISC-V 无 panic 并主动关机。
- musl 组已暂存 30 个 ELF，但 30 次 `execve` 全部失败，没有进入任何
  `========== START test_* ==========`。
- LoongArch 仍因 `kernel-la` 是 RISC-V 占位 ELF 无法加载。

### 当前修复

- runner 输出 `execve` 的负 errno，下一次线上失败可直接区分 `ENOENT/ENOEXEC`。
- ELF 安全校验只检查头部、program-header 表和段文件范围，不再拒绝 loader
  不使用的合法扩展 program-header 类型。
- ELF 映射和解释器扫描只处理需要的 `PT_LOAD/PT_INTERP`，未知类型安全跳过。
- 主 ELF、解释器查找/布局/解析和进程替换失败时输出简短阶段日志。

### 本地验证

- glibc basic 仍为 `91/102`。
- 动态 glibc、双组静态、BusyBox 外部探针和无盘启动均无 panic 并主动关机。
- 将非执行的 RISC-V attributes header 改为未知扩展类型后，动态 ELF 仍进入
  `main`。
- 损坏 loader 探针输出 `execve ... failed: -8`，继续输出组 END 并主动关机。

## 2026-06-14 动态 loader 与双组队列

### 线上证据

- 2026-06-13 19:30:50 的官方提交已编译成功，状态为 `Accpted`。
- RISC-V 找到 `musl/basic_testcode.sh` 并暂存 30 个 basic ELF。
- 首个动态 ELF 执行时，tmpfs 缺少 musl 动态解释器；
  `memory_space/mod.rs:871` 对解释器 inode 执行 `unwrap()`，导致内核 panic。
- LoongArch 因 `kernel-la` 仍是 RISC-V 占位 ELF 无法加载，本轮未处理。

### 已完成

- basic 探测顺序固定为 `glibc -> musl`，仅在两者都不存在时使用根目录脚本。
- 两组分别暂存到 `/oscomp-glibc`、`/oscomp-musl`，隔离 ELF、资源和运行时。
- `/oscomp-queue` 改为 NUL 分隔的 `G/X/E` 记录；runner 切换工作目录后串行执行。
- 扫描 ELF `PT_INTERP`，只为动态组暂存 glibc 或 musl 运行时。
- `MemorySpace::from_elf` 与动态解释器加载改为返回错误；缺失、无法打开或无效
  的解释器向 `execve` 传播 `ENOENT/ENOEXEC`，不再 panic。

### 本地验证

- 单组 glibc basic：官方解析器 `91/102`；一次复跑因 pipe 串口输出交错得到
  `88/102`，再次复跑恢复 `91/102`。
- 双组静态镜像：一次启动依次输出 glibc、musl START/END，执行 60 个命令后
  输出 `!TEST FINISH!` 并主动关机。
- RISC-V glibc 动态探针：成功通过私有 loader/libc 进入 `main`。
- 损坏 glibc loader 探针：`execve` 返回失败，runner 继续输出组 END 并主动关机，
  无 kernel panic。
- 故障注入：glibc 静态组加缺少 musl 运行时的动态组时，musl 被跳过，glibc
  仍完整执行并正常关机。
- 无盘与外部 BusyBox 镜像：无 panic、无超时并主动关机。

## 2026-06-12 完整 basic 串行队列

### 已完成

- 将 `tests="..."` 从只取首项改为解析完整有序队列。
- 将 basic ELF 使用 `oscomp-basic-<name>-elf` 别名暂存到 tmpfs，避免 `sleep`
  被 Titanix 的 BusyBox 命令重定向逻辑截获。
- 额外暂存 `test_echo`、`text.txt`，并创建 `mnt` 目录。
- 用户态 `runtestcase` 逐项执行 `fork + execve + waitpid`。
- 普通测试失败后继续执行后续测试，全部完成后统一输出 END marker、
  `!TEST FINISH!` 并主动关机。
- 跳过 `mount` 和 `umount`：它们当前会在 `src/fs/file_system.rs:65`
  的未实现路径触发 kernel panic。

### 本地结果

- 串行暂存并执行 30 个 basic 测试。
- 官方 `test_runner.py`：`91/102`。
- `getdents`：`4/5`，是已执行测试中唯一未满分项。
- `mount`、`umount`：主动跳过，共 10 项暂未得分。
- 无 kernel panic，输出 `!TEST FINISH!` 并主动关机。

## 2026-06-12 官方编译错误修复

### 线上证据

- 官方页面最后一次提交时间为 2026-06-11 19:44:39。
- 失败发生在 Compile 阶段，首先尝试联网下载 `nightly-2025-02-18`，随后因
  `titanix/vendor/spin-0.7.1/Cargo.lock` 被官方隐藏文件过滤移除而校验失败。
- 页面分数为 `0.00`，没有产生任何运行阶段反馈。

### 已完成

- 将工具链固定为官方镜像预装的 `nightly-2025-02-01`。
- 构建流程移除 `rustup target add`、`rustup component add` 和 `cargo install`。
- 新增 `tools/vendor_checksums.py`，按 Git 实际追踪的非隐藏文件重建并检查
  53 个 vendor manifest。
- 保留根 Makefile 从非隐藏备份恢复 `.cargo/` 和 `.cargo-checksum.json` 的流程。
- 在删除全部隐藏文件的干净导出中完成强制离线 `make all`。

### 验证结果

- vendor 校验：53 个 manifest，0 个问题。
- 强制离线构建：通过，日志中无 rustup 同步、组件下载或 crates.io 请求。
- `kernel-rv`、`kernel-la`：均为 RISC-V executable ELF，入口 `0x80200000`。
- 无盘启动：输出 `!TEST FINISH!` 并主动关机。
- `glibc/basic/brk`：官方解析器 `3/3`。
- 外部官方 BusyBox 镜像：60 秒内无 panic，输出允许的
  `official basic script not found` 后主动关机。

### 后续线上结果

- 2026-06-13 19:30:50 的评测状态为 `Accpted`，确认离线工具链和 vendor 修复已
  通过 Compile 阶段；后续 0 分原因已转为运行期动态 loader panic。

## 2026-06-11 首个真实 basic ELF

### 已完成

- 为 `oscomp` 增加 EXT4 普通文件读取，支持 extent tree、直接块和一级间接块。
- 读取 `basic_testcode.sh`，处理 `cd` 和嵌套 `run-all.sh`。
- 解析 `tests="..."` 中的首个测试名，当前得到 `musl/basic/brk`。
- 从 EXT4 读取静态 RISC-V ELF，并校验 ELF magic。
- 将 ELF、argv 和 END 标记写入 Titanix tmpfs。
- 内置 `runtestcase` 检测暂存文件，使用 `fork + execve + wait4` 串行执行。
- 保持真实输出位于 `basic-musl` START/END 区间内。
- QEMU 执行结束后主动关机。

### 验证结果

- `make all`：通过。
- 官方风格 `256M`、单核 QEMU：通过。
- `basic/brk`：完整输出 `START test_brk`、三次堆位置和 `END test_brk`。
- 官方 `test_runner.py`：`test_brk` 共 3 项，全部通过。
- 无测试盘：输出 `oscomp: no staged basic ELF` 后主动关机。

### 当前边界

- 当前跳过 `mount` 和 `umount`，避免未实现挂载路径导致整个评测 panic。
- `getdents` 当前为 `4/5`。
- EXT4 仍未整体挂载进 Titanix VFS，而是按需读取并复制到 tmpfs。
- 线上评测尚未重新确认，不能把本地解析结果视为线上成绩。

## 2026-06-09 Titanix 重写

### 架构决策

- 停止同时扩展旧自建内核和 rCore 迁移版本。
- 使用 Titanix 作为唯一新内核主体。
- 从新主线移除旧 `kernel/`、`user/` 和 `rCore-Tutorial-v3-main/`。
- 旧自建内核的 `basic=102` 成果继续由归档分支保存。

### 已完成

- 从 Titanix `final-submit-qemu` 分支获取内核、用户态和依赖源码。
- 保留 Titanix GPLv3 许可证与来源。
- 将 Windows 不允许检出的 `aux.rs` 改名为 `aux_file.rs`。
- 将工具链从不可下载的 nightly `2022-11-03` 迁移到本机已有的
  nightly `2025-02-18`。
- 修复 PanicInfo、Poll、trap 汇编符号和 virtio-drivers API 兼容问题。
- vendor 全部 Cargo 依赖，构建过程无需连接 crates.io。
- 为 vendor 校验文件建立非隐藏备份，模拟官方隐藏文件过滤后仍可全量构建。
- 根目录 `make all` 已切换为 Titanix 构建。
- 新增 wrapper ELF 流程，把 Titanix 高虚拟地址 raw kernel 封装成物理入口
  `0x80200000` 的官方 `kernel-rv`。
- 官方风格 `256M`、单核 QEMU 启动成功并主动关机。
- 新增 `titanix/kernel/src/oscomp.rs`，通过 Titanix BlockDevice 只读探测 EXT4。
- fixed path 命中 `musl/basic_testcode.sh` 后输出 basic START/END。

## 历史里程碑

- 2026-06-08：旧自建内核官方线上 basic 得分 102。
- 2026-06-08：旧自建内核加入 BusyBox 简单命令队列。
- 2026-06-09：旧自建内核冻结在 `codex/basic-102-archive`。
- 2026-06-09：rCore 迁移尝试能够启动到 shell，但未接入官方 EXT4。
- 2026-06-09：路线切换为 Titanix 完整架构，并完成启动与 basic 入口。
- 2026-06-11：Titanix 执行 `basic/brk`，本地官方解析器得到 `3/3`。

## 下一里程碑

先提交当前 loader 与双组修复，确认 glibc-rv basic 得分非零、musl-rv 至少开始
执行真实测试且全程无 `Panicked`。稳定线上基线后，再按首个 musl 失败日志补 ABI；
`getdents`、`mount/umount` 与 LoongArch 暂不混入本轮。
