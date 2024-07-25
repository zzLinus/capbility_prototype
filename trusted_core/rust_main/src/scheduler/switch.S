#include
.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm

.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm

    .section .text
    .global __switch
__switch:
    # switch between sche contexts: src & dst
    # sche context: ra, sp, called saved register s/n * 12
    # calling convention: __switch(src, dst)

    # save src
    sd ra, 0(a0)
    sd sp, 8(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n+1
    .endr

    # load dst
    ld ra, 0(a1)
    ld sp, 8(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n+1
    .endr
    ret


