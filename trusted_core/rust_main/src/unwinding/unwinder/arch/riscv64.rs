use core::arch::asm;
use core::fmt;
use core::ops;
use gimli::{Register, RiscV};

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; 32],
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=31 {
            fmt.field(RiscV::register_name(Register(i as _)).unwrap(), &self.gp[i]);
        }
        fmt.finish()
    }
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=31) => &self.gp[reg.0 as usize],
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=31) => &mut self.gp[reg.0 as usize],
            _ => unimplemented!(),
        }
    }
}

macro_rules! code {
    (save_gp) => {
        "
        sd x0, 0x00(sp)
        sd ra, 0x08(sp)
        sd t0, 0x10(sp)
        sd gp, 0x18(sp)
        sd tp, 0x20(sp)
        sd s0, 0x40(sp)
        sd s1, 0x48(sp)
        sd s2, 0x90(sp)
        sd s3, 0x98(sp)
        sd s4, 0xA0(sp)
        sd s5, 0xA8(sp)
        sd s6, 0xB0(sp)
        sd s7, 0xB8(sp)
        sd s8, 0xC0(sp)
        sd s9, 0xC8(sp)
        sd s10, 0xD0(sp)
        sd s11, 0xD8(sp)
        "
    };
    (restore_gp) => {
        "
        ld ra, 0x08(a0)
        ld sp, 0x10(a0)
        ld gp, 0x18(a0)
        ld tp, 0x20(a0)
        ld t0, 0x28(a0)
        ld t1, 0x30(a0)
        ld t2, 0x38(a0)
        ld s0, 0x40(a0)
        ld s1, 0x48(a0)
        ld a1, 0x58(a0)
        ld a2, 0x60(a0)
        ld a3, 0x68(a0)
        ld a4, 0x70(a0)
        ld a5, 0x78(a0)
        ld a6, 0x80(a0)
        ld a7, 0x88(a0)
        ld s2, 0x90(a0)
        ld s3, 0x98(a0)
        ld s4, 0xA0(a0)
        ld s5, 0xA8(a0)
        ld s6, 0xB0(a0)
        ld s7, 0xB8(a0)
        ld s8, 0xC0(a0)
        ld s9, 0xC8(a0)
        ld s10, 0xD0(a0)
        ld s11, 0xD8(a0)
        ld t3, 0xE0(a0)
        ld t4, 0xE8(a0)
        ld t5, 0xF0(a0)
        ld t6, 0xF8(a0)
        "
    };
}

#[naked]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    // No need to save caller-saved registers here.
    unsafe {
        asm!(
            "
            mv t0, sp
            add sp, sp, -0x110
            sd ra, 0x100(sp)
            ",
            code!(save_gp),
            "
            mv t0, a0
            mv a0, sp
            jalr t0
            ld ra, 0x100(sp)
            add sp, sp, 0x110
            ret
            ",
            options(noreturn)
        );
    }
}

pub unsafe fn restore_context(ctx: &Context) -> ! {
    unsafe {
        asm!(
            code!(restore_gp),
            "
            ld a0, 0x50(a0)
            ret
            ",
            in("a0")ctx,
            options(noreturn)
        );
    }
}
