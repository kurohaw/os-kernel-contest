# 最小用户态执行计划

## 目标

建立最小用户态闭环：

```text
kernel
-> 构造用户态 TrapContext
-> sret 进入 U-mode
-> 用户态执行 ecall
-> trap 回到 S-mode
-> syscall dispatcher