use crate::cpu::{r_scause, r_sstatus, SSTATUS_SPP};

use crate::kernel_object::tcb::TrapContext;
use crate::timer::clear_pending_tintr;

const S_TIMER_INT: usize = 0x1usize << 63 | 0x5;
const S_SOFT_INT: usize = 0x1usize << 63 | 0x1;
const M_TIMER_INT: usize = 0x1usize << 63 | 0x7;
const U_ECALL: usize = 0x8;

#[no_mangle]
extern "C" fn kernel_trap(ctx: &mut TrapContext) -> &TrapContext {
    assert!(r_sstatus() & SSTATUS_SPP != 0);
    let scause = r_scause();
    match scause {
        // clear pending SINTR set in M mode
        S_SOFT_INT => clear_pending_tintr(),
        _ => unreachable!(),
    }
    ctx
}
