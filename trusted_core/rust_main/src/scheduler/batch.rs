use alloc::vec::Vec;
use alloc::sync::Arc;
use crate::mutex::Mutex;
use crate::println;

use lazy_static::lazy_static;

pub struct BatchScheduler{
    next_to_run_id: usize,
    num_app: usize,
    app_addr_space: Vec<usize>
}

const MAX_NUM_APP: usize = 32;


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

    pub fn dump_app_info(&self) {
        for i in 0..self.num_app{
            let (s_addr, e_addr) = (self.app_addr_space[i], self.app_addr_space[i+1]);
            println!("[app {}] {:#x} -> {:#x}", i, s_addr, e_addr);
        }
    }
}

pub fn dump_app_info() {
    SCHEDULER.lock().dump_app_info();
}

