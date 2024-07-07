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

	csrr a1, mcause
	bgez a1, .Lecall

	li a0, MIP_MTIP
	csrc mie, a0
	li a0, MIP_SSIP
	csrs mip, a0

.Lmret:
	LOAD a0, 10*REGBYTES(sp)
	LOAD a1, 11*REGBYTES(sp)
	csrrw sp, mscratch, sp
	mret

.Lecall:
	li a0, MIP_STIP
	csrc mip, a0
	li a0, MIP_MTIP
	csrs mie, a0
	csrr a0, mepc
	addi a0, a0, 4
	csrw mepc, a0
	j .Lmret
1:
	mret
