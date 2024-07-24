#include "bits.h"
#include "encoding.h"
.altmacro

# load general purpose register
.macro LOAD_GR n
	LOAD x\n, \n*REGBYTES(sp)	
.endm

	.option norvc
	.section .text
	.global s_trap_vector
	.global __restore_context
	.align 4

s_trap_vector:
	csrrw	sp, sscratch, sp
	# allot space for trap context
	addi sp, sp, -REGBYTES*36

	STORE ra, 1*REGBYTES(sp)
	STORE gp, 3*REGBYTES(sp)
	STORE tp, 4*REGBYTES(sp)
	STORE t0, 5*REGBYTES(sp)
	STORE t1, 6*REGBYTES(sp)
	STORE t2, 7*REGBYTES(sp)
	STORE s0, 8*REGBYTES(sp)
	STORE s1, 9*REGBYTES(sp)
	STORE a0, 10*REGBYTES(sp)
	STORE a1, 11*REGBYTES(sp)
	STORE a2, 12*REGBYTES(sp)    
	STORE a3, 13*REGBYTES(sp)
	STORE a4, 14*REGBYTES(sp)
	STORE a5, 15*REGBYTES(sp)
	STORE a6, 16*REGBYTES(sp)
	STORE a7, 17*REGBYTES(sp)
	STORE s2, 18*REGBYTES(sp)
	STORE s3, 19*REGBYTES(sp)
	STORE s4, 20*REGBYTES(sp)
	STORE s5, 21*REGBYTES(sp)
	STORE s6, 22*REGBYTES(sp)
	STORE s7, 23*REGBYTES(sp)
	STORE s8, 24*REGBYTES(sp)
	STORE s9, 25*REGBYTES(sp)
	STORE s10, 26*REGBYTES(sp)
	STORE s11, 27*REGBYTES(sp)
	STORE t3, 28*REGBYTES(sp)
	STORE t4, 29*REGBYTES(sp)
	STORE t5, 30*REGBYTES(sp)
	STORE t6, 31*REGBYTES(sp)

	csrr s0, sscratch
	csrr s1, sstatus
	csrr s2, sepc
	csrr s3, stval
	csrr s4, scause

	# addi s5, sp, 0 <- omit storing kernel stack for now
	STORE s0, 2*REGBYTES(sp)
	STORE s1, 32*REGBYTES(sp)
	STORE s2, 33*REGBYTES(sp)
	STORE s3, 34*REGBYTES(sp)
	STORE s4, 35*REGBYTES(sp)
	# STORE s5, 36*REGBYTES(sp)

	# set up argument(&TrapContext) for s_trap
	mv a0, sp

	call s_trap

__restore_context:
	# call convention: __restore_context(trap_context*)
	# jump from general kernel stack allocated when booting to specified kernel stack 

	# restore status register, currently only restore sstatus & sepc
	LOAD t0, 32*REGBYTES(sp)
	LOAD t1, 33*REGBYTES(sp)
	LOAD t2, 2*REGBYTES(sp)

	csrw sstatus, t0
	csrw sepc, t1
	# store user sp for later exchange
	csrw sscratch, t2 

	# restore general purpose registers
	LOAD_GR 1
	LOAD_GR 3

	# skip sp(x2), load x4-x31
	.set n, 4
	.rept 28
		LOAD_GR %n
		.set n, n+1 
	.endr

	# release trapcontext on kernel stack
	addi sp, sp, 36*REGBYTES
	csrrw sp, sscratch, sp
	sret


