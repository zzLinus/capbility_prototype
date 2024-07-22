TARGET := riscv64gc-unknown-none-elf
MODE := release
TARGET_DIR := target/${TARGET}/${MODE}

APP_DIR := src/bin
APP_SRC := $(wildcard ${APP_DIR}/*.rs)
APP_ELF := $(patsubst ${APP_DIR}/%.rs, ${TARGET_DIR}/%, ${APP_SRC})

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

CARGO_OPT := 
ifeq (${MODE}, release)
	CARGO_OPT := --release
endif

# strip elf header so that initial pc is tied to the loaded position and can start execute from there
all:
	@cargo build ${CARGO_OPT}
	@$(foreach elf, ${APP_ELF}, ${OBJCOPY} ${elf} --strip-all -O binary $(patsubst ${TARGET_DIR}/%, ${TARGET_DIR}/%.bin, ${elf});)