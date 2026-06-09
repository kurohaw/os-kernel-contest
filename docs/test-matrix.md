# 测试矩阵

## 当前验证

| 项目 | 状态 | 结果 |
|---|---|---|
| 根目录 `make all` | 通过 | 离线构建，生成 `kernel-rv`、`kernel-la` |
| 隐藏文件过滤后构建 | 通过 | 删除 `.cargo/` 和 `.cargo-checksum.json` 后可自动恢复并全量构建 |
| `kernel-rv` 格式 | 通过 | RISC-V executable ELF，入口 `0x80200000` |
| 官方风格 256M 单核启动 | 通过 | Titanix 启动并主动关机 |
| 无效/空测试盘 | 通过 | 输出无 EXT4 提示，继续运行提交 runner |
| EXT4 superblock | 通过 | 从 x0 virtio-blk 识别无分区 EXT4 |
| `musl/basic_testcode.sh` fixed path | 通过 | 输出 basic START/END |
| `glibc/basic_testcode.sh` fixed path | 待单独回归 | 代码已支持 |
| 根目录 `basic_testcode.sh` fixed path | 待单独回归 | 代码已支持 |
| 读取 basic 脚本内容 | 未实现 | 下一步 |
| 从 EXT4 加载 basic ELF | 未实现 | 下一步 |
| Titanix 主线真实 basic testcase | 未实现 | 当前无分数预期 |
| 旧自建内核官方 basic | 已归档 | `codex/basic-102-archive` 线上 basic=102 |

## 官方风格命令

```bash
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic \
  -smp 1 -bios default \
  -drive file=/path/to/test.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot
```

当前 EXT4 basic 入口通过标准：

```text
oscomp: found official basic script musl/basic_testcode.sh
#### OS COMP TEST GROUP START basic ####
oscomp: Titanix official basic entry reached
#### OS COMP TEST GROUP END basic ####
[kernel] kernel will shutdown...
```

## 关键回归风险

| 模块 | 风险 |
|---|---|
| 根 Makefile | 破坏离线依赖恢复或 wrapper ELF |
| `kernel-rv-wrapper.ld` | 入口或 PT_LOAD 不再位于 `0x80200000` |
| `driver/qemu` | x0 virtio block 设备初始化失败 |
| `oscomp.rs` | EXT4 fixed path 无法命中 |
| nightly 升级 | 旧 RISC-V crate、汇编或 async API 再次不兼容 |
| 日志 | basic START/END 被调试输出污染 |
