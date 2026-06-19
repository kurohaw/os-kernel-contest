SWTC_DIR := SWTC
SWTC_KERNEL := $(SWTC_DIR)/kernel
SWTC_USER := $(SWTC_DIR)/user
TARGET_RV := riscv64gc-unknown-none-elf
RUST_TOOLCHAIN := nightly-2025-02-01
MODE := release
SWTC_ELF := $(SWTC_KERNEL)/target/$(TARGET_RV)/$(MODE)/kernel
SWTC_BIN := $(SWTC_ELF).bin
WRAPPER_OBJ := $(SWTC_KERNEL)/target/$(TARGET_RV)/$(MODE)/kernel-rv-wrapper.o
WRAPPER_LD := $(SWTC_KERNEL)/kernel-rv-wrapper.ld
KERNEL_RV := kernel-rv
KERNEL_LA := kernel-la

export RUSTUP_TOOLCHAIN := $(RUST_TOOLCHAIN)

RUST_SYSROOT = $(shell rustc --print sysroot)
RUST_HOST = $(shell rustc -vV | sed -n 's/^host: //p')
RUST_LLD := $(RUST_SYSROOT)/lib/rustlib/$(RUST_HOST)/bin/rust-lld

all: build

check-tools:
	@command -v rustc >/dev/null || { echo "error: rustc is required"; exit 1; }
	@command -v cargo >/dev/null || { echo "error: cargo is required"; exit 1; }
	@command -v rust-objcopy >/dev/null || { echo "error: rust-objcopy is required"; exit 1; }
	@test -d "$(RUST_SYSROOT)/lib/rustlib/$(TARGET_RV)/lib" || { \
		echo "error: Rust target $(TARGET_RV) is not installed for $(RUST_TOOLCHAIN)"; exit 1; }
	@test -x "$(RUST_LLD)" || { \
		echo "error: rust-lld is not installed for $(RUST_TOOLCHAIN)"; exit 1; }

restore-vendor:
	find $(SWTC_DIR)/vendor -name cargo-checksum.json -exec sh -c \
		'cp "$$1" "$$(dirname "$$1")/.cargo-checksum.json"' _ {} \;

prepare-cargo: check-tools restore-vendor
	mkdir -p $(SWTC_KERNEL)/.cargo $(SWTC_USER)/.cargo
	cp $(SWTC_KERNEL)/cargo-config/config.toml $(SWTC_KERNEL)/.cargo/config.toml
	cp $(SWTC_USER)/cargo-config/config.toml $(SWTC_USER)/.cargo/config.toml

build: prepare-cargo
	$(MAKE) -C $(SWTC_USER) build PRELIMINARY=0
	$(MAKE) -C $(SWTC_KERNEL) kernel TMPFS=1 SUBMIT=1
	rust-objcopy $(SWTC_ELF) --strip-all -O binary $(SWTC_BIN)
	cd $(dir $(SWTC_BIN)) && rust-objcopy -I binary -O elf64-littleriscv \
		--binary-architecture=riscv64 \
		--rename-section .data=.text,alloc,load,readonly,code,contents \
		$(notdir $(SWTC_BIN)) $(notdir $(WRAPPER_OBJ))
	$(RUST_LLD) -flavor gnu -m elf64lriscv -T $(WRAPPER_LD) \
		-o $(KERNEL_RV) $(WRAPPER_OBJ)
	cp $(KERNEL_RV) $(KERNEL_LA)
	@echo "warning: kernel-la is a temporary placeholder; LoongArch is not implemented."

clean:
	$(MAKE) -C $(SWTC_KERNEL) clean
	$(MAKE) -C $(SWTC_USER) clean
	rm -f $(KERNEL_RV) $(KERNEL_LA)

.PHONY: all check-tools restore-vendor prepare-cargo build clean
