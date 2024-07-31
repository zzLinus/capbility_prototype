mod io;
mod process;
use crate::kernel_object::endpoint::test_ep;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_TESTEP: usize = 0;

pub fn router(id: usize, args: [usize; 3]) {
    match id {
        SYS_WRITE => io::sys_write(args[0], args[1], args[2]),
        SYS_EXIT => process::sys_exit(args[0] as i32),
        SYS_TESTEP => test_ep(),
        _ => unreachable!(),
    }
}
