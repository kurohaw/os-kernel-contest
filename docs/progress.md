# 初赛开发进度

## 当前状态

| 项目 | 内容 |
|---|---|
| 当前日期 | 2026-06-11 |
| 当前分支 | `codex/titanix-architecture` |
| 当前内核主体 | `titanix/` |
| 保分归档 | `codex/basic-102-archive`，官方 basic=102 |
| 当前里程碑 | Titanix 已执行官方 EXT4 中的首个 basic ELF `brk` |
| 当前得分状态 | 本地官方解析器 `test_brk=3/3`，线上分数待评测确认 |

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

- 只执行 `run-all.sh` 中的第一个测试 `brk`。
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

把 `run-all.sh` 的完整测试列表变成串行命令队列，保持 `brk` 回归通过，并
推进到下一个真实失败项。
