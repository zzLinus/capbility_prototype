use crate::cpu::{r_scause, r_sstatus, SSTATUS_SPP};
use crate::syscall;

use crate::scheduler::layout::TrapContext;
use crate::kprintln;
use crate::timer::clint_reset;
use crate::scheduler::batch::{
    mark_current_runnbale, 
    load_next_and_run
};

const S_TIMER_INT: usize = 0x1usize << 63 | 0x5;
const S_SOFT_INT: usize = 0x1usize << 63 | 0x1;
const M_TIMER_INT: usize = 0x1usize << 63 | 0x7;
const U_ECALL: usize = 0x8;



#[no_mangle]
extern "C" fn s_trap(ctx: &mut TrapContext) -> &TrapContext{
    // kprintln!("scause: {:#x} =? {:#x}", ctx.scause, r_scause());
    let scause = r_scause();
    match scause{
        U_ECALL => {
            // kprintln!("syscall id: {:#x}", ctx.registers[17]);
            let syscall_id = ctx.registers[17];
            let args: [usize; 3] = ctx.registers[10..13].try_into().unwrap();
            syscall::router(syscall_id, args);
            // goto next instruction of `ecall`
            ctx.sepc += 4;
        }, 
        S_TIMER_INT => {
            // clear timer interrupt by writing to mtimecmp register
            let previous_priv = r_sstatus() & SSTATUS_SPP;
            if previous_priv == 0usize {
                // if interrupt comes from U mode, do sche
                mark_current_runnbale();
                clint_reset();
                load_next_and_run();
                kprintln!("sche to next thread");
            }
            clint_reset();
        },
        _ => unreachable!(),
    }
    ctx
}

