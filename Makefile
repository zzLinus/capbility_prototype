# Build the image of safeOS

include include.mk

# Build path and variables
ROOT_DIR := $(realpath $(dir $(firstword $(MAKEFILE_LIST))))
BUILD_DIR = $(ROOT_DIR)/build
TRUSTED_CORE_SRC_DIR = $(ROOT_DIR)/trusted_core

# rust libray
RUST_BUILD_TYPE = debug
TRUSTED_CORE_RUST_DIR = $(TRUSTED_CORE_SRC_DIR)/rust_main

# compiler options, borrowed from xv6-riscv
LINKER_SCRIPT = $(TRUSTED_CORE_SRC_DIR)/boot/kernel.ld

LDFLAGS = -z max-page-size=4096

# Target
TARGET = $(BUILD_DIR)/safeos.elf
TRUSTED_CORE_RUST_LIB_DIR = $(TRUSTED_CORE_SRC_DIR)/rust_main/target/$(RUST_TOOLCHAIN_TARGET)/$(RUST_BUILD_TYPE)
TRUSTED_CORE_RUST_BIN = $(TRUSTED_CORE_RUST_LIB_DIR)/rust_main

# qemu
QEMU = qemu-system-riscv64

# implicit rules to compile assembly files
$(BUILD_DIR)/%.o: $(ROOT_DIR)/%.S
	@mkdir -p $(dir $@)
	@echo CC $<
	$(CC) $(CFLAGS) -c $< -o $@

# implicit rules to compile C files
$(BUILD_DIR)/%.o: $(ROOT_DIR)/%.c
	@mkdir -p $(dir $@)
	@echo CC $<
	$(CC) $(CFLAGS) -c $< -o $@

# build all
all: $(BUILD_DIR)
	$(MAKE) --directory=$(TRUSTED_CORE_SRC_DIR) all
	cp $(TRUSTED_CORE_RUST_BIN) $(TARGET)

# build test
test: $(BUILD_DIR)
	$(MAKE) --directory=$(TRUSTED_CORE_SRC_DIR) test
	cp $(TRUSTED_CORE_RUST_BIN) $(TARGET)
	
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

# clean
clean:
	$(MAKE) --directory=$(TRUSTED_CORE_SRC_DIR) clean
	rm -rf $(BUILD_DIR)