use core::arch::asm;

//Ecall no.
pub const E_TIMER: usize = 1;

pub fn do_ecall(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
    arg7: usize,
) {
    unsafe {
        asm!("mv a0, {0}", in(reg) arg0);
        asm!("mv a1, {0}", in(reg) arg1);
        asm!("mv a2, {0}", in(reg) arg2);
        asm!("mv a3, {0}", in(reg) arg3);
        asm!("mv a4, {0}", in(reg) arg4);
        asm!("mv a5, {0}", in(reg) arg5);
        asm!("mv a6, {0}", in(reg) arg6);
        asm!("mv a7, {0}", in(reg) arg7);
        asm!("ecall");
    }
}
