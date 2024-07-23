use crate::cpu;
const PAGE_SIZE: usize = 4096;
const KERNEL_STACK_SIZE: usize = PAGE_SIZE;
const USER_STACK_SIZE: usize = PAGE_SIZE;

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


#[repr(align(4096))]
pub struct KernelStack{
    pub mem: [u8; KERNEL_STACK_SIZE]
}

impl KernelStack{
    pub fn new() -> Self {
        Self {mem: [0; KERNEL_STACK_SIZE]}
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

#[repr(align(4096))]
pub struct UserStack{
    pub mem: [u8; USER_STACK_SIZE]
}

impl UserStack{
    pub fn new() -> Self{
        Self {mem: [0; USER_STACK_SIZE]}
    }
    pub fn get_sp(&self) -> usize{
        self.mem.as_ptr() as usize + USER_STACK_SIZE
    }
}

