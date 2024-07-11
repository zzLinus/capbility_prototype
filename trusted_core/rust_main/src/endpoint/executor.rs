//use super::waker;
use alloc::boxed::Box;
//use crossbeam_queue::SegQueue;
use crate::println;
use crate::sync::mutex::Mutex;
use conquer_once::spin::OnceCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use lazy_static::lazy_static;

use alloc::sync::Arc;

lazy_static! {
    pub static ref KERNEL_EXECUTOR: CapsuleExecutor = CapsuleExecutor::new();
}

// pub static KERNEL_EXECUTOR: OnceCell<CapsuleExecutor> = OnceCell::uninit();
//
// pub fn init_kernel_executor() {
//     KERNEL_EXECUTOR
//         .try_init_once(|| CapsuleExecutor::new())
//         .expect("KERNEL_EXECUTOR already initialized");
// }

type BoxedFut = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/*
    queue: holding type erased trait object ptr with Output = ()

    for an incoming future: F
    wrap it into MaybeDone<F>, which is generic over F
                    | -> register a handler here
                    | <- type erased it
    CapsuleNode contains type erased future and notifier
*/

pub(crate) struct CapsuleNode {
    pub fut: Mutex<BoxedFut>,
    pub tx: Sender<Arc<Self>>,
}

pub struct CapsuleHandle<R: Send> {
    return_data: Receiver<R>,
}

impl<R: Send> CapsuleHandle<R> {
    fn new(rx: Receiver<R>) -> Self {
        Self { return_data: rx }
    }
}

pub trait IntoCapsule {
    type Output;
    fn resolve(self) -> impl Future<Output = Self::Output> + Send + 'static;

    fn add_to_job_queue(self) -> CapsuleHandle<Self::Output>
    where
        Self: Sized,
        <Self as IntoCapsule>::Output: Send,
    {
        // let executor = KERNEL_EXECUTOR
        //     .get()
        //     .expect("KERNEL_EXECUTOR not initialized");
        KERNEL_EXECUTOR.atomic_push(self.resolve())
        //KERNEL_EXECUTOR.atomic_push(self.resolve())
    }
}

use super::waker::ChannelWaker;
use crate::sync::{self, Receiver, Sender};

impl CapsuleNode {
    fn new(fut: BoxedFut, tx: &Sender<Arc<Self>>) -> Self {
        Self {
            fut: Mutex::new(fut),
            tx: tx.clone(),
        }
    }
}

pub struct CapsuleExecutor {
    tx: Sender<Arc<CapsuleNode>>,
    ready_queue: Receiver<Arc<CapsuleNode>>,
}

// interior mutability required
// this struct is used as static
impl CapsuleExecutor {
    fn new() -> Self {
        let (tx, rx) = sync::new();
        Self {
            tx,
            ready_queue: rx,
        }
    }

    fn atomic_push<F>(&self, fut: F) -> CapsuleHandle<F::Output>
    where
        F: Future + 'static + Send,
        F::Output: Send,
    {
        let (join_tx, join_rx) = sync::new();
        let task = async move {
            let out = fut.await;
            join_tx.send(out);
        };
        let new_node: Arc<CapsuleNode> = Arc::new(CapsuleNode::new(Box::pin(task), &self.tx));
        self.tx.send(new_node);
        CapsuleHandle::new(join_rx)
    }

    pub fn exec(&self) {
        loop {
            // blocking recv
            if let Some(next_to_do) = self.ready_queue.recv() {
                // waker should be thread safe <- rust requirement
                let waker = Arc::new(ChannelWaker::new(Arc::clone(&next_to_do))).into();
                match next_to_do
                    .fut
                    .lock()
                    .as_mut()
                    .poll(&mut Context::from_waker(&waker))
                {
                    Poll::Pending => println!("this one is pending"),
                    Poll::Ready(_) => println!("finish one"),
                };
            }
        }
    }

    ///non-blocking executing logic for now
    pub fn nb_exec(&self) {
        while let Some(todo) = self.ready_queue.nb_recv() {
            // waker should be thread safe <- rust requirement
            let waker = Arc::new(ChannelWaker::new(Arc::clone(&todo))).into();
            match todo
                .fut
                .lock()
                .as_mut()
                .poll(&mut Context::from_waker(&waker))
            {
                Poll::Pending => println!("this one is pending"),
                Poll::Ready(_) => println!("finish one"),
            };
        }
    }
}

pub fn block_on<R: Send>(handle: CapsuleHandle<R>) -> Result<R, ()> {
    //add error propagation
    match handle.return_data.recv() {
        Some(a) => Ok(a),
        None => Err(()),
    }
}
