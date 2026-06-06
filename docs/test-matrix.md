# 测试矩阵

## 当前验证命令

```bash
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default -no-reboot
```

当前 smoke test 通过标准：

- 根目录生成 ELF 格式的 `kernel-rv`。
- QEMU 能启动到 `kernel started`。
- `app0` 和 `app1` 都能完成 syscall 验证输出。
- 最后出现 `all tasks exited`，随后 QEMU 主动退出。

官方测试盘扫描验证命令：

```bash
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 256M -nographic -smp 1 -bios default \
  -drive file=/tmp/oskernel-ext4.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot \
  -device virtio-net-device,netdev=net -netdev user,id=net -rtc base=utc
```

本地无分区 EXT4 镜像包含 `basic_testcode.sh` 和 `lua_testcode.sh` 时，内核应输出：

- `virtio-blk: ready`
- `oscomp: found test script basic_testcode.sh`
- `oscomp: found test script lua_testcode.sh`
- `ext4: found 2 test script(s)`

## 官方评测快照

| 时间 | 提交状态 | 总分 | 结论 |
|---|---|---:|---|
| 2026-06-06 12:01 | Accepted | 0.0 | 产物被评测系统接受，但所有官方测试套件均未得分 |

截图中的汇总结果显示，`basic`、`busybox`、`cyclictest`、`iozone`、`iperf`、`libcbench`、`libctest`、`lmbench`、`ltp`、`lua`、`netperf` 在 glibc/musl 与 rv/la 维度下均为 0。

当前判断：这不是单个 syscall 的失败，而是官方测例入口尚未打通。当前内核仍运行内嵌的 `app0/app1` 自测程序，尚未具备加载并调度官方 glibc/musl/busybox 等 Linux ABI 用户程序的完整路径。

优先级应调整为：

| 优先级 | 方向 | 目标 |
|---:|---|---|
| 1 | 官方测例入口 | 已能识别 virtio-blk EXT4 测试盘并列出根目录脚本；下一步读取脚本内容并输出官方组标记 |
| 2 | ELF 与地址空间 | 从固定裸二进制加载过渡到 ELF segment 映射、entry/sp/auxv 初始化 |
| 3 | 进程模型 | 补齐 `execve`、`fork/clone`、`wait4`、`exit_group` 等 basic/busybox 常用路径 |
| 4 | 文件系统接口 | 在现有 EXT4 根目录扫描基础上支持打开/读取测试脚本和 ELF 文件 |
| 5 | syscall 矩阵 | 用官方 basic 失败日志反推最小 syscall 集，而不是只按自测程序扩展 |

## 基础 syscall 状态

| syscall | 编号 | 当前状态 | 验证方式 | 备注 |
|---|---:|---|---|---|
| test | 0 | 已支持 | `app0/app1` 调用 `sys_test` | 自定义测试 syscall |
| exit | 1 | 已支持 | `app0/app1` 正常退出 | 支持退出码打印 |
| yield | 2 | 已支持 | 两个任务轮转运行 | 已修复 yield 后恢复 `TrapContext` |
| openat | 56 | 最小支持 | 打开 `/dev/null`、`/hello.txt` | 不存在路径返回 `-1` |
| close | 57 | 最小支持 | 关闭标准 fd 和动态 fd | 重复关闭动态 fd 返回 `-1` |
| read | 63 | 最小支持 | 读取 `/hello.txt` | stdin 当前返回 `0` |
| write | 64 | 已支持 | stdout/stderr 输出字符串 | `/dev/null` 写入直接丢弃 |
| fstat | 80 | 最小支持 | stdout、动态 fd 返回成功 | stat buffer 当前最小填充 |
| getpid | 172 | 已支持 | app0 返回 0，app1 返回 1 | 当前 pid 等于 task id |
| brk | 214 | 最小支持 | 查询和设置堆边界 | 当前只维护边界，未真实映射堆页 |

## 用户程序验证点

| 测试点 | app0 | app1 | 状态 |
|---|---|---|---|
| write 输出 | 通过 | 通过 | 已验证 |
| getpid | 通过 | 通过 | 已验证 |
| read stdin EOF | 通过 | 通过 | 已验证 |
| brk 查询/设置 | 通过 | 通过 | 已验证 |
| close 标准 fd | 通过 | 通过 | 已验证 |
| close 非法 fd | 通过 | 通过 | 已验证 |
| fstat stdout | 通过 | 通过 | 已验证 |
| fstat 非法 fd | 通过 | 通过 | 已验证 |
| open `/dev/null` | 通过 | 通过 | 已验证 |
| fd 动态分配 | 通过 | 通过 | 已验证 |
| fd 释放与重复关闭 | 通过 | 通过 | 已验证 |
| open `/hello.txt` | 通过 | 通过 | 已验证 |
| read `/hello.txt` | 通过 | 通过 | 已验证 |
| read 文件 EOF | 通过 | 通过 | 已验证 |

## 文件系统验证点

| 路径 | openat | read | write | close | 备注 |
|---|---|---|---|---|---|
| `/dev/null` | 成功 | 返回 0 | 返回写入长度 | 成功 | 动态 fd |
| `/hello.txt` | 成功 | 返回内嵌内容 | 不支持 | 成功 | 只读内嵌文件 |
| `/missing` | 返回 -1 | 不适用 | 不适用 | 不适用 | 不存在路径 |

## 官方测试盘入口状态

| 项目 | 当前状态 | 验证方式 | 备注 |
|---|---|---|---|
| virtio-blk 设备识别 | 已支持 | 官方风格 QEMU `-drive ... virtio-blk-device` | 支持 legacy virtio-mmio v1 |
| 扇区读取 | 已支持 | 初始化时读取 sector 0 smoke test | 512 字节扇区接口 |
| EXT4 superblock | 已支持 | 本地无分区 EXT4 镜像 | 支持 1K/2K/4K block size |
| root inode | 已支持 | 读取 group descriptor 和 inode table | 当前只读 |
| extent 目录块 | 最小支持 | 根目录扫描验证 | 支持 depth=0/1 |
| `*_testcode.sh` 发现 | 已支持 | 本地镜像发现 2 个脚本 | 目前只打印文件名 |
| 脚本内容读取 | 未开始 | 待验证 | 下一步 |
| ELF 文件加载 | 未开始 | 待验证 | 下一步之后 |

## 当前限制

| 模块 | 限制 |
|---|---|
| `brk` | 只维护堆边界，尚未为新增堆区映射用户页 |
| `read` | stdin 暂时直接返回 0 |
| `openat` | 只识别 `/dev/null` 和 `/hello.txt` |
| 文件描述符表 | 当前是全局表，尚未按进程隔离 |
| 文件系统 | 已能只读扫描 EXT4 根目录测试脚本，但 `openat/read` 尚未接入 EXT4 文件内容 |
| 进程模型 | 尚未实现 fork/exec/wait/waitpid |

## 下一步待测

| 方向 | 目标 | 状态 |
|---|---|---|
| 堆内存 | `brk` 增长后真实映射用户页 | 未开始 |
| 官方测例 | 读取 `*_testcode.sh` 内容并输出官方测试组标记 | 下一步 |
| 进程模型 | fork/exec/wait/waitpid | 未开始 |
| 文件系统 | 将 EXT4 文件内容读取接入脚本/ELF loader | 下一步 |
