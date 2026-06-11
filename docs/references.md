# 参考来源与增量贡献

## 当前基础版本

当前新主线基于 Titanix 往届开源作品：

- 项目：`oskernel2023-Titanix`
- 来源：https://gitlab.eduxiji.net/202318123101314/oskernel2023-Titanix
- 采用分支：`final-submit-qemu`
- 上游提交：`605b408c56cb63a4e2f9b53db62bb6c632f33277`
- 许可证：GNU GPLv3 或更高版本

`titanix/LICENSE`、原作者信息和上游 README 必须保留。队伍还应向指导老师或
组委会确认使用完整往届架构作为比赛基线的复用边界。

## 当前队伍增量

| 时间 | 内容 |
|---|---|
| 2026-06-09 | 将 Titanix 导入 Windows 工作区并处理 `aux.rs` 保留名冲突 |
| 2026-06-09 | 将工具链迁移到 nightly `2025-02-18` |
| 2026-06-09 | 修复新 nightly、Poll、PanicInfo、trap 和 virtio API 兼容问题 |
| 2026-06-09 | vendor Cargo 依赖，建立离线构建 |
| 2026-06-09 | 重写根 Makefile，生成官方可加载的 wrapper ELF |
| 2026-06-09 | 新增只读 EXT4 `oscomp` 适配层和 basic fixed path 入口 |

## 其他参考

- rCore-Tutorial-v3：理解 Rust/RISC-V 内核基础。
- 旧自建内核：仅通过 `codex/basic-102-archive` 参考官方 EXT4 和 ABI 经验。
- Phoenix、Starry、NoAxiom：仅参考设计和比赛推进方法，不直接复制代码。

## 贡献记录原则

- 每个阶段更新 `docs/progress.md` 和 `docs/test-matrix.md`。
- 明确区分 Titanix 上游能力与本队新增适配。
- 保留所有第三方许可证。
- 队员应能够解释实际提交代码和架构设计。
