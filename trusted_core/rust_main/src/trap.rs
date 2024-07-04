use crate::cpu::r_scause;
use crate::syscall;

use crate::scheduler::layout::TrapContext;
use crate::kprintln;

const SUPERVISOR_TIMER_INTERRUPT: usize = 0x8000000000000005;


const ECALL_FROM_U: usize = 0x8;



#[no_mangle]
extern "C" fn s_trap(ctx: &mut TrapContext) -> &TrapContext{
    kprintln!("syscall id: {:#x}", ctx.registers[17]);
    kprintln!("scause: {:#x} =? {:#x}", ctx.scause, r_scause());
    let scause = r_scause();
    match scause{
        ECALL_FROM_U => {
            let syscall_id = ctx.registers[17];
            let args: [usize; 3] = ctx.registers[10..13].try_into().unwrap();
            syscall::router(syscall_id, args);
            ctx.sepc += 4;
        } 
        _ => unreachable!(),
    }
    ctx
}

