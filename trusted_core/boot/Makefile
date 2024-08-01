SOURCES := $(wildcard *.c) $(wildcard *.S)
OBJECTS := $(patsubst %.c,%.o,$(SOURCES))
OBJECTS := $(patsubst %.S,%.o,$(OBJECTS))

include ../../include.mk

TRUSTED_CORE_BOOT_CLIB		?= libbootc.a

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

all:$(TRUSTED_CORE_BOOT_CLIB)
	echo $(TRUSTED_CORE_BOOT_CLIB)

$(TRUSTED_CORE_BOOT_CLIB):$(OBJECTS)
	$(AR) rcs $@ $^

.PHONY: clean
clean:
	-rm -rf *.a *.o *.d
