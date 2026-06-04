TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel
KERNEL_BIN := $(KERNEL_ELF).bin
BOOTLOADER := ../rCore-Tutorial-v3-main/bootloader/rustsbi-qemu.bin
KERNEL_ENTRY_PA := 0x80200000
USER_DIR := ../user

OBJCOPY := rust-objcopy --binary-architecture=riscv64

all: build

env:
	rustup target add $(TARGET)
	rustup component add llvm-tools
	cargo install cargo-binutils

build:
	$(MAKE) -C $(USER_DIR) build
	cargo build --release
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

run: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

clean:
	cargo clean

.PHONY: all env build run clean
