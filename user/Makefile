TARGET := riscv64gc-unknown-none-elf
MODE := release
APPS := app0 app1
ELF_DIR := target/$(TARGET)/$(MODE)
BIN_DIR := build
CARGO_CONFIG_SRC := cargo-config/config.toml
CARGO_CONFIG_DST := .cargo/config.toml

OBJCOPY := rust-objcopy --binary-architecture=riscv64

all: build

prepare-cargo:
	mkdir -p .cargo
	cp $(CARGO_CONFIG_SRC) $(CARGO_CONFIG_DST)

build: prepare-cargo
	cargo build --release
	mkdir -p $(BIN_DIR)
	for app in $(APPS); do \
		$(OBJCOPY) $(ELF_DIR)/$$app --strip-all -O binary $(BIN_DIR)/$$app.bin; \
	done

clean:
	cargo clean
	rm -rf $(BIN_DIR)

.PHONY: all prepare-cargo build clean
