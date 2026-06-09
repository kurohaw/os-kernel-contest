# 初赛开发进度

## 当前状态

| 项目 | 内容 |
|---|---|
| 当前日期 | 2026-06-09 |
| 当前分支 | `codex/titanix-architecture` |
| 当前内核主体 | `titanix/` |
| 保分归档 | `codex/basic-102-archive`，官方 basic=102 |
| 当前里程碑 | Titanix 已启动并进入官方 EXT4 basic 入口 |
| 当前得分状态 | Titanix 主线尚未执行真实 basic ELF，未恢复分数 |

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

### 当前边界

- 当前 `oscomp` 只负责识别 basic 脚本路径。
- 尚未读取脚本内容。
- 尚未把 EXT4 文件接入 Titanix VFS。
- 尚未从测试盘读取和加载 ELF。
- 当前 START/END 中没有真实 testcase 结果，因此不能期待官方得分。

## 历史里程碑

- 2026-06-08：旧自建内核官方线上 basic 得分 102。
- 2026-06-08：旧自建内核加入 BusyBox 简单命令队列。
- 2026-06-09：旧自建内核冻结在 `codex/basic-102-archive`。
- 2026-06-09：rCore 迁移尝试能够启动到 shell，但未接入官方 EXT4。
- 2026-06-09：路线切换为 Titanix 完整架构，并完成启动与 basic 入口。

## 下一里程碑

让 Titanix 从 EXT4 读取 `basic_testcode.sh` 和脚本引用的第一个 ELF，使用
Titanix `Process/MemorySpace` 创建真实用户进程，并获得第一个可解析 testcase
结果。
