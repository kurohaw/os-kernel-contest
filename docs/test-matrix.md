# 测试矩阵

## 当前验证

| 项目 | 状态 | 结果 |
|---|---|---|
| 官方页面最后可见结果 | 编译错误已定位并修复 | 2026-06-19 19:09:49，`Compile Error / 0.00`；官方离线 Cargo 解析不到 `hashbrown` 拉入的 `ahash` |
| 上一条通过基线 | 通过并得分 | 2026-06-19 17:00:35，`Accepted / 377.200790558321`；basic=204、BusyBox=98、Lua=18、libcbench=57.2007905583205 |
| 上一条编译错误 | 已修复 | 2026-06-19 19:09:49，`Compile Error / 0.00`；`no matching package found: ahash`，本轮移除内核 `hashbrown` 依赖链 |
| 上一条高分结果 | 通过并得分 | 2026-06-18 09:46:55，`Accepted / 377.3228370332187`；libcbench glibc-rv=30.15271484677692、musl-rv=27.170122186441827 |
| iozone 回归结果 | 已止血 | 2026-06-18 16:00:21，`Accepted / 320.0`；libcbench=0、iozone=0；已撤回 `b10e9f0` |
| musl-rv basic | 通过 | 线上 `102/102` |
| RISC-V BusyBox | 通过并得分 | 线上 glibc-rv=49、musl-rv=49 |
| RISC-V Lua | 通过并得分 | 线上 glibc-rv=9、musl-rv=9 |
| 根目录 `make all` | 通过 | 移除 `hashbrown` 后，强制离线构建生成 `kernel-rv`、`kernel-la` |
| 官方同版本 Rust 工具链 | 通过 | `nightly-2025-02-18`，构建日志无联网安装请求 |
| vendor checksum | 已本地修复 | `tools/vendor_checksums.py --check` 为 53 个 manifest、0 个问题 |
| `managed` path/patch | 本地通过 | 直接依赖和 crates.io patch 均指向 `SWTC/vendor/managed-0.8.0`，Cargo.lock 不再记录其 registry source |
| `hashbrown/ahash` 依赖链 | 已移除 | inode 缓存改用 `BTreeMap`，`Cargo.lock` 不再出现 `hashbrown`、`ahash`、`allocator-api2` |
| 隐藏文件过滤后构建 | 通过 | 干净导出删除全部隐藏文件后可自动恢复并全量构建 |
| `kernel-rv` 格式 | 通过 | RISC-V executable ELF，入口 `0x80200000` |
| 官方完整 1G 单核启动参数 | 通过 | 含网络设备与 RTC；SWTC 启动并主动关机 |
| 无效/空测试盘 | 通过 | 输出无 EXT4 提示，继续运行提交 runner |
| EXT4 superblock | 通过 | 从 x0 virtio-blk 识别无分区 EXT4 |
| `musl/basic_testcode.sh` fixed path | 通过 | 与 glibc 同时存在时排在 glibc 后执行 |
| `glibc/basic_testcode.sh` fixed path | 通过 | 与 musl 同时存在时优先执行 |
| 根目录 `basic_testcode.sh` fixed path | 通过 | 执行 `basic/brk`，解析 `3/3` |
| 读取 basic 脚本内容 | 通过 | 支持 `cd`、嵌套脚本和完整 `tests="..."` 队列 |
| 从 EXT4 加载 basic ELF | 通过 | 每组使用私有 tmpfs 目录，双组镜像串行执行 64 个命令 |
| basic 依赖资源 | 通过 | 每组独立暂存 `test_echo`、`text.txt`，创建 `mnt` |
| `G/X/E` 双组队列协议 | 通过 | 依次输出 glibc、musl START/END，结束后统一关机 |
| `A` 带 argv 队列协议 | 本地通过 | 支持 `A<timeout_ms>\t<argv0>\t<arg1>...`，用于小批量直接执行带参数 ELF |
| 队列文件读取 | 本地通过 | 从固定 4 KiB 改为分块读取，上限 64 KiB |
| 子进程超时保护 | 本地通过 | `A` 记录使用 `wait4(WNOHANG)` 轮询，超时后 `kill(SIGKILL)` 并继续 |
| 动态解释器缺失 | 通过 | 返回 `ENOENT/ENOEXEC`，不再在 `memory_space/mod.rs:871` panic |
| glibc 动态 ELF 探针 | 通过 | 暂存私有 loader/libc 后成功进入动态程序 `main` |
| 动态 ELF `PT_INTERP` 解析 | 通过 | 从 ELF 读取真实解释器路径，并按组私有目录创建完整匹配路径 |
| 损坏动态 loader 探针 | 通过 | 安全 ELF 布局校验返回失败，runner 继续并主动关机 |
| execve errno 诊断 | 通过 | 损坏 loader 输出 `execve ... failed: -8` |
| 未知扩展 program header | 通过 | loader 跳过未使用类型，动态 ELF 仍进入 `main` |
| 缺少 musl 运行时故障注入 | 通过 | 跳过 musl 动态组，已暂存的 glibc 组完整执行并主动关机 |
| `test_brk` | 通过 | 官方 `test_runner.py` 解析 `3/3` |
| 官方 basic 解析器总量 | 已确认 | 32 个测试，共 102 项断言 |
| basic 串行命令队列 | 通过 | 双组官方布局镜像完整运行，官方解析器 `102/102` |
| `getdents` | 通过 | 本地官方解析器 `5/5` |
| `mount`、`umount` | 通过 | 本地官方解析器均 `5/5`，线上 basic 已满分 |
| 无测试盘回归 | 通过 | runner 回退并主动关机 |
| 外部官方 BusyBox 镜像探针 | 通过 | 线上 BusyBox glibc/musl 均 `49/49` |
| Lua staging | 通过并得分 | 线上 Lua glibc/musl 均 `9/9` |
| libcbench staging | 通过并部分得分 | 最新结果为 glibc-rv=30.12274508733359、musl-rv=27.07823396849846 |
| futex bitset | 已线上验证有增益 | libcbench 曾从 `6.0` 提升到 `57.32283703321875` 总分 |
| lmbench-lite staging | 线上仍 0，继续修正 | 识别 glibc/musl `lmbench_testcode.sh`，只执行 `lat_syscall null/read/write/stat/fstat/open`；本轮改用官方 marker、`lat_syscall` argv0 和 readlinkat 长度返回 |
| iozone staging | 已撤回 | `b10e9f0` 后线上回退到 `320.0`，当前先恢复 libcbench 基线 |
| 旧自建内核官方 basic | 历史基线 | 曾取得线上 basic=102 |

