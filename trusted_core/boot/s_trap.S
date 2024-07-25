#include "bits.h"
#include "encoding.h"
.altmacro

# load general purpose register
.macro LOAD_GR n
	LOAD x\n, \n*REGBYTES(sp)	
.endm
.macro SAVE_GR n
	STORE x\n, \n*REGBYTES(sp)
.endm

	.option norvc
	.section .text
	.global __kernel_trap_vector 
	.global __user_trap_vector
	.global __restore_context
	.align 4

__kernel_trap_vector:
	# handle trap issued from S mode (delegated from M mode)
	# use current sp to store context instead of sscratch pointer
	addi sp, sp, -REGBYTES*36
	SAVE_GR 1

	.set n, 3
	.rept 29
		SAVE_GR %n
		.set n, n+1
	.endr

	csrr s1, sstatus
	csrr s2, sepc
	csrr s3, stval
	csrr s4, scause

	addi a0, sp, REGBYTES*36

	STORE a0, 2*REGBYTES(sp)
	STORE s1, 32*REGBYTES(sp)
	STORE s2, 33*REGBYTES(sp)
	STORE s3, 34*REGBYTES(sp)
	STORE s4, 35*REGBYTES(sp)
	call kernel_trap
	j __restore_context

__user_trap_vector:
	# handle trap issued from U mode, sscratch is properly set with kernel stack pointer
	csrrw	sp, sscratch, sp
	# allot space for trap context
	addi sp, sp, -REGBYTES*36
	SAVE_GR 1
	# skip 2
	.set n, 3
	.rept 29
		SAVE_GR %n
		.set n, n+1
	.endr

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

	call user_trap

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

	# skip sp(x2), load x3-x31
	.set n, 3
	.rept 29
		LOAD_GR %n
		.set n, n+1 
	.endr

	# release trapcontext on kernel stack
	addi sp, sp, 36*REGBYTES
	csrrw sp, sscratch, sp
	sret


