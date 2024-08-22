# Define the output directory
BUILD_DIR := build

# Create the build directory if it doesn't exist
$(shell mkdir -p $(BUILD_DIR))

# Define sources and objects
SOURCES := $(wildcard *.c) $(wildcard *.S)
OBJECTS := $(patsubst %.c,$(BUILD_DIR)/%.o,$(SOURCES))
OBJECTS := $(patsubst %.S,$(BUILD_DIR)/%.o,$(OBJECTS))

include ../../include.mk

TRUSTED_CORE_BOOT_CLIB ?= $(BUILD_DIR)/libbootc.a

# Define rules for building object files from source files
$(BUILD_DIR)/%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

$(BUILD_DIR)/%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

all:$(TRUSTED_CORE_BOOT_CLIB)

# Rule for building the final library
$(TRUSTED_CORE_BOOT_CLIB): $(OBJECTS)
	$(AR) rcs $@ $^

# Clean target
.PHONY: clean
clean:
#   -rm -rf $(BUILD_DIR)/*.a $(BUILD_DIR)/*.o $(BUILD_DIR)/*.d
	rm -rf $(BUILD_DIR)

