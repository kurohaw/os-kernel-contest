# 下一轮 LTP 大批量候选

本文记录 LTP 提分清单和启用状态。2026-06-29 已在批次 A 主体之后继续启用
批次 B 的 fd/pipe/vector-I/O 子集、批次 C 的进程/信号/时间子集，以及批次
D 中已有 syscall 支撑的文件操作子集；后续不要重复加入这些 case，应根据
官方结果决定回滚、拆分或继续推进剩余 D 批。

## 当前基线

当前 RISC-V/LoongArch 路径里已经放入的 LTP allowlist：

```text
alarm02 chown01 close01 close02
dup01 dup02 dup03 dup04 dup06 dup07 dup202 dup204 dup206 dup207
exit02 exit_group01 fork01 fork03 fork07 fork08 fork10
getcwd01 getegid02 geteuid01 getgid03 getpid02 getppid02
gettimeofday01 gettimeofday02 getuid01 lseek01 lseek07 uname01 uname04
mkdir05 mkdirat01 pipe01 pipe06 pipe10 pipe11 pipe14 readv01 rmdir01
```

线上已证明 RISC-V LTP 可稳定得到 `155`。LoongArch LTP 是否开始计分要等
当前评测结果确认。

## 禁止直接加入

这些项已经有明确风险或收益不足，不能进入下一轮大批量：

```text
readv02
fork05
iozone/cyclictest/network 相关项
```

- `readv02` 曾触发 `src/fs/file.rs` panic。
- `fork05` 退出成功但没有有效 TPASS，不计分。
- iozone、cyclictest、iperf/netperf 不属于 LTP 扩容批次，曾造成 320/838 回退。

## 批次 A：低风险 syscall 和文件元数据

目的：先用大量短 case 验证输出格式、timeout 和进程回收，目标是从 LTP
当前 `155` 向 `300+` 推进。

```text
access01 access02 access03 access04
faccessat01 faccessat02 faccessat201 faccessat202
chmod01 chmod03 chmod05 chmod06 chmod07
fchmod01 fchmod02 fchmod03 fchmod04 fchmod05 fchmod06 fchmodat01 fchmodat02
chown02 chown03 chown04 chown05
fchown01 fchown02 fchown03 fchown04 fchown05 fchownat01 fchownat02
getrlimit01 getrlimit02 getrlimit03
getrusage01 getrusage02 getrusage03 getrusage04
gettid01 gettid02
getrandom01 getrandom02 getrandom03 getrandom04 getrandom05
eventfd01 eventfd02 eventfd03 eventfd04 eventfd05 eventfd06
eventfd2_01 eventfd2_02 eventfd2_03
dup05 dup201 dup203 dup205 dup3_01 dup3_02
openat01 openat02 openat03 openat04 openat201 openat202 openat203
```

注意：

- `openat02_child`、`getrusage03_child` 这类 helper 不直接加入。
- `*_16` 兼容旧 16-bit uid/gid 的 case 暂缓，避免一次引入 ABI 分叉。

## 批次 B：fd、pipe、vector I/O 和锁

目的：吃掉 pipe/readv/writev/fcntl 类分数。该批比 A 风险高，建议 A 在线上
确认后再上。

状态：2026-06-29 已启用其中的 `fcntl*`、`pipe*`、`pipe2_*`、`writev*`、
`preadv*`、`pwritev*`、`pwrite02*`、`poll01/02`、`pselect*`、
`select01-04`。仍未启用 `flock*`，也没有恢复禁止项 `readv02`。

```text
fcntl01 fcntl01_64 fcntl02 fcntl02_64 fcntl03 fcntl03_64
fcntl04 fcntl04_64 fcntl05 fcntl05_64 fcntl07 fcntl07_64
fcntl08 fcntl08_64 fcntl09 fcntl09_64 fcntl10 fcntl10_64
fcntl11 fcntl11_64 fcntl12 fcntl12_64 fcntl13 fcntl13_64
flock01 flock02 flock03 flock04 flock06
pipe02 pipe03 pipe04 pipe05 pipe07 pipe08 pipe09 pipe12 pipe13 pipe15
pipe2_01 pipe2_02 pipe2_04
writev01 writev02 writev03 writev05 writev06 writev07
preadv01 preadv01_64 preadv02 preadv02_64 preadv03 preadv03_64
pwritev01 pwritev01_64 pwritev02 pwritev02_64 pwritev03 pwritev03_64
pwrite02 pwrite02_64
poll01 poll02
pselect01 pselect01_64 pselect02 pselect02_64 pselect03 pselect03_64
select01 select02 select03 select04
```

注意：

- `readv02` 仍禁止加入。
- 若该批出现卡死，优先缩回 `fcntl*` 和 `pselect*`，保留 pipe/writev。

## 批次 C：进程、信号、时间

目的：补进程控制和时间类 LTP。该批可能暴露 wait/signal/nanosleep 的边界，
建议放在 A/B 后。

状态：2026-06-29 已启用其中的 `alarm03/05/06/07`、`nanosleep01/02/04`、
`kill02/03/05-13`、`waitpid01/03/04/06-13` 和 `fork04/09/13/14`。
`timerfd*` 暂不启用，因为 RISC-V 主线尚未提供 timerfd fd 对象实现。

```text
alarm03 alarm05 alarm06 alarm07
nanosleep01 nanosleep02 nanosleep04
timerfd_create01 timerfd_gettime01 timerfd_settime01 timerfd_settime02
timerfd01 timerfd02 timerfd04
kill02 kill03 kill05 kill06 kill07 kill08 kill09 kill10 kill11 kill12 kill13
waitpid01 waitpid03 waitpid04 waitpid06 waitpid07 waitpid08
waitpid09 waitpid10 waitpid11 waitpid12 waitpid13
fork04 fork09 fork13 fork14
```

注意：

- `fork05` 不加入。
- 如果日志出现 `wait4` 被信号打断、后代进程串扰下一项，先修进程回收再扩容。

## 批次 D：目录、链接、stat、truncate、sendfile

目的：扩大文件系统覆盖面，靠 TPASS 数量继续抬 LTP。

```text
creat01 creat03 creat04 creat05 creat06 creat07 creat08 creat09
link02 link04 link05 link08 linkat01 linkat02
symlink01 symlink02 symlink03 symlink04 symlinkat01
unlink05 unlink07 unlink08 unlink09 unlinkat01
rename01 rename03 rename04 rename05 rename06 rename07 rename08
rename09 rename10 rename11 rename12 rename13 rename14
renameat01 renameat201 renameat202
statx01 statx02 statx03 statx04 statx05 statx06
statx07 statx08 statx09 statx10 statx11 statx12
truncate02 truncate02_64 truncate03 truncate03_64
utime01 utime02 utime03 utime04 utime05 utime06 utime07
utimensat01 utimes01
sendfile02 sendfile02_64 sendfile03 sendfile03_64
sendfile04 sendfile04_64 sendfile05 sendfile05_64
sendfile06 sendfile06_64 sendfile07 sendfile07_64
sendfile08 sendfile08_64 sendfile09 sendfile09_64
```

注意：

- 这批对 VFS 语义更敏感，要在 A/B/C 稳定后加入。
- `rename*`、`link*`、`symlink*` 若出现权限或路径失败，优先单独拆分调试。

状态：2026-06-29 已启用 D 批中的 `unlink*`、`rename*`、`renameat*`、
`truncate*`、`utimensat01` 和 `sendfile*` 子集。RISC-V 同步补齐
`renameat=38`、`truncate=45`，并修正 `utimensat(NULL)` 和 `sendfile`
大块复制路径。`creat*`、`link*`、`symlink*`、`statx*`、`utime*` 和
`utimes01` 暂缓，等 RISC-V 主线补齐对应 syscall 或单项验证后再加入。

## 如何启用

RISC-V 路径：

1. 修改 `SWTC/kernel/src/oscomp.rs` 的 `LTP_ALLOWLIST`。
2. 每次只追加一个批次，或者从一个批次中挑 40-80 个先跑。
3. 保持 `LTP_TIMEOUT_MS` 和 `L` 队列输出格式不变。

LoongArch 路径：

1. 同步修改 `SWTC-la/src/init.sh` 的 `run_ltp_subset` case 列表。
2. 保持每个 case 使用 `/musl/busybox timeout 5 "./$case_name"`。
3. 无论退出码是否为 0，都输出 `FAIL LTP CASE name : status`，这和
   RISC-V `runtestcase` 已线上计分的 LTP 收尾协议一致。

本地日志分析：

```bash
python3 tools/analyze_ltp_log.py /path/to/qemu.log --format safe
```

只有满足下面全部条件的 case 才能进入正式提交：

- 至少一个 `TPASS`；
- `TFAIL=0`；
- `TBROK=0`；
- status 为 `0`；
- 无 timeout；
- 无 kernel panic；
- START/END marker 完整，最终主动关机。

## 等待当前评测时的分叉

| 当前结果 | LTP 动作 |
|---|---|
| `< 900` | 不扩 LTP，先修本轮截断或 panic |
| `900-1100` | 先启用批次 A 的前 40-80 个 |
| `1100-1400` | 批次 A 全量 + 批次 B 子集 |
| `> 1400` | A/B/C 并行筛，本地通过后下一轮目标 +300 以上 |
