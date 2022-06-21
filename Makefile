# Build the image of safeOS

# Tool chain
TOOLCHAIN_PATH = /usr/bin
TOOLCHAIN_PREFIX = $(TOOLCHAIN_PATH)/riscv64-linux-gnu-
CC = $(TOOLCHAIN_PREFIX)gcc
LD = $(TOOLCHAIN_PREFIX)ld
OBJCOPY = $(TOOLCHAIN_PREFIX)objcopy
OBJDUMP = $(TOOLCHAIN_PREFIX)objdump

# Build path and variables
ROOT_DIR := $(realpath $(dir $(firstword $(MAKEFILE_LIST))))
BUILD_DIR = $(ROOT_DIR)/build
TRUSTED_CORE_SRC_DIR = $(ROOT_DIR)/trusted_core

TRUSTED_CORE_ASM_FILES = $(wildcard $(TRUSTED_CORE_SRC_DIR)/*.S)
TRUSTED_CORE_ASM_OBJS = $(subst $(ROOT_DIR), $(BUILD_DIR), $(TRUSTED_CORE_ASM_FILES:.S=.o))
TRUSTED_CORE_ASM_DEPS = $(TRUSTED_CORE_ASM_OBJS:.o=.d)
TRUSTED_CORE_C_FILES = $(wildcard $(TRUSTED_CORE_SRC_DIR)/*.c)
TRUSTED_CORE_C_OBJS = $(subst $(ROOT_DIR), $(BUILD_DIR), $(TRUSTED_CORE_C_FILES:.c=.o))
TRUSTED_CORE_C_DEPS = $(TRUSTED_CORE_C_OBJS:.o=.d)

# compiler options, borrowed from xv6-riscv
LINKER_SCRIPT = $(TRUSTED_CORE_SRC_DIR)/kernel.ld
CFLAGS = -Wall -Werror -O -fno-omit-frame-pointer -ggdb
CFLAGS += -MD
CFLAGS += -mcmodel=medany
CFLAGS += -ffreestanding -fno-common -nostdlib -mno-relax
CFLAGS += -I.
CFLAGS += $(shell $(CC) -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)

LDFLAGS = -z max-page-size=4096

# Target
TRUSTED_CORE_LIB = $(BUILD_DIR)/trusted_core.a

# implicit rules to compile assembly files
$(BUILD_DIR)/%.o: $(ROOT_DIR)/%.S
	@mkdir -p $(dir $@)
	@echo CC $<
	@$(CC) $(CFLAGS) -c $< -o $@

# implicit rules to compile C files
$(BUILD_DIR)/%.o: $(ROOT_DIR)/%.c
	@mkdir -p $(dir $@)
	@echo CC $<
	@$(CC) $(CFLAGS) -c $< -o $@

# build trusted_core
$(TRUSTED_CORE_LIB): $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_C_OBJS)
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -T$(LINKER_SCRIPT) -o $@ $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_C_OBJS)

# build and run qemu image
qemu: $(TRUSTED_CORE_LIB)

# build all
all: $(TRUSTED_CORE_LIB)

# clean
clean:
	rm -rf $(TRUSTED_CORE_ASM_OBJS) $(TRUSTED_CORE_ASM_DEPS) $(TRUSTED_CORE_C_OBJS) $(TRUSTED_CORE_C_DEPS) $(TRUSTED_CORE_LIB)
# test
test:
	@echo $(ROOT_DIR)
	@echo $(TRUSTED_CORE_DIR)
	@echo $(TRUSTED_CORE_ASM_OBJS)
	@echo $(TRUSTED_CORE_ASM_FILES)
	@echo $(TRUSTED_CORE_ASM_OBJS)
