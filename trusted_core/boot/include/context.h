#ifndef _CONTEXT_H
#define _CONTEXT_H

#include "types.h"

struct riscv64_user_context {
	unsigned long pc;
	unsigned long ra;
	unsigned long sp;
	unsigned long gp;
	unsigned long tp;
	unsigned long t0;
	unsigned long t1;
	unsigned long t2;
	unsigned long s0;
	unsigned long s1;
	unsigned long a0;
	unsigned long a1;
	unsigned long a2;
	unsigned long a3;
	unsigned long a4;
	unsigned long a5;
	unsigned long a6;
	unsigned long a7;
	unsigned long s2;
	unsigned long s3;
	unsigned long s4;
	unsigned long s5;
	unsigned long s6;
	unsigned long s7;
	unsigned long s8;
	unsigned long s9;
	unsigned long s10;
	unsigned long s11;
	unsigned long t3;
	unsigned long t4;
	unsigned long t5;
	unsigned long t6;
};

struct riscv64_d_ext_context {
	uint64 f[32];
	uint32 fcsr;
	uint8 _padding[4];
};

struct cpu_context {
	struct riscv64_user_context u_context;
	struct riscv64_d_ext_context fpu_context;
};

#endif /* _CONTEXT_H */
