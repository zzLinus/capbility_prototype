# Tool chain
# TOOLCHAIN_PATH = /usr/bin
# TOOLCHAIN_PREFIX = $(TOOLCHAIN_PATH)/riscv64-unknown-elf-
TOOLCHAIN_PREFIX := riscv64-unknown-linux-gnu-
CC := $(TOOLCHAIN_PREFIX)gcc
LD := $(TOOLCHAIN_PREFIX)ld
AS := $(TOOLCHAIN_PREFIX)as
AR := $(TOOLCHAIN_PATH)ar
GDB := $(TOOLCHAIN_PREFIX)gdb
OBJCOPY := $(TOOLCHAIN_PREFIX)objcopy
OBJDUMP := $(TOOLCHAIN_PREFIX)objdump
RUST_BUILD_TYPE := debug
RUST_TOOLCHAIN_TARGET := riscv64-unknown-safeos

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