# Build the image of safeOS

# Tool chain
TOOLCHAIN_PATH = /usr/bin
TOOLCHAIN_PREFIX = $(TOOLCHAIN_PATH)/riscv64-unknown-elf-
CC = $(TOOLCHAIN_PREFIX)gcc
LD = $(TOOLCHAIN_PREFIX)ld
AS = $(TOOLCHAIN_PREFIX)as
OBJCOPY = $(TOOLCHAIN_PREFIX)objcopy
OBJDUMP = $(TOOLCHAIN_PREFIX)objdump
RUST_BUILD_TYPE = debug
RUST_TOOLCHAIN_TARGET = riscv64gc-unknown-none-elf

# Build path and variables
ROOT_DIR := $(realpath $(dir $(firstword $(MAKEFILE_LIST))))
BUILD_DIR = $(ROOT_DIR)/build
TRUSTED_CORE_SRC_DIR = $(ROOT_DIR)/trusted_core

TRUSTED_CORE_ASM_FILES = $(wildcard $(TRUSTED_CORE_SRC_DIR)/boot/*.S)
TRUSTED_CORE_ASM_OBJS = $(subst $(ROOT_DIR), $(BUILD_DIR), $(TRUSTED_CORE_ASM_FILES:.S=.o))
TRUSTED_CORE_ASM_DEPS = $(TRUSTED_CORE_ASM_OBJS:.o=.d)
TRUSTED_CORE_C_FILES = $(wildcard $(TRUSTED_CORE_SRC_DIR)/boot/*.c)
TRUSTED_CORE_C_OBJS = $(subst $(ROOT_DIR), $(BUILD_DIR), $(TRUSTED_CORE_C_FILES:.c=.o))
TRUSTED_CORE_C_DEPS = $(TRUSTED_CORE_C_OBJS:.o=.d)

# rust libray
RUST_BUILD_TYPE = debug
TRUSTED_CORE_RUST_DIR = $(TRUSTED_CORE_SRC_DIR)/rust_main

# compiler options, borrowed from xv6-riscv
LINKER_SCRIPT = $(TRUSTED_CORE_SRC_DIR)/boot/kernel.ld
CFLAGS = -Wall -Werror -O -fno-omit-frame-pointer -ggdb
CFLAGS += -MD
CFLAGS += -mcmodel=medany
CFLAGS += -ffreestanding -fno-common -nostdlib -mno-relax
CFLAGS += -I.
CFLAGS += $(shell $(CC) -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)

# Disable PIE when possible (for Ubuntu toolchain)
ifneq ($(shell $(CC) -dumpspecs 2>/dev/null | grep -e '[^f]no-pie'),)
CFLAGS += -fno-pie -no-pie
endif
ifneq ($(shell $(CC) -dumpspecs 2>/dev/null | grep -e '[^f]nopie'),)
CFLAGS += -fno-pie -nopie
endif

LDFLAGS = -z max-page-size=4096

# Target
TARGET = $(BUILD_DIR)/safeos.elf
TRUSTED_CORE_RUST_LIB_DIR = $(TRUSTED_CORE_SRC_DIR)/rust_main/target/$(RUST_TOOLCHAIN_TARGET)/$(RUST_BUILD_TYPE)
TRUSTED_CORE_RUST_LIB = $(TRUSTED_CORE_RUST_LIB_DIR)/librust_main.a

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

.PHONY: rust_lib rust_lib_with_tests

# build rust libs
rust_lib:
	cd $(TRUSTED_CORE_RUST_DIR) && cargo build

# build rust libs with tests
rust_lib_with_tests:
	cd $(TRUSTED_CORE_RUST_DIR) && cargo rustc -- --cfg 'kernel_test'

# build all
all: $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_C_OBJS) rust_lib user_lib
	$(LD) $(LDFLAGS) -T$(LINKER_SCRIPT) -o $(TARGET) $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_C_OBJS) $(TRUSTED_CORE_RUST_LIB)

# build test
test: $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_C_OBJS) rust_lib_with_tests
	$(LD) $(LDFLAGS) -T$(LINKER_SCRIPT) -o $(TARGET) $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_C_OBJS) $(TRUSTED_CORE_RUST_LIB)

# build and run qemu image
CPU_NUM = 1
QEMUOPTS = -machine virt -bios none -kernel $(TARGET) -m 128M -smp $(CPU_NUM) -nographic
QEMUOPTS += -drive file=$(BUILD_DIR)/hdd.dsk,if=none,format=raw,id=x0
QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
qemu: all
	cd $(BUILD_DIR) && dd if=/dev/zero of=$(BUILD_DIR)/hdd.dsk bs=1M count=32
	$(QEMU) $(QEMUOPTS)
#	$(QEMU) -machine $(MACH) -cpu $(CPU) -smp $(CPU_NUM) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(TARGET) -drive if=none,format=raw,file=$(DRIVE),id=foo -device virtio-blk-device,scsi=off,drive=foo

# run qemu with gdb
QEMUGDB = -S -gdb tcp::26000
qemu-gdb: all
	cd $(BUILD_DIR) && dd if=/dev/zero of=$(BUILD_DIR)/hdd.dsk bs=1M count=32
	$(QEMU) $(QEMUOPTS) $(QEMUGDB)

# qemu test
qemu-test: test
	cd $(BUILD_DIR) && dd if=/dev/zero of=$(BUILD_DIR)/hdd.dsk bs=1M count=32
	$(QEMU) $(QEMUOPTS)

# clean
clean:
	rm -rf $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_ASM_DEPS) $(TRUSTED_CORE_C_OBJS) $(TRUSTED_CORE_C_DEPS) $(TRUSTED_CORE_LIB)
	cd $(TRUSTED_CORE_RUST_DIR) && cargo clean