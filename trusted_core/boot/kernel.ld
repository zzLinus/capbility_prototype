OUTPUT_ARCH( "riscv" )
ENTRY( _entry )

SECTIONS
{
  /*
   * ensure that entry.S / _entry is at 0x80000000,
   * where qemu's -kernel jumps.
   */
  . = 0x80000000;
  PROVIDE(kernel_base = .);

  .text : {
    *(.text.entry .text*)
    . = ALIGN(0x1000);
    _trampoline = .;
    *(trampsec)
    . = ALIGN(0x1000);
    /* ASSERT(. - _trampoline == 0x1000, "error: trampoline larger than one page"); */
    PROVIDE(text_end = .);
  }

  .rodata : {
    . = ALIGN(16);
    PROVIDE(rodata_start = .);
    *(.srodata .srodata.*) /* do not need to distinguish this from .rodata */
    . = ALIGN(16);
    *(.rodata .rodata.*)
    PROVIDE(rodata_end = .);
  }

  .data : {
    . = ALIGN(16);
    PROVIDE(data_start = .);
    *(.sdata .sdata.*) /* do not need to distinguish this from .data */
    . = ALIGN(16);
    *(.data .data.*)
    PROVIDE(data_end = .);
  }

  .bss : {
    . = ALIGN(16);
    PROVIDE(bss_start = .);
    *(.sbss .sbss.*) /* do not need to distinguish this from .bss */
    . = ALIGN(16);
    *(.bss .bss.*)
    PROVIDE(bss_end = .);
  }

  . = ALIGN(16);
  . = . + 0x2000;
  heap_start = .;
  heap_end = . + 0x4000000;
  . = heap_end;

  PROVIDE(end = .);
  PROVIDE(kernel_end = 0x88000000);
}
