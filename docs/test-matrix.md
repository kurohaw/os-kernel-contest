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
| `musl/basic_testcode.sh` fixed path | 通过 | 执行 `musl/basic/brk`，解析 `3/3` |
| `glibc/basic_testcode.sh` fixed path | 待单独回归 | 代码已支持 |
| 根目录 `basic_testcode.sh` fixed path | 通过 | 执行 `basic/brk`，解析 `3/3` |
| 读取 basic 脚本内容 | 通过 | 支持 `cd`、嵌套脚本和 `tests="..."` 首项 |
| 从 EXT4 加载 basic ELF | 通过 | `musl/basic/brk` 复制到 tmpfs 后 `execve` |
| argv 传递 | 通过 | NUL 分隔 argv 经 `/oscomp-argv` 传给 runner |
| `test_brk` | 通过 | 官方 `test_runner.py` 解析 `3/3` |
| basic 完整命令队列 | 未实现 | 当前只运行第一个 ELF |
| 无测试盘回归 | 通过 | runner 回退并主动关机 |
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

当前首个 basic ELF 通过标准：

```text
oscomp: found official basic script musl/basic_testcode.sh
oscomp: first basic command musl/basic/brk
#### OS COMP TEST GROUP START basic-musl ####
========== START test_brk ==========
Before alloc,heap pos: ...
After alloc,heap pos: ...
Alloc again,heap pos: ...
========== END test_brk ==========
#### OS COMP TEST GROUP END basic-musl ####
[kernel] kernel will shutdown...
```

## 关键回归风险

| 模块 | 风险 |
|---|---|
| 根 Makefile | 破坏离线依赖恢复或 wrapper ELF |
| `kernel-rv-wrapper.ld` | 入口或 PT_LOAD 不再位于 `0x80200000` |
| `driver/qemu` | x0 virtio block 设备初始化失败 |
| `oscomp.rs` | EXT4 fixed path、extent 读取或脚本解析退化 |
| `runtestcase.rs` | `/oscomp-*` 协议、argv 或 END 标记退化 |
| nightly 升级 | 旧 RISC-V crate、汇编或 async API 再次不兼容 |
| 日志 | basic START/END 被调试输出污染 |
