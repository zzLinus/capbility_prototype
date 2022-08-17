use crate::cpu::r_scause;
use crate::timer::s_timer_trap;

const SUPERVISOR_TIMER_INTERRUPT: usize = 0x8000000000000005;

#[no_mangle]
extern "C" fn s_trap() {
    match r_scause() {
        SUPERVISOR_TIMER_INTERRUPT => s_timer_trap(),
        _ => unreachable!(),
    }
}
