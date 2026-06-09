RCORE_DIR := rCore-Tutorial-v3-main
RCORE_OS := $(RCORE_DIR)/os
RCORE_USER := $(RCORE_DIR)/user
TARGET_RV := riscv64gc-unknown-none-elf
MODE := release
RCORE_ELF := $(RCORE_OS)/target/$(TARGET_RV)/$(MODE)/os
RCORE_BIN := $(RCORE_ELF).bin
RCORE_BIN_TARGET := target/$(TARGET_RV)/$(MODE)/os.bin
RCORE_FS := $(RCORE_USER)/target/$(TARGET_RV)/$(MODE)/fs.img
KERNEL_RV := kernel-rv
KERNEL_LA := kernel-la
DISK_IMG := disk.img

all: build

prepare-cargo:
	mkdir -p $(RCORE_OS)/.cargo $(RCORE_USER)/.cargo
	cp $(RCORE_OS)/cargo-config/config.toml $(RCORE_OS)/.cargo/config.toml
	cp $(RCORE_USER)/cargo-config/config.toml $(RCORE_USER)/.cargo/config.toml

build: prepare-cargo
	$(MAKE) -C $(RCORE_OS) kernel
	$(MAKE) -C $(RCORE_OS) fs-img
	$(MAKE) -C $(RCORE_OS) $(RCORE_BIN_TARGET)
	cp $(RCORE_ELF) $(KERNEL_RV)
	cp $(RCORE_ELF) $(KERNEL_LA)
	cp $(RCORE_FS) $(DISK_IMG)
	@echo "warning: migration branch uses kernel-rv as a temporary kernel-la placeholder."
	@echo "warning: do not submit this branch until official test execution is restored."

clean:
	$(MAKE) -C $(RCORE_OS) clean
	$(MAKE) -C $(RCORE_USER) clean
	rm -f $(KERNEL_RV) $(KERNEL_LA) $(DISK_IMG)

.PHONY: all prepare-cargo build clean
