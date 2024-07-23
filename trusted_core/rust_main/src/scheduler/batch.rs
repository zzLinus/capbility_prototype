use core::slice;
use core::arch::asm;

use alloc::vec::Vec;
use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::mutex::Mutex;
use crate::kprintln;
use super::layout::{
    TrapContext,
    UserStack, KernelStack
};


pub struct BatchScheduler{
    next_to_run_id: usize,
    num_app: usize,
    app_addr_space: Vec<usize>
}

const MAX_NUM_APP: usize = 32;
const APP_TEXT_BASE_ADDR: usize = 0x8040_0000;


static KERNEL_STACK: KernelStack = KernelStack{
    mem: [0; 4096]
};
static USER_STACK: UserStack = UserStack {
    mem: [0; 4096]
};

// introduce interior mutability + thread safety throught Arc<Mutex>
lazy_static! {
    static ref SCHEDULER: Arc<Mutex<BatchScheduler>> = Arc::new(Mutex::new(BatchScheduler::new()));
}

impl BatchScheduler{
    pub fn new() -> Self{
        extern "C" {
            fn _num_app();
        }
        // SAFETY: _num_app specifies the address of #app, constructed by ../build.rs
        let mut ptr = _num_app as *mut usize;
        let num_app = unsafe {ptr.read_volatile()};
        let mut app_addr_space = unsafe {
            ptr.add(1);
            // +1 due to the last end address
            Vec::from_raw_parts(ptr.add(1), num_app+1, MAX_NUM_APP)
        };
        Self {
            num_app, app_addr_space, 
            next_to_run_id: 0
        }
    }

    // move app to the base address
    pub fn load_next_app(&mut self){
        let (s_addr, e_addr) = (self.app_addr_space[self.next_to_run_id], self.app_addr_space[self.next_to_run_id+1]);
        let num_bytes = e_addr - s_addr;

        // SAFETY: dst points to the region starting from 0x8000_4000, always valid
        let dst = unsafe {
            slice::from_raw_parts_mut(APP_TEXT_BASE_ADDR as *mut u8, num_bytes)
        };
        let src = unsafe {
            slice::from_raw_parts(s_addr as *const u8, num_bytes)
        };
        dst.copy_from_slice(src);
        self.next_to_run_id += 1;

        // SAFETY: flush instruction cache
        unsafe {asm!("fence.i")};
    }

    pub fn dump_app_info(&self) {
        for i in 0..self.num_app{
            let (s_addr, e_addr) = (self.app_addr_space[i], self.app_addr_space[i+1]);
            kprintln!("[app {}] {:#x} -> {:#x}", i, s_addr, e_addr);
        }
    }
}

pub fn dump_app_info() {
    SCHEDULER.lock().dump_app_info();
}

pub fn load_next_and_run() {
    // force sche to drop, because this function will jump to restore, default drop won't be invoked
    {
        let mut sche = SCHEDULER.lock();
        sche.load_next_app();
    }

    extern "C" {
        fn __restore_context(ctx_addr: usize);
    }
    let sp_top = USER_STACK.get_sp();
    let mut init_context = TrapContext::init_user_context(sp_top, APP_TEXT_BASE_ADDR);

    kprintln!("user stack: {:#x}", USER_STACK.get_sp());
    kprintln!("init ker sp: {:#x}", KERNEL_STACK.get_sp());

    // for now, every app is loaded to a fixed address
    let ctx_ptr = KERNEL_STACK.push_context(init_context) as *const _;

    unsafe {
        __restore_context(ctx_ptr as usize);
    }
}

