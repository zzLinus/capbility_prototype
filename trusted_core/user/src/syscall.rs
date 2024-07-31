use core::arch::asm;
/*
    riscv syscall convention
    x17: syscall id
    x10-x16: arguments
    x10: also serves as ret
*/

pub fn syscall(id: usize, args: [usize; 3]) -> isize{
    let ret: isize;
    unsafe {
        asm!(
            "ecall", 
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        )
    }
    ret
}

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_TESTEP:usize =0;

pub fn sys_write(fd: usize, buf: &[u8]) -> isize{
    syscall(SYS_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}
pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYS_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_testep()-> isize{
    syscall(SYS_TESTEP, [0,0,0])
}
