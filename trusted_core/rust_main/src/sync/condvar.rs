use crate::scheduler::batch::{block_task, wake_task};
use alloc::sync::Arc;
use crossbeam_queue::SegQueue;
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
    }

    pub fn signal(&self) {
        if let Some(id) = self.queue.pop() {
            wake_task(id);
        }
    }
}
