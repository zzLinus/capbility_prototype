use crate::cpu;
use crate::config::*;

#[repr(C)]
pub struct TrapContext{
    pub registers: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
    pub stval: usize,
    pub scause: usize
}

impl TrapContext{
    #[inline]
    pub fn set_sp(&mut self, sp: usize){
        self.registers[2] = sp;
    }
    #[inline]
    pub fn set_sepc(&mut self, sepc: usize) {
        self.sepc = sepc;
    }

    // set privilege level to user mode
    #[inline]
    pub fn set_user_spp() -> usize{
        let mut sstatus = 1usize << 33;
        sstatus &= !(cpu::SSTATUS_SPP);
        sstatus
    }

    pub fn init_user_context(sp: usize, sepc: usize) -> Self{
        let sstatus = Self::set_user_spp();
        let mut registers = [0; 32];
        registers[2] = sp;
        Self {
            registers, sstatus, sepc, 
            stval: 0, scause: 0
        }

    }
}


// #[repr(align(4096))]
pub struct KernelStack{
    pub mem: &'static [u8]
}

impl KernelStack{
    pub fn new(mem: &'static [u8]) -> Self {
        Self {mem}
    }
    // interior mutability required
    pub fn push_context(&self, context: TrapContext) -> &'static TrapContext{
        let stack_top = self.get_sp();
        let ptr = (stack_top - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

        // SAFETY: ptr is aligned and valid
        unsafe {
            *ptr = context;
            ptr.as_mut().unwrap()
        }
    }

    pub fn get_sp(&self) -> usize{
        self.mem.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

// #[repr(align(4096))]
pub struct UserStack{
    pub mem: &'static [u8]
}

impl UserStack{
    pub fn new(mem: &'static [u8]) -> Self{
        Self {mem}
    }
    pub fn get_sp(&self) -> usize{
        self.mem.as_ptr() as usize + USER_STACK_SIZE
    }
}


// context when invoking half way through scheduler's sche
#[repr(C)]
pub struct ScheContext{
    pub(crate) ra: usize, 
    pub(crate) sp: usize,
    // (crate)callee saved registers
    pub(crate) s: [usize; 12]
}



impl ScheContext {
    pub fn init_with(ra: usize, sp: usize) -> Self{
        Self {ra, sp, s: [0; 12]}
    }
}
