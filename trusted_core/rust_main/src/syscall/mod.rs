mod io;
mod process;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;

pub fn router(id: usize, args: [usize; 3]) {
    match id {
        SYS_WRITE => io::sys_write(args[0], args[1], args[2]),
        SYS_EXIT => process::sys_exit(args[0] as i32),
        _ => unreachable!(),
    }
}
