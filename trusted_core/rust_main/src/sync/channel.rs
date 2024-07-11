use crate::scheduler::batch::load_next_and_run;

//use super::Mutex;
use super::Condvar;
//use crate::println;
use alloc::sync::Arc;
use crossbeam_queue::{SegQueue};
//use crate::scheduler::batch::load_next_and_run;
struct Channel<T>{
    inner: Arc<SegQueue<T>>,//SegQueue is unbounded
    notifier: Arc<Condvar>
}

unsafe impl<T: Send> Send for Channel<T> {}
// unsafe impl<T: Sync> Sync for Channel<T> {}

impl<T> Clone for Channel<T>{
   fn clone(&self) -> Self {
      Self {
        inner: Arc::clone(&self.inner),
        notifier: Arc::clone(&self.notifier)
      }
   } 
}

impl<T> Channel<T>{
    pub fn new() -> Self{
        Self {
            //inner: Arc::new(Mutex::new(SegQueue::new())),
            inner: Arc::new(SegQueue::new()),
            notifier: Arc::new(Condvar::new())
        }
    }
}

pub struct Sender<T>(Channel<T>);
pub struct Receiver<T>(Channel<T>);

unsafe impl<T: Send> Send for Sender<T>{}
unsafe impl<T: Send> Sync for Sender<T>{}

unsafe impl<T: Send> Send for Receiver<T>{}
unsafe impl<T: Send> Sync for Receiver<T>{}


impl<T> Clone for Sender<T>{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn new<T>() -> (Sender<T>, Receiver<T>){
    let channel = Channel::new();
    (Sender(channel.clone()), Receiver(channel))
}

impl<T> Sender<T>{
    pub fn send(&self, data: T) {
        // let channel=self.0.inner.lock();//.unwrap();
        // channel.push(data);
        self.0.inner.push(data);
        self.0.notifier.signal();
    }
}


impl<T> Receiver<T>{
    // blocking recv, if nothing to receive return None
    pub fn recv(&self) -> Option<T>{
        let channel=self.0.inner.clone();
        if channel.is_empty() {
            self.0.notifier.wait();
            load_next_and_run();
        }
        //self.0.inner.lock().pop()
         self.0.inner.pop()
    }

    pub fn nb_recv(&self) -> Option<T>{
        //self.0.inner.lock().pop()
         self.0.inner.pop()
    }
}


