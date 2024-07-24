use crate::cpu::r_sstatus;
use crate::cpu::w_sstatus;
use crate::cpu::SSTATUS_SIE;

use crate::cpu::{r_sip, w_sip, SIP_SSIP};
use crate::ecall::do_ecall;
use crate::ecall::E_TIMER;

pub const CLINT_MTIME: usize = 0x200BFF8;
pub const CLINT_CMP: usize = 0x2004000;
pub const CMP_COUNT: usize = 500000;
pub static mut TIMER_COUNT: usize = 0;

pub fn clint_init() {
    let cmp_ptr: *mut usize = CLINT_CMP as *mut usize;
    let mtime_ptr: *mut usize = CLINT_MTIME as *mut usize;
    unsafe {
        TIMER_COUNT = 0;
        cmp_ptr.write_volatile(mtime_ptr.read_volatile() + CMP_COUNT);
    }
}

#[inline]
pub fn clear_pending_tintr() {
    // timer intr in S mode is raised by set software intr in M mode
    w_sip(r_sip() & (!SIP_SSIP))
}

#[inline]
pub fn clint_set_cmp() {
    let cmp_ptr: *mut usize = CLINT_CMP as *mut usize;
    let mtime_ptr: *mut usize = CLINT_MTIME as *mut usize;
    unsafe {
        cmp_ptr.write_volatile(mtime_ptr.read_volatile() + CMP_COUNT);
    }
}

pub fn read_mtime() -> usize {
    let mtime_ptr: *mut usize = CLINT_MTIME as *mut usize;
    unsafe {
        let rval: usize = mtime_ptr.read_volatile();
        return rval;
    }
}

pub fn s_timer_trap() {
    unsafe {
        TIMER_COUNT += 1;
    }
    clint_set_cmp();
    ret_from_stimer();
}

pub fn ret_from_stimer() {
    do_ecall(0, 0, 0, 0, 0, 0, 0, E_TIMER);
    w_sstatus(r_sstatus() | SSTATUS_SIE);
}

pub fn clint_reset() {
    clint_set_cmp();
    ret_from_stimer();
}
