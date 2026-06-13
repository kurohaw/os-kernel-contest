# 测试矩阵

## 当前验证

| 项目 | 状态 | 结果 |
|---|---|---|
| 官方页面最后可见结果 | 失败 | 2026-06-11 19:44:39，`0.00 / Compile Error`，属于修复前旧提交 |
| `gitlab/main` 修复提交 | 待线上评测 | `bab4cd0`，官方页面尚未给出对应结果 |
| 根目录 `make all` | 通过 | 离线构建，生成 `kernel-rv`、`kernel-la` |
| 官方同版本 Rust 工具链 | 通过 | `nightly-2025-02-01`，构建日志无联网安装请求 |
| 隐藏文件过滤后 vendor 校验 | 通过 | 删除全部隐藏文件后，53 个 manifest、0 个问题 |
| 隐藏文件过滤后构建 | 通过 | 干净导出删除全部隐藏文件后可自动恢复并全量构建 |
| `kernel-rv` 格式 | 通过 | RISC-V executable ELF，入口 `0x80200000` |
| 官方风格 256M 单核启动 | 通过 | Titanix 启动并主动关机 |
| 无效/空测试盘 | 通过 | 输出无 EXT4 提示，继续运行提交 runner |
| EXT4 superblock | 通过 | 从 x0 virtio-blk 识别无分区 EXT4 |
| `musl/basic_testcode.sh` fixed path | 通过 | 执行 `musl/basic/brk`，解析 `3/3` |
| `glibc/basic_testcode.sh` fixed path | 通过 | 执行 `glibc/basic/brk`，解析 `3/3` |
| 根目录 `basic_testcode.sh` fixed path | 通过 | 执行 `basic/brk`，解析 `3/3` |
| 读取 basic 脚本内容 | 通过 | 支持 `cd`、嵌套脚本和完整 `tests="..."` 队列 |
| 从 EXT4 加载 basic ELF | 通过 | 30 个安全 basic ELF 使用别名复制到 tmpfs 后串行 `execve` |
| basic 依赖资源 | 通过 | 暂存 `test_echo`、`text.txt`，创建 `mnt` |
| `test_brk` | 通过 | 官方 `test_runner.py` 解析 `3/3` |
| 官方 basic 解析器总量 | 已确认 | 32 个测试，共 102 项断言 |
| basic 串行命令队列 | 通过 | 执行 30 个测试，官方解析器 `91/102` |
| `getdents` | 部分通过 | `4/5`，已执行测试中唯一未满分项 |
| `mount`、`umount` | 主动跳过 | 当前会在 `src/fs/file_system.rs:65` 触发 kernel panic |
| 无测试盘回归 | 通过 | runner 回退并主动关机 |
| 外部官方 BusyBox 镜像探针 | 通过 | 无 panic、未超时、主动关机；当前不执行 BusyBox 测试入口 |
| 旧自建内核官方 basic | 历史基线 | 曾取得线上 basic=102 |

未直接运行 `zhouzhouyi/os-contest:20260510` Docker 镜像，因为当前机器没有
Docker CLI；当前通过结果来自同版本官方 nightly、隐藏文件过滤干净导出和
强制离线环境。

## 官方风格命令

```bash
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic \
  -smp 1 -bios default \
  -drive file=/path/to/test.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot
```

当前完整 basic 队列通过标准：

```text
oscomp: skip known unsafe basic command mount
oscomp: skip known unsafe basic command umount
oscomp: staged 30 basic commands
#### OS COMP TEST GROUP START basic-glibc ####
========== START test_brk ==========
========== END test_brk ==========
...
========== END test_yield ==========
#### OS COMP TEST GROUP END basic-glibc ####
 !TEST FINISH!
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