未直接运行 `zhouzhouyi/os-contest:20260510` Docker 镜像，因为当前机器没有
Docker CLI；当前通过结果来自同版本官方 nightly、隐藏文件过滤干净导出和
强制离线环境。

## 官方风格命令

```bash
cd /mnt/d/os-kernel-contest
make all
qemu-system-riscv64 -machine virt -kernel kernel-rv -m 1G -nographic \
  -smp 1 -bios default \
  -drive file=/path/to/test.img,if=none,format=raw,id=x0 \
  -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -no-reboot \
  -device virtio-net-device,netdev=net -netdev user,id=net -rtc base=utc
```

当前完整 basic 队列通过标准：

```text
oscomp: staged 2 test groups with 64 commands
#### OS COMP TEST GROUP START basic-glibc ####
========== START test_brk ==========
========== END test_brk ==========
...
========== END test_yield ==========
#### OS COMP TEST GROUP END basic-musl ####
 !TEST FINISH!
[kernel] kernel will shutdown...
```

## 关键回归风险

| 模块 | 风险 |
|---|---|
| 根 Makefile | 破坏离线依赖恢复或 wrapper ELF |
| `rust-toolchain.toml` | 使用官方未预装版本会触发无网络下载并直接编译失败 |
| vendor checksum | 引用隐藏或未跟踪文件会在官方过滤后报文件不存在 |
| `managed` dependency | 若走 directory source 解析，官方可能再次报 `no matching package named managed found` |
| `kernel-rv-wrapper.ld` | 入口或 PT_LOAD 不再位于 `0x80200000` |
| `driver/qemu` | x0 virtio block 设备初始化失败 |
| `oscomp.rs` | EXT4 fixed path、extent 读取或脚本解析退化 |
| `runtestcase.rs` | `G/X/E` 队列、工作目录切换、argv 或 END 标记退化 |
| `A` 队列记录 | 超时轮询、kill 或 argv 构造错误会拖死后续组 |
| lmbench-lite | 真实官方 `lmbench_all` 资源路径或参数不匹配可能导致 0 分，但不得影响现有 8 组 |
| 动态 loader | 缺失/无效解释器重新触发 panic，或组间 libc 相互覆盖 |
| nightly 升级 | 旧 RISC-V crate、汇编或 async API 再次不兼容 |
| 日志 | basic START/END 被调试输出污染 |
