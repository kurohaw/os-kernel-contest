KERNEL_DIR := kernel
TARGET_RV := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := $(KERNEL_DIR)/target/$(TARGET_RV)/$(MODE)/kernel
KERNEL_BIN := $(KERNEL_ELF).bin
KERNEL_RV := kernel-rv
KERNEL_LA := kernel-la
EXEC_OUT := exec.out

all: build

build: build-rv build-la-placeholder

build-rv:
	$(MAKE) -C $(KERNEL_DIR) build
	cp $(KERNEL_ELF) $(KERNEL_RV)
	cp $(KERNEL_BIN) $(EXEC_OUT)

build-la-placeholder: build-rv
	cp $(KERNEL_RV) $(KERNEL_LA)
	@echo "warning: kernel-la is a temporary placeholder; LoongArch kernel is not implemented yet."

clean:
	$(MAKE) -C $(KERNEL_DIR) clean
	rm -f $(KERNEL_RV) $(KERNEL_LA) $(EXEC_OUT)

.PHONY: all build build-rv build-la-placeholder clean
