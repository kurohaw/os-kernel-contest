KERNEL_DIR := kernel
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_BIN := $(KERNEL_DIR)/target/$(TARGET)/$(MODE)/kernel.bin
EXEC_OUT := exec.out

all: build

build:
	$(MAKE) -C $(KERNEL_DIR) build
	cp $(KERNEL_BIN) $(EXEC_OUT)

clean:
	$(MAKE) -C $(KERNEL_DIR) clean
	rm -f $(EXEC_OUT)

.PHONY: all build clean
