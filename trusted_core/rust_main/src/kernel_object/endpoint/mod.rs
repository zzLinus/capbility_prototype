pub mod executor;
pub use executor::KERNEL_EXECUTOR;
mod waker;
use alloc::boxed::Box;
use executor::{CapsuleHandle, IntoCapsule};


#[derive(Debug, Default, Clone)]
pub struct IPCBuffer {
    regs: [usize; 32],
    extra_caps: [usize; 32],
}
pub struct ReturnDataHook<R: Send>(Option<CapsuleHandle<R>>);

impl<R: Send> ReturnDataHook<R> {
    fn new(hooked_data: CapsuleHandle<R>) -> Self {
        Self(Some(hooked_data))
    }
    fn block(mut self) -> R {
        executor::block_on(self.0.take().unwrap()).unwrap()
    }
}

impl<R: Send> Drop for ReturnDataHook<R> {
    fn drop(&mut self) {
        match self.0.take() {
            // `None` indicates that return data has been explicitly blocked before
            None => {}
            Some(handle) => {
                if handle.try_take_data().is_none() {
                    executor::block_on(handle).unwrap();
                }
            }
        }
    }
}

/// Endpoint for cross crate communication
///
/// # Example
/// ```
///  fn callback(_: Box<IPCBuffer>) -> R{...}
///  let ep = EndPoint::new(callback);
///
///  // blocking send
///  let buf = Box::new(IPCBuffer::default());
///  let result: R = ep.send(buf);
///  // non-blocking send
///  let result_fut = ep.nb_send(buf);
///  // some workload()
///  // either explicitly block on `result_fut` otherwise it will be automatically blocked on drop()
///  let result = result_fut.block();
/// ```
pub struct Endpoint<P, R> {
    callback: fn(P) -> R,
    ipc_buf: Option<Box<IPCBuffer>>,
}

impl<R> Endpoint<Box<IPCBuffer>, R>
where
    Self: IntoCapsule,
    <Self as IntoCapsule>::Output: Send + Sync,
{
    pub fn new(callback: fn(Box<IPCBuffer>) -> R) -> Self {
        Self {
            callback,
            ipc_buf: None,
        }
    }

    /// nb send with hooked returned type, return type will be forced to complete prior it is dropped
    pub fn nb_send(
        &self,
        buf_ptr: Box<IPCBuffer>,
    ) -> ReturnDataHook<<Self as IntoCapsule>::Output> {
        let capsule = Self {
            callback: self.callback,
            ipc_buf: Some(buf_ptr),
        };
        ReturnDataHook::new(IntoCapsule::add_to_job_queue(capsule))
    }

    /// blocking send, blocks until wrapped CapsuleNode gets executed
    pub fn send(&self, buf_ptr: Box<IPCBuffer>) -> <Self as IntoCapsule>::Output {
        let capsule = Self {
            callback: self.callback,
            ipc_buf: Some(buf_ptr),
        };
        match executor::block_on(IntoCapsule::add_to_job_queue(capsule)) {
            Ok(output) => output,
            Err(_) => {
                panic!("Failed send");
            }
        }
    }
}

impl<R> IntoCapsule for Endpoint<Box<IPCBuffer>, R>
where
    R: Send + Sync + 'static,
{
    type Output = R;
    async fn resolve(self) -> Self::Output {
        (self.callback)(self.ipc_buf.unwrap())
    }
}
