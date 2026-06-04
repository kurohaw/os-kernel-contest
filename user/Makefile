TARGET := riscv64gc-unknown-none-elf
MODE := release
APPS := app0 app1
ELF_DIR := target/$(TARGET)/$(MODE)
BIN_DIR := build

OBJCOPY := rust-objcopy --binary-architecture=riscv64

all: build

build:
	cargo build --release
	mkdir -p $(BIN_DIR)
	for app in $(APPS); do \
		$(OBJCOPY) $(ELF_DIR)/$$app --strip-all -O binary $(BIN_DIR)/$$app.bin; \
	done

clean:
	cargo clean
	rm -rf $(BIN_DIR)

.PHONY: all build clean