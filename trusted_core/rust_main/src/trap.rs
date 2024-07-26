use crate::cpu::{
    r_scause, r_sstatus, turn_off_s_intr, turn_on_s_intr, w_sstatus, w_stvec, SSTATUS_SPIE,
    SSTATUS_SPP,
};
use crate::syscall;

use crate::scheduler::batch::{load_next_and_run, mark_current_runnbale};
use crate::scheduler::layout::TrapContext;
use crate::timer::clear_pending_tintr;

const S_TIMER_INT: usize = 0x1usize << 63 | 0x5;
const S_SOFT_INT: usize = 0x1usize << 63 | 0x1;
const M_TIMER_INT: usize = 0x1usize << 63 | 0x7;
const U_ECALL: usize = 0x8;

#[no_mangle]
extern "C" fn user_trap(ctx: &mut TrapContext) -> &TrapContext {
    extern "C" {
        fn __kernel_trap_vector();
    }
    assert!(r_scause() & SSTATUS_SPP == 0);
    w_stvec(__kernel_trap_vector as usize);

    let scause = r_scause();
    match scause {
        U_ECALL => {
            turn_on_s_intr();
            let syscall_id = ctx.registers[17];
            let args: [usize; 3] = ctx.registers[10..13].try_into().unwrap();
            syscall::router(syscall_id, args);
            // goto next instruction of `ecall`
            ctx.sepc += 4;
        }
        S_SOFT_INT => {
            mark_current_runnbale();
            load_next_and_run();
            clear_pending_tintr();
        }
        _ => unreachable!(),
    }
    ret_from_user_trap();
    ctx
}

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

pub fn ret_from_user_trap() {
    extern "C" {
        fn __user_trap_vector();
    }
    turn_off_s_intr();
    w_stvec(__user_trap_vector as usize);
    let mut sstatus = r_sstatus();
    // enable intr
    sstatus |= SSTATUS_SPIE;
    // make sure return to U mode
    sstatus &= !SSTATUS_SPP;
    w_sstatus(sstatus);
}
