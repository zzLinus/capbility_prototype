# Build the image of safeOS

include include.mk

# Build path and variables
ROOT_DIR := $(realpath $(dir $(firstword $(MAKEFILE_LIST))))
BUILD_DIR = $(ROOT_DIR)/build
RUST_BUILD_TYPE = debug

# Target
TARGET = $(BUILD_DIR)/safeos.elf
TRUSTED_OS_RUST_LIB_DIR = $(BUILD_DIR)/$(RUST_TOOLCHAIN_TARGET)/$(RUST_BUILD_TYPE)
TRUSTED_OS_RUST_BIN = $(TRUSTED_OS_RUST_LIB_DIR)/os

# qemu
QEMU = qemu-system-riscv64

# build all
all: $(BUILD_DIR) fmt
	@cargo build
	@ln -sf $(TRUSTED_OS_RUST_BIN) $(TARGET)

# build test
# TODO: impl test
test: $(BUILD_DIR) fmt
	@ln -sf $(TRUSTED_OS_RUST_BIN) $(TARGET)
	
# build and run qemu image
CPU_NUM = 1
QEMUOPTS = -machine virt -bios none -kernel $(TARGET) -m 128M -smp $(CPU_NUM) -nographic
QEMUOPTS += -drive file=$(BUILD_DIR)/hdd.dsk,if=none,format=raw,id=x0
QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
qemu: all
	cd $(BUILD_DIR) && dd if=/dev/zero of=$(BUILD_DIR)/hdd.dsk bs=1M count=32
	$(QEMU) $(QEMUOPTS)

# run qemu with gdb
QEMUGDB = -S -gdb tcp::26000
qemu-gdb: all
	cd $(BUILD_DIR) && dd if=/dev/zero of=$(BUILD_DIR)/hdd.dsk bs=1M count=32
	$(QEMU) $(QEMUOPTS) $(QEMUGDB)

# qemu test
qemu-test: test
	cd $(BUILD_DIR) && dd if=/dev/zero of=$(BUILD_DIR)/hdd.dsk bs=1M count=32
	$(QEMU) $(QEMUOPTS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

clippy:
	@cargo clippy

fmt:
	@cargo fmt

clean:
	@cargo clean