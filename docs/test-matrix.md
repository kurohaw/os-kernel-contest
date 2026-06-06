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
- 脚本内或 fallback 生成的 `#### OS COMP TEST GROUP START ... ####`
- 脚本内或 fallback 生成的 `#### OS COMP TEST GROUP END ... ####`
- `ext4: found 2 test script(s)`

## 官方评测快照

| 时间 | 提交状态 | 总分 | 结论 |
|---|---|---:|---|
| 2026-06-06 12:01 | Accepted | 0.0 | 产物被评测系统接受，但所有官方测试套件均未得分 |

截图中的汇总结果显示，`basic`、`busybox`、`cyclictest`、`iozone`、`iperf`、`libcbench`、`libctest`、`lmbench`、`ltp`、`lua`、`netperf` 在 glibc/musl 与 rv/la 维度下均为 0。

当前判断：0 分主要来自官方测例入口和 Linux ABI 尚未完整打通。当前内核已经可以从测试盘读取根目录脚本、定位脚本中的根目录 ELF、映射 `PT_LOAD` 并进入外部用户程序，但尚未具备运行官方 glibc/musl/busybox 等 Linux ABI 用户程序的完整路径。

优先级应调整为：

| 优先级 | 方向 | 目标 |
|---:|---|---|
| 1 | 官方测例入口 | 已能识别 virtio-blk EXT4 测试盘、读取根目录脚本、输出官方组标记，并从脚本定位根目录 ELF |
| 2 | ELF 与地址空间 | 已从固定裸二进制过渡到 ELF segment 映射、entry 和最小启动栈初始化 |
| 3 | 进程模型 | 补齐 `execve`、`fork/clone`、`wait4`、`exit_group` 等 basic/busybox 常用路径 |
| 4 | 文件系统接口 | 已在现有 EXT4 根目录扫描基础上支持用户态打开/读取根目录普通文件 |
| 5 | syscall 矩阵 | 用官方 basic 失败日志反推最小 syscall 集，而不是只按自测程序扩展 |

## 基础 syscall 状态

| syscall | 编号 | 当前状态 | 验证方式 | 备注 |
|---|---:|---|---|---|
| test | 0 | 已支持 | `app0/app1` 调用 `sys_test` | 自定义测试 syscall |
| exit | 1 | 已支持 | `app0/app1` 正常退出 | 支持退出码打印 |
| yield | 2 | 已支持 | 两个任务轮转运行 | 已修复 yield 后恢复 `TrapContext` |
| openat | 56 | 最小支持 | 打开 `/dev/null`、`/hello.txt`、EXT4 普通文件 | 支持多级只读路径，不存在路径返回 `-1` |
| close | 57 | 最小支持 | 关闭标准 fd 和动态 fd | 重复关闭动态 fd 返回 `-1` |
| read | 63 | 最小支持 | 读取 `/hello.txt` | stdin 当前返回 `0` |
| write | 64 | 已支持 | stdout/stderr 输出字符串 | `/dev/null` 写入直接丢弃 |
| fstat | 80 | 最小支持 | stdout、动态 fd 返回成功 | stat buffer 当前最小填充 |
| getpid | 172 | 已支持 | app0 返回 0，app1 返回 1 | 当前 pid 等于 task id |
| brk | 214 | 最小支持 | 查询、增长堆边界并写入新增页 | 增长时映射零页，缩小时暂不回收 |

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
| 测试盘普通文件 | 成功 | 按 fd offset 读取 EXT4 内容 | 不支持 | 成功 | 本地 `ext4read` 读取 `data.txt`、`subpath` 读取 `dir/data.txt` 通过 |

## 官方测试盘入口状态

| 项目 | 当前状态 | 验证方式 | 备注 |
|---|---|---|---|
| virtio-blk 设备识别 | 已支持 | 官方风格 QEMU `-drive ... virtio-blk-device` | 支持 legacy virtio-mmio v1 |
| 扇区读取 | 已支持 | 初始化时读取 sector 0 smoke test | 512 字节扇区接口 |
| EXT4 superblock | 已支持 | 本地无分区 EXT4 镜像 | 支持 1K/2K/4K block size |
| root inode | 已支持 | 读取 group descriptor 和 inode table | 当前只读 |
| extent 目录块 | 最小支持 | 根目录扫描验证 | 支持 depth=0/1 |
| `*_testcode.sh` 发现 | 已支持 | 本地镜像发现 2 个脚本 | 打印文件名并继续读取脚本内容 |
| 脚本内容读取 | 已支持 | 本地镜像读取 `basic_testcode.sh` 和 `lua_testcode.sh` | 当前最多读取 16 KiB |
| 官方组标记输出 | 已支持 | 脚本内 marker 原样输出；无 marker 时按文件名前缀生成 START/END | 暂时跳过测试组执行 |
| ELF 文件加载 | 最小支持 | 本地 EXT4 镜像加载根目录 `app0` ELF 并运行 | 支持 ELF64 `PT_LOAD`，暂用 4 MiB 缓冲 |
| ELF 输出包裹 | 已支持 | START 在外部 ELF 前输出，END 在 task exit 后输出 | 单外部 ELF 路径 |
| 外部 ELF 启动栈 | 最小支持 | 本地 EXT4 镜像加载 `app0` ELF 后正常运行 | `argc=1`、`argv[0]`、空 `envp`、基础 `auxv` |
| EXT4 syscall 读取 | 最小支持 | 临时外部 ELF `ext4read` 打开并读取根目录 `data.txt` | 只读根目录普通文件 |
| EXT4 子目录路径 | 最小支持 | 临时外部 ELF `subpath` 打开并读取 `dir/data.txt` | 只读普通文件 |
| 脚本下钻和 argv | 最小支持 | 顶层脚本 `busybox echo` + `cd ./basic` + 嵌套 `./run-all.sh`，最终运行 `basic/argshow one two` | 只定位第一个真实 ELF |

## 当前限制

| 模块 | 限制 |
|---|---|
| `brk` | 增长时映射用户零页；缩小时暂不回收页 |
| `read` | stdin 暂时直接返回 0 |
| `openat` | 识别 `/dev/null`、`/hello.txt` 和 EXT4 多级普通文件；尚未支持目录 fd 和挂载点语义 |
| 文件描述符表 | 当前是全局表，尚未按进程隔离 |
| 文件系统 | `openat/read/fstat` 已能读取 EXT4 多级普通文件；尚未支持写入、目录 fd、挂载点和完整 Linux 路径语义 |
| 进程模型 | 尚未实现 fork/exec/wait/waitpid |
| ELF loader | 已支持脚本命令 argv、空 `envp` 和基础 `auxv`；尚未支持动态链接器、解释器路径、多个 ELF 串行运行 |

## 下一步待测

| 方向 | 目标 | 状态 |
|---|---|---|
| 堆内存 | `brk` 增长后真实映射用户页 | 已完成（增长映射） |
| 官方测例 | 用官方 basic/busybox ELF 运行日志反推缺失 syscall | 下一步 |
| 进程模型 | 多 ELF 串行、fork/exec/wait/waitpid | 下一步 |
| 文件系统 | 支持目录 fd、挂载点和更多 pseudo path | 下一步 |
