SWTC_DIR := SWTC
SWTC_KERNEL := $(SWTC_DIR)/kernel
SWTC_USER := $(SWTC_DIR)/user
SWTC_LA := SWTC-la
TARGET_RV := riscv64gc-unknown-none-elf
TARGET_LA := loongarch64-unknown-none
RV_TOOLCHAIN ?= nightly-2025-02-01
LA_TOOLCHAIN ?= nightly-2025-02-18
MODE := release
SWTC_ELF := $(SWTC_KERNEL)/target/$(TARGET_RV)/$(MODE)/kernel
SWTC_BIN := $(SWTC_ELF).bin
WRAPPER_OBJ := $(SWTC_KERNEL)/target/$(TARGET_RV)/$(MODE)/kernel-rv-wrapper.o
WRAPPER_LD := $(SWTC_KERNEL)/kernel-rv-wrapper.ld
KERNEL_RV := kernel-rv
KERNEL_LA := kernel-la
SWTC_LA_ELF := $(SWTC_LA)/SWTC-la_loongarch64-qemu-virt.elf

RV_RUST_SYSROOT = $(shell RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) rustc --print sysroot)
RV_RUST_HOST = $(shell RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) rustc -vV | sed -n 's/^host: //p')
RV_RUST_LLD = $(RV_RUST_SYSROOT)/lib/rustlib/$(RV_RUST_HOST)/bin/rust-lld

all: build

check-rv-tools:
	@command -v rustc >/dev/null || { echo "error: rustc is required"; exit 1; }
	@command -v cargo >/dev/null || { echo "error: cargo is required"; exit 1; }
	@command -v rust-objcopy >/dev/null || { echo "error: rust-objcopy is required"; exit 1; }
	@target_libdir="$$(RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) rustc \
		--print target-libdir --target $(TARGET_RV))"; \
	ls "$$target_libdir"/libcore-*.rlib >/dev/null 2>&1 || { \
		echo "error: Rust target $(TARGET_RV) is not installed for $(RV_TOOLCHAIN)"; exit 1; }
	@test -x "$(RV_RUST_LLD)" || { \
		echo "error: rust-lld is not installed for $(RV_TOOLCHAIN)"; exit 1; }

check-la-tools:
	@command -v rustup >/dev/null || { echo "error: rustup is required for kernel-la"; exit 1; }
	@command -v rustc >/dev/null || { echo "error: rustc is required"; exit 1; }
	@command -v cargo >/dev/null || { echo "error: cargo is required"; exit 1; }
	@command -v rust-objcopy >/dev/null || { echo "error: rust-objcopy is required"; exit 1; }
	@sysroot="$$(rustup run $(LA_TOOLCHAIN) rustc --print sysroot 2>/dev/null)" || { \
		echo "error: Rust toolchain $(LA_TOOLCHAIN) is not installed"; exit 1; }; \
	ls "$$sysroot/lib/rustlib/$(TARGET_LA)/lib"/libcore-*.rlib >/dev/null 2>&1 || { \
		echo "error: Rust target $(TARGET_LA) is not installed for $(LA_TOOLCHAIN)"; exit 1; }
	@command -v cmake >/dev/null || { echo "error: cmake is required for kernel-la"; exit 1; }
	@command -v loongarch64-linux-musl-gcc >/dev/null || { \
		echo "error: loongarch64-linux-musl-gcc is required for kernel-la"; exit 1; }

restore-vendor-rv:
	find $(SWTC_DIR)/vendor -name cargo-checksum.json -exec sh -c \
		'cp "$$1" "$$(dirname "$$1")/.cargo-checksum.json"' _ {} \;

restore-vendor-la:
	find $(SWTC_LA)/vendor -name cargo-checksum.json -exec sh -c \
		'cp "$$1" "$$(dirname "$$1")/.cargo-checksum.json"' _ {} \;

prepare-rv: check-rv-tools restore-vendor-rv
	mkdir -p $(SWTC_KERNEL)/.cargo $(SWTC_USER)/.cargo
	cp $(SWTC_KERNEL)/cargo-config/config.toml $(SWTC_KERNEL)/.cargo/config.toml
	cp $(SWTC_USER)/cargo-config/config.toml $(SWTC_USER)/.cargo/config.toml

prepare-la: check-la-tools restore-vendor-la
	mkdir -p $(SWTC_LA)/.cargo
	cp $(SWTC_LA)/cargo-config/config.toml $(SWTC_LA)/.cargo/config.toml

build-rv: prepare-rv
	RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) $(MAKE) -C $(SWTC_USER) build PRELIMINARY=0
	RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) $(MAKE) -C $(SWTC_KERNEL) kernel TMPFS=1 SUBMIT=1
	RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) rust-objcopy \
		$(SWTC_ELF) --strip-all -O binary $(SWTC_BIN)
	cd $(dir $(SWTC_BIN)) && RUSTUP_TOOLCHAIN=$(RV_TOOLCHAIN) rust-objcopy \
		-I binary -O elf64-littleriscv \
		--binary-architecture=riscv64 \
		--rename-section .data=.text,alloc,load,readonly,code,contents \
		$(notdir $(SWTC_BIN)) $(notdir $(WRAPPER_OBJ))
	$(RV_RUST_LLD) -flavor gnu -m elf64lriscv -T $(WRAPPER_LD) \
		-o $(KERNEL_RV) $(WRAPPER_OBJ)

build-la-strict: prepare-la
	$(MAKE) -C $(SWTC_LA) TOOLCHAIN=$(LA_TOOLCHAIN) ARCH=loongarch64 \
		BLK=y NET=y FEATURES=fp_simd,lwext4_rs,driver-virtio-blk build
	cp $(SWTC_LA_ELF) $(KERNEL_LA)

build-la: build-rv
	@if $(MAKE) --no-print-directory check-la-tools >/dev/null 2>&1; then \
		$(MAKE) --no-print-directory build-la-strict; \
	else \
		echo "warning: kernel-la placeholder generated because LoongArch toolchain is unavailable."; \
		cp $(KERNEL_RV) $(KERNEL_LA); \
	fi

build: build-rv build-la

clean:
	$(MAKE) -C $(SWTC_KERNEL) clean
	$(MAKE) -C $(SWTC_USER) clean
	$(MAKE) -C $(SWTC_LA) clean
	rm -f $(KERNEL_RV) $(KERNEL_LA)

.PHONY: all check-rv-tools check-la-tools restore-vendor-rv restore-vendor-la \
	prepare-rv prepare-la build build-rv build-la build-la-strict clean
