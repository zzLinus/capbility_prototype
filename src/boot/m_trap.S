#include "bits.h"
#include "encoding.h"

	.option norvc

	.section .text
	.global m_trap_vector

	.align 4

m_trap_vector:
	csrrw sp, mscratch, sp
	STORE a0, 10*REGBYTES(sp)
	STORE a1, 11*REGBYTES(sp)
	STORE a2, 12*REGBYTES(sp) 

	# clint cmp addr
	ld a0, 3*REGBYTES(sp)
	ld a1, 4*REGBYTES(sp)
	ld a2, 0(a0)
	add a2, a2, a1
	sd a2, 0(a0)

	li a0, MIP_SSIP
	csrs mip, a0

.Lmret:
	LOAD a0, 10*REGBYTES(sp)
	LOAD a1, 11*REGBYTES(sp)
	LOAD a2, 12*REGBYTES(sp)
	csrrw sp, mscratch, sp
	mret

1:
	mret
