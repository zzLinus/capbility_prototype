/// CapsuleNode Executor
///
/// Designed as an async executor to constantly drive the task binded to CapsuleNode to finish.
/// Task can be registered by impl the `IntoCapsule ` trait (see examples in `Endpoint`).
/// Currently, the executor follows a FIFO policy and greedily polls the runnable Capsule till no further progress can be made.
///
/// Ochestration with TCB scheduler
/// When a scheduling request is made, `KERNEL_EXECUTOR` will first probe the pending CapSuleNode and drive all of them to finish
/// before any TCB level scheduling takes place.
use super::waker::ChannelWaker;
use crate::sync::mutex::Mutex;
use crate::sync::{self, Receiver, Sender};
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use lazy_static::lazy_static;

use alloc::sync::Arc;
use log::info;

lazy_static! {
    pub static ref KERNEL_EXECUTOR: CapsuleExecutor = CapsuleExecutor::new();
}

type BoxedFut = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub(crate) struct CapsuleNode {
    pub fut: Mutex<BoxedFut>,
    pub tx: Sender<Arc<Self>>,
}

/// Channel based return value hook
/// caller can block on this handle for the return value to complete
/// Capsule Executor is responsible for inserting the requested return vale into this channel
pub struct CapsuleHandle<R: Send> {
    return_data: Receiver<R>,
}

impl<R: Send> CapsuleHandle<R> {
    fn new(rx: Receiver<R>) -> Self {
        Self { return_data: rx }
    }

    pub fn try_take_data(&self) -> Option<R> {
        self.return_data.nb_recv()
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
        KERNEL_EXECUTOR.atomic_push(self.resolve())
    }
}

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

/// interior mutability required for this struct is used as static
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

    fn try_poll_next(next_to_do: Arc<CapsuleNode>) {
        let waker = Arc::new(ChannelWaker::new(Arc::clone(&next_to_do))).into();
        match next_to_do
            .fut
            .lock()
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
        {
            Poll::Pending => {
                info!("task is pending")
            }
            Poll::Ready(_) => {
                info!("task finished")
            }
        };
    }

    pub fn exec(&self) {
        while let Some(next_to_do) = self.ready_queue.recv() {
            Self::try_poll_next(next_to_do);
        }
    }

    /// non blocking recv, try to probe the associated ready queue, if there is nothing, immediately return
    pub fn nb_exec(&self) {
        while let Some(next_to_do) = self.ready_queue.nb_recv() {
            Self::try_poll_next(next_to_do);
        }
    }
}

pub fn block_on<R: Send>(handle: CapsuleHandle<R>) -> Result<R, ()> {
    handle.return_data.recv().ok_or(())
}
