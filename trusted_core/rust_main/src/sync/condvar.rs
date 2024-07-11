use crate::scheduler::batch::{block_task, load_next_and_run, wake_task};
use alloc::sync::Arc;
//use crate::thread::TCB;
use crossbeam_queue::SegQueue;
//use spin::MutexGuard;
pub struct Condvar {
    pub queue: Arc<SegQueue<usize>>,
}

impl Condvar {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    pub fn wait(&self) {
        let id = block_task();
        self.queue.push(id);
        //schedule
        //load_next_and_run();
    }

    pub fn signal(&self) {
        if let Some(id) = self.queue.pop() {
            wake_task(id);
        }
    }
}
