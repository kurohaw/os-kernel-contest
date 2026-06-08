TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel
KERNEL_BIN := $(KERNEL_ELF).bin
BOOTLOADER := ../rCore-Tutorial-v3-main/bootloader/rustsbi-qemu.bin
KERNEL_ENTRY_PA := 0x80200000
USER_DIR := ../user
CARGO_CONFIG_SRC := cargo-config/config.toml
CARGO_CONFIG_DST := .cargo/config.toml
TEST_GROUP ?= basic

ifeq ($(TEST_GROUP),basic)
CARGO_FEATURES :=
else ifeq ($(TEST_GROUP),busybox)
CARGO_FEATURES := --features busybox-test
else
$(error unsupported TEST_GROUP "$(TEST_GROUP)"; expected basic or busybox)
endif

OBJCOPY := rust-objcopy --binary-architecture=riscv64

all: build

env:
	rustup target add $(TARGET)
	rustup component add llvm-tools
	cargo install cargo-binutils

prepare-cargo:
	mkdir -p .cargo
	cp $(CARGO_CONFIG_SRC) $(CARGO_CONFIG_DST)

build: prepare-cargo
	$(MAKE) -C $(USER_DIR) build
	cargo build --release $(CARGO_FEATURES)
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

run: build
	qemu-system-riscv64 \
		-machine virt \
		-kernel $(KERNEL_ELF) \
		-m 256M \
		-nographic \
		-smp 1 \
		-bios default \
		-no-reboot

run-loader: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

clean:
	cargo clean

.PHONY: all env prepare-cargo build run run-loader clean
