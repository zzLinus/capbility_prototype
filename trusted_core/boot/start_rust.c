#include "include/page.h"
#include "include/context.h"
#include "include/riscv_asm_c_wrap.h"

#define CPU_NUM 2

#define SCRATCH_REG_SIZE 5
#define PMP_ALL_PHY_MEM 0x3fffffffffffffUL

//CLINT
#define CLINT_BASE 0x2000000
#define CLINT_MTIMECMP (CLINT_BASE + 0x4000)
#define CLINT_MTIME (CLINT_BASE + 0xBFF8)
#define CLINT_INTERVAL 500000

#define REG64(addr) ((volatile uint64_t*)addr)
#define ARRAY_SIZE(a) (sizeof(a) / sizeof(a[0]))

// M mode trap vector which is implemted by assmbly language
extern void m_trap_vector();
extern void rust_main();
extern void __user_trap_vector();
extern void __kernel_trap_vector();

/*
 * spaces for kernel stack, S mode trap context, and M mode trap context.
 * kernel stack: 4K
 * S mode trap context: 4K
 * M mode trap context: 4k
 * currently safeOS only provies only one context space, so it does not
 * support multiple process/thread.
 */
__attribute__ ((aligned (16))) char kernel_stack[PAGE_SIZE * 4];
__attribute__ ((aligned (16))) uint64 m_trap_context[PAGE_SIZE/sizeof(uint64)];
__attribute__ ((aligned (16))) uint64 s_trap_context[PAGE_SIZE/sizeof(uint64)];



// entry.S jumps start_rust in M mode on stack0
// start_rust() prepares to jump to rust language which is in S mode, it:
// 1. initialize memory management, includng satp, stack, bss(?)
// 2. initialize traps and interrupts:
//    a) delegate traps and interupts to S mode (MEDELEG, MIDELEG), and
//       set trap/interrupt handler (MTVEC)
//    b) set the mscratch register used to save context
//    c) set return mode to S mode (MPP, MEPC)
void start_rust()
{
	uint64 *mscratch_ptr, *sscratch_ptr;

	// initialzie memory mamagement
	// 1. set satp
	// 2. todo: set separte stack for S mode
	w_satp(0);

	// delegate U-ecall, keep S-ecall
	w_medeleg(0xF1FF);
	w_mideleg(0xFFFF);
	w_mie(r_mie() | MIE_MEIE | MIE_MTIE | MIE_MSIE);
	w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);
	w_mtvec((uint64)m_trap_vector);
	w_stvec((uint64)__kernel_trap_vector);

	// set context saving spaces for S and M mode (mscratch, sscratch)
	m_trap_context[3] = CLINT_MTIMECMP;
	m_trap_context[4] = CLINT_INTERVAL; 
	mscratch_ptr = &m_trap_context[0];
	sscratch_ptr = &s_trap_context[ARRAY_SIZE(s_trap_context)-1];
	w_mscratch((uint64)mscratch_ptr);
	w_sscratch((uint64)sscratch_ptr);

	// configure PMP to allow S mode accessing all physical memory
	w_pmpaddr0(PMP_ALL_PHY_MEM);
	w_pmpcfg0(0xF);

	// prepare to return to S mode
	uint64 temp = r_mstatus();
	temp &= ~MSTATUS_MPP_MASK;
	temp |= MSTATUS_MPP_S;
	w_mstatus(temp);
	w_mepc((uint64)rust_main);
	asm volatile("mret");
}
