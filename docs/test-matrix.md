# 测试矩阵

## 当前验证命令

```bash
cd /mnt/d/os-kernel-contest/kernel
make build
make run
```

当前 smoke test 通过标准：

- QEMU 能启动到 `kernel started`。
- `app0` 和 `app1` 都能完成 syscall 验证输出。
- 最后出现 `all tasks exited`。

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

## 当前限制

| 模块 | 限制 |
|---|---|
| `brk` | 只维护堆边界，尚未为新增堆区映射用户页 |
| `read` | stdin 暂时直接返回 0 |
| `openat` | 只识别 `/dev/null` 和 `/hello.txt` |
| 文件描述符表 | 当前是全局表，尚未按进程隔离 |
| 文件系统 | 当前只有内嵌只读文件，尚无真实目录和 inode |
| 进程模型 | 尚未实现 fork/exec/wait/waitpid |

## 下一步待测

| 方向 | 目标 | 状态 |
|---|---|---|
| 堆内存 | `brk` 增长后真实映射用户页 | 未开始 |
| 官方测例 | 接入比赛测例并记录通过情况 | 未开始 |
| 进程模型 | fork/exec/wait/waitpid | 未开始 |
| 文件系统 | 扩展更多只读文件和路径处理 | 未开始 |
