#![allow(unused_imports)]
mod layout;
use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
pub use layout::{KernelStack, ScheContext, TrapContext, UserStack};

#[derive(Debug, Default, Clone)]
pub struct IPCBuffer {
    pub regs: [usize; 32],
    pub extra_caps: [usize; 32],
}

#[derive(Debug)]
pub enum ThreadState {
    Runnable,
    Running,
    Exited,
    Sleep,
    Uninit,
}

pub struct TCB {
    pub(crate) k_stack: KernelStack,
    pub(crate) u_stack: UserStack,
    pub(crate) sche_ctx: ScheContext,
    pub(crate) state: ThreadState,
    pub(crate) ipc_buf: Box<IPCBuffer>,
}

impl Debug for TCB {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TCB")
            .field("kernel sp", &format_args!("{:#x}", self.k_stack.get_sp()))
            .field("user sp", &format_args!("{:#x}", self.u_stack.get_sp()))
            .field(
                "sche context",
                &format_args!(
                    "ra = 0x{:#x} sp = 0x{:#x}",
                    self.sche_ctx.ra, self.sche_ctx.sp
                ),
            )
            .field("state", &self.state)
            .finish()
    }
}

impl TCB {
    pub fn new(k_stack: &'static [u8], u_stack: &'static [u8], code_addr: usize) -> Self {
        let (k_stack, u_stack) = (KernelStack::new(k_stack), UserStack::new(u_stack));
        let init_trap_context = TrapContext::init_user_context(u_stack.get_sp(), code_addr);
        let sp = k_stack.push_context(init_trap_context) as *const _ as usize;
        extern "C" {
            fn __restore_context();
        }
        let sche_ctx = ScheContext::init_with(__restore_context as usize, sp);
        Self {
            k_stack,
            u_stack,
            sche_ctx,
            state: ThreadState::Uninit,
            ipc_buf: Box::new(IPCBuffer::default()),
        }
    }
}
