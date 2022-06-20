#include <riscv.h>

#define CPU_NUM 2

#define SCRATCH_REG_SIZE 5
#define PMP_ALL_PHY_MEM 0x3fffffffffffffUL

// M mode trap vector which is implemted by assmbly language
extern void m_trap_vector();

__attribute__ ((aligned (16))) char c_stack[4096 * CPU_NUM];
uint64 scratch_context[SCRATCH_REG_SIZE];

// entry.S jumps start_rust in M mode on stack0
// start_rust() jumps to rust language which is in S mode, before that
// it needs to:
// 1. initialize memory management, includng satp, stack, bss(?)
// 2. initialize traps and interrupts:
//    a) delegate traps and interupts to S mode (MEDELEG, MIDELEG), and
//       set trap/interrupt handler (MTVEC)
//    b) set the mscratch register used to save context
//    c) set return mode to S mode (MPP, MEPC)
void start_rust()
{
	uint64 *scratch_ptr;

	// initialzie memory mamagement
	// 1. set satp
	// 2. todo: set separte stack for S mode
	w_satp(0);

	// delegate traps and interrupts to S mode
	w_medeleg(0xFFFF);
	w_mideleg(0xFFFF);
	w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);
	w_mtvec((uint64)m_trap_vector);

	// set mscratch register
	scratch_ptr = &scratch_context[0];
	w_mscratch((uint64)scratch_ptr);

	// configure PMP to allow S mode accessing all physical memory
	w_pmpaddr0(PMP_ALL_PHY_MEM);
	w_pmpcfg0(0xF);

	// prepare to return to S mode
	uint64 temp = r_mstatus();
	temp &= ~MSTATUS_MPP_MASK;
	temp |= MSTATUS_MPP_S;
	w_mstatus(temp);
	// w_mepc((uint64)rust_main);
	asm volatile("mret");
}
