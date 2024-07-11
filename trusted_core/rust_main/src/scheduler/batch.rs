use core::arch::{asm, global_asm};
use core::slice;

use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;

use super::layout::ScheContext;
use crate::endpoint::executor::KERNEL_EXECUTOR;
use crate::kprintln;
use crate::sync::Mutex;
use crate::thread::{ThreadState, TCB};

pub struct BatchScheduler {
    current_id: usize,
    app_code_addr: Vec<usize>,
    num_app: usize,
    tasks: Vec<TCB>,
}
use crate::config::*;

// introduce interior mutability + thread safety throught Arc<Mutex>
lazy_static! {
    static ref SCHEDULER: Arc<Mutex<BatchScheduler>> = Arc::new(Mutex::new(BatchScheduler::new()));
    // TODO: lazy static init of KERNEL_STACK somehow fails
}

#[repr(align(4096))]
struct GlobalKernelStack([u8; KERNEL_STACK_SIZE * MAX_NUM_TASK]);
#[repr(align(4096))]
struct GlobalUserStack([u8; USER_STACK_SIZE * MAX_NUM_TASK]);

static KERNEL_STACKS: GlobalKernelStack = GlobalKernelStack([0; KERNEL_STACK_SIZE * MAX_NUM_TASK]);
static USER_STACKS: GlobalUserStack = GlobalUserStack([0; USER_STACK_SIZE * MAX_NUM_TASK]);

global_asm!(include_str!("switch.S"));

impl BatchScheduler {
    pub fn new() -> Self {
        extern "C" {
            fn _num_app();
        }
        let ptr = _num_app as *const usize;
        // SAFETY: ptr is a valid ptr
        let num_app = unsafe { ptr.read_volatile() };

        // layout: [saddr of app0, sddr of app1, ..., eddr of app N]
        let app_code_addr =
            unsafe { Vec::from_raw_parts(ptr.add(1) as *mut usize, num_app + 1, MAX_NUM_TASK) };

        // potential bug in lazy_static: avoid entering into large memory alloc postone this
        // TODO: fix this logic
        // let tasks: Vec<TCB> = (0..num_app).map(|id| {
        //     let (s_addr, e_addr) = (app_code_addr[id], app_code_addr[id+1]);
        //     let text_addr = Self::load_app(id, s_addr, e_addr);
        //     TCB::new(text_addr)
        // }).collect();

        // inst fence: make sure text load takes effect
        unsafe { asm!("fence.i") };
        Self {
            num_app,
            current_id: 0,
            app_code_addr,
            tasks: Vec::new(),
        }
    }

    fn init_tcbs(&mut self) {
        self.tasks = (0..self.num_app)
            .map(|id| {
                let (s_addr, e_addr) = (self.app_code_addr[id], self.app_code_addr[id + 1]);
                let code_addr = Self::load_app(id, s_addr, e_addr);
                let k_stack =
                    &KERNEL_STACKS.0[id * KERNEL_STACK_SIZE..(id + 1) * KERNEL_STACK_SIZE];
                let u_stack = &USER_STACKS.0[id * USER_STACK_SIZE..(id + 1) * USER_STACK_SIZE];
                TCB::new(k_stack, u_stack, code_addr)
            })
            .collect();
    }

    fn load_app(id: usize, s_addr: usize, e_addr: usize) -> usize {
        let num_bytes = e_addr - s_addr;
        assert!(num_bytes <= TASK_TEXT_LIMIT);
        let src = unsafe { slice::from_raw_parts(s_addr as *const u8, num_bytes) };
        let dst_s_addr = (TASK_TEXT_BASE_ADDR + id * TASK_TEXT_LIMIT) as *mut u8;
        let dst = unsafe { slice::from_raw_parts_mut(dst_s_addr, TASK_TEXT_LIMIT) };
        dst[..num_bytes].copy_from_slice(src);
        // fill the rest with zero
        dst[num_bytes..].fill(0u8);

        dst_s_addr as usize
    }

    fn find_next_runnable(&self) -> Option<usize> {
        let num_tasks = self.tasks.len();
        for id in self.current_id + 1..self.current_id + num_tasks + 1 {
            match self.tasks[id % num_tasks].state {
                ThreadState::Runnable => return Some(id % num_tasks),
                _ => {}
            }
        }
        None
    }

    pub fn dump_app_info(&self) {
        for (id, tcb) in self.tasks.iter().enumerate() {
            kprintln!("[app {}]: {:?}", id, tcb);
        }
    }
}

pub fn dump_app_info() {
    SCHEDULER.lock().dump_app_info();
}

pub fn load_next_and_run() {
    // let executor = KERNEL_EXECUTOR
    //     .get()
    //     .expect("KERNEL_EXECUTOR not initialized");
    KERNEL_EXECUTOR.nb_exec();
    //KERNEL_EXECUTOR.nb_exec();
    let mut sche = SCHEDULER.lock();
    sche.dump_app_info();
    extern "C" {
        fn __switch(src: usize, dst: usize);
    }
    // state of current running thread should be changed prior entering `sche`
    match sche.find_next_runnable() {
        Some(switch_dst) => {
            let src = &sche.tasks[sche.current_id].sche_ctx as *const _ as usize;
            sche.current_id = switch_dst;
            let dst_tcb = &mut sche.tasks[switch_dst];
            dst_tcb.state = ThreadState::Running;
            let dst = &dst_tcb.sche_ctx as *const _ as usize;
            drop(sche);
            // SAFETY: src and dst are properly inited
            unsafe {
                __switch(src, dst);
            }
        }
        None => panic!("no more tasks to run, all finished"),
    }
}
pub fn init_task() {
    // currently just takes the first and run
    // hack: set first src to be a dummy ctx, which will be released when exit the func
    let dummy_sche_ctx = ScheContext::init_with(0, 0);

    // force sche to drop
    let dst_sche_ctx = {
        let mut sche = SCHEDULER.lock();
        sche.init_tcbs();
        let current_id = sche.current_id;
        let first_tcb = &mut sche.tasks[current_id];
        first_tcb.state = ThreadState::Running;
        &first_tcb.sche_ctx as *const _ as usize
    };
    kprintln!("finish loading task");
    extern "C" {
        fn __switch(src: usize, dst: usize);
    }
    unsafe {
        __switch(&dummy_sche_ctx as *const _ as usize, dst_sche_ctx);
    }
}

pub fn mark_current_exited() {
    let mut sche = SCHEDULER.lock();
    let current_id = sche.current_id;
    sche.tasks[current_id].state = ThreadState::Exited;
}

pub fn mark_current_runnbale() {
    let mut sche = SCHEDULER.lock();
    let current_id = sche.current_id;
    sche.tasks[current_id].state = ThreadState::Runnable;
}

pub fn wake_task(id: usize) {
    let mut sche = SCHEDULER.lock();
    sche.tasks[id].state = ThreadState::Runnable;
}

pub fn block_task() -> usize {
    let mut sche = SCHEDULER.lock();
    let current_id = sche.current_id;
    sche.tasks[current_id].state = ThreadState::Sleep;
    current_id
}
