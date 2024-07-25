use core::arch::asm;

//MSTATUS
pub const MSTATUS_MPP_MASK: usize = 0b11 << 11;
pub const MSTATUS_MPP_M: usize = 0b11 << 11;
pub const MSTATUS_MPP_S: usize = 0b1 << 11;
pub const MSTATUS_MPP_U: usize = 0b0 << 11;
pub const MSTATUS_MIE: usize = 0b1 << 3;

//SSTATUS
pub const SSTATUS_SPP: usize = 0b1 << 8;
pub const SSTATUS_SPIE: usize = 0b1 << 5;
pub const SSTATUS_UPIE: usize = 0b1 << 4;
pub const SSTATUS_SIE: usize = 0b1 << 1;
pub const SSTATUS_UIE: usize = 0b1 << 0;

//SIE
pub const SIE_SEIE: usize = 0b1 << 9;
pub const SIE_STIE: usize = 0b1 << 5;
pub const SIE_SSIE: usize = 0b1 << 1;

// SIP
pub const SIP_SSIP: usize = 0b1 << 1;
pub const SIP_STIP: usize = 0b1 << 5;

//MIE
pub const MIE_MEIE: usize = 0b1 << 11;
pub const MIE_MTIE: usize = 0b1 << 7;
pub const MIE_MSIE: usize = 0b1 << 3;

//sv39 -> SATP
pub const SATP_SV39: usize = 0b1000 << 60;
pub const SATP_SV48: usize = 0b1001 << 60;

//PAGE
pub const PGSIZE: usize = 4096;
pub const PGSHIFT: usize = 12;

pub const PTE_V: usize = 0b1 << 0;
pub const PTE_R: usize = 0b1 << 1;
pub const PTE_W: usize = 0b1 << 2;
pub const PTE_X: usize = 0b1 << 3;
pub const PTE_U: usize = 0b1 << 4;

pub const MAXVA: usize = 0b1 << (9 + 9 + 9 + 12 - 1);

pub fn r_mhartid() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, mhartid", out(reg) rval);
        return rval;
    }
}

pub fn r_mstatus() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, mstatus", out(reg) rval);
        return rval;
    }
}

pub fn w_mstatus(wval: usize) {
    unsafe {
        asm!("csrw mstatus, {0}", in(reg) wval);
    }
}

pub fn w_mepc(wval: usize) {
    unsafe {
        asm!("csrw mepc, {0}", in(reg) wval);
    }
}

pub fn r_sstatus() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, sstatus", out(reg) rval);
        return rval;
    }
}

pub fn w_sstatus(wval: usize) {
    unsafe {
        asm!("csrw sstatus, {0}", in(reg) wval);
    }
}

pub fn r_sip() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, sip", out(reg) rval);
        return rval;
    }
}

pub fn w_sip(wval: usize) {
    unsafe {
        asm!("csrw sip, {0}", in(reg) wval);
    }
}

pub fn r_sie() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, sie", out(reg) rval);
        return rval;
    }
}

pub fn w_sie(wval: usize) {
    unsafe {
        asm!("csrw sie, {0}", in(reg) wval);
    }
}

pub fn r_mie() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, mie", out(reg) rval);
        return rval;
    }
}

pub fn w_mie(wval: usize) {
    unsafe {
        asm!("csrw sie, {0}", in(reg) wval);
    }
}

pub fn r_sepc() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, sepc", out(reg) rval);
        return rval;
    }
}

pub fn w_sepc(wval: usize) {
    unsafe {
        asm!("csrw sepc, {0}", in(reg) wval);
    }
}

pub fn r_medeleg() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, medeleg", out(reg) rval);
        return rval;
    }
}

pub fn w_medeleg(wval: usize) {
    unsafe {
        asm!("csrw medeleg, {0}", in(reg) wval);
    }
}

pub fn r_mideleg() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, mideleg", out(reg) rval);
        return rval;
    }
}

pub fn w_mideleg(wval: usize) {
    unsafe {
        asm!("csrw mideleg, {0}", in(reg) wval);
    }
}

pub fn r_stvec() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, stvec", out(reg) rval);
        return rval;
    }
}

pub fn w_stvec(wval: usize) {
    unsafe {
        asm!("csrw stvec, {0}", in(reg) wval);
    }
}

pub fn r_mtvec() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, mtvec", out(reg) rval);
        return rval;
    }
}

pub fn w_mtvec(wval: usize) {
    unsafe {
        asm!("csrw mtvec, {0}", in(reg) wval);
    }
}

pub fn w_pmpcfg0(wval: usize) {
    unsafe {
        asm!("csrw pmpcfg0, {0}", in(reg) wval);
    }
}

pub fn w_pmpaddr0(wval: usize) {
    unsafe {
        asm!("csrw pmpaddr0, {0}", in(reg) wval);
    }
}

pub fn w_satp(wval: usize) {
    unsafe {
        asm!("csrw satp, {0}", in(reg) wval);
    }
}

pub fn r_satp() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, satp", out(reg) rval);
        return rval;
    }
}

pub fn w_sscratch(wval: usize) {
    unsafe {
        asm!("csrw sscratch, {0}", in(reg) wval);
    }
}

pub fn w_mscratch(wval: usize) {
    unsafe {
        asm!("csrw mscratch, {0}", in(reg) wval);
    }
}

pub fn r_scause() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, scause", out(reg) rval);
        return rval;
    }
}

pub fn r_stval() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, stval", out(reg) rval);
        return rval;
    }
}

pub fn w_mcounteren(wval: usize) {
    unsafe {
        asm!("csrw mcounteren, {0}", in(reg) wval);
    }
}

pub fn r_mcounteren() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, mcounteren", out(reg) rval);
        return rval;
    }
}

pub fn r_time() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, time", out(reg) rval);
        return rval;
    }
}

pub fn intr_on() {
    w_sstatus(r_sstatus() | SSTATUS_SIE);
}

pub fn intr_off() {
    w_sstatus(r_sstatus() & !SSTATUS_SIE);
}

pub fn intr_get() -> bool {
    let x = r_sstatus();
    (x & SSTATUS_SIE) != 0
}

pub fn r_sp() -> usize {
    unsafe {
        let rval;
        asm!("mv {0}, sp", out(reg) rval);
        return rval;
    }
}

pub fn r_tp() -> usize {
    unsafe {
        let rval;
        asm!("mv {0}, tp", out(reg) rval);
        return rval;
    }
}

pub fn w_tp(wval: usize) {
    unsafe {
        asm!("mv tp, {0}", in(reg) wval);
    }
}

pub fn r_ra() -> usize {
    unsafe {
        let rval;
        asm!("mv {0}, ra", out(reg) rval);
        return rval;
    }
}

pub fn sfence_vma() {
    unsafe {
        asm!("sfence.vma zero, zero");
    }
}

pub fn r_() -> usize {
    unsafe {
        let rval;
        asm!("csrr {0}, sie", out(reg) rval);
        return rval;
    }
}

pub fn w_(wval: usize) {
    unsafe {
        asm!("csrw sip, {0}", in(reg) wval);
    }
}

#[inline]
pub fn turn_off_s_intr() {
    w_sscratch(r_sstatus() & (!SSTATUS_SIE));
}

#[inline]
pub fn turn_on_s_intr() {
    w_sscratch(r_sstatus() | SSTATUS_SIE);
}
