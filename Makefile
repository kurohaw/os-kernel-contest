TITANIX_DIR := titanix
TITANIX_KERNEL := $(TITANIX_DIR)/kernel
TITANIX_USER := $(TITANIX_DIR)/user
TARGET_RV := riscv64gc-unknown-none-elf
MODE := release
TITANIX_ELF := $(TITANIX_KERNEL)/target/$(TARGET_RV)/$(MODE)/kernel
TITANIX_BIN := $(TITANIX_ELF).bin
WRAPPER_OBJ := $(TITANIX_KERNEL)/target/$(TARGET_RV)/$(MODE)/kernel-rv-wrapper.o
WRAPPER_LD := $(TITANIX_KERNEL)/kernel-rv-wrapper.ld
KERNEL_RV := kernel-rv
KERNEL_LA := kernel-la

RUST_SYSROOT := $(shell cd $(TITANIX_DIR) && rustc --print sysroot)
RUST_HOST := $(shell cd $(TITANIX_DIR) && rustc -vV | sed -n 's/^host: //p')
RUST_LLD := $(RUST_SYSROOT)/lib/rustlib/$(RUST_HOST)/bin/rust-lld

all: build

restore-vendor:
	find $(TITANIX_DIR)/vendor -name cargo-checksum.json -exec sh -c \
		'cp "$$1" "$$(dirname "$$1")/.cargo-checksum.json"' _ {} \;

prepare-cargo: restore-vendor
	mkdir -p $(TITANIX_KERNEL)/.cargo $(TITANIX_USER)/.cargo
	cp $(TITANIX_KERNEL)/cargo-config/config.toml $(TITANIX_KERNEL)/.cargo/config.toml
	cp $(TITANIX_USER)/cargo-config/config.toml $(TITANIX_USER)/.cargo/config.toml

build: prepare-cargo
	$(MAKE) -C $(TITANIX_USER) build PRELIMINARY=0
	$(MAKE) -C $(TITANIX_KERNEL) kernel TMPFS=1 SUBMIT=1
	rust-objcopy $(TITANIX_ELF) --strip-all -O binary $(TITANIX_BIN)
	cd $(dir $(TITANIX_BIN)) && rust-objcopy -I binary -O elf64-littleriscv \
		--binary-architecture=riscv64 \
		--rename-section .data=.text,alloc,load,readonly,code,contents \
		$(notdir $(TITANIX_BIN)) $(notdir $(WRAPPER_OBJ))
	$(RUST_LLD) -flavor gnu -m elf64lriscv -T $(WRAPPER_LD) \
		-o $(KERNEL_RV) $(WRAPPER_OBJ)
	cp $(KERNEL_RV) $(KERNEL_LA)
	@echo "warning: kernel-la is a temporary placeholder; LoongArch is not implemented."

clean:
	$(MAKE) -C $(TITANIX_KERNEL) clean
	$(MAKE) -C $(TITANIX_USER) clean
	rm -f $(KERNEL_RV) $(KERNEL_LA)

.PHONY: all restore-vendor prepare-cargo build clean
