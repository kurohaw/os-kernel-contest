# Progress
## 2026-05-18

### 今日目标
运行 rCore baseline。

### 命令
make run

### 结果
成功 

观察到：

RustSBI-QEMU 启动成功
内核进入初始化流程
GPU、keyboard、mouse 初始化成功
trap 初始化成功
检测到 block device
成功进入 Rust user shell

### 下一步
进入 boot 阅读。