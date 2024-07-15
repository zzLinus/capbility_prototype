pub mod executor;
mod waker;
use crate::println;
use alloc::boxed::Box;

#[derive(Debug, Default, Clone)]
pub struct IPCBuffer {
    regs: [usize; 32],
    extra_caps: [usize; 32],
}
use executor::{CapsuleHandle, IntoCapsule};
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
            // if none, which means that return data has been blocked
            None => {}
            Some(handle) => {
                if handle.try_take_data().is_none() {
                    executor::block_on(handle).unwrap();
                }
            }
        }
    }
}

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

    // nb send with hooked returned type, return type will be forced to complete prior it is dropped
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

    /// blocking send
    /// in rCore,send will block this thread and schedule->nb_exec immdiately
    pub fn send(mut self, buf_ptr: Box<IPCBuffer>) -> <Self as IntoCapsule>::Output {
        self.ipc_buf = Some(buf_ptr);
        match executor::block_on(IntoCapsule::add_to_job_queue(self)) {
            Ok(output) => output,
            Err(_) => {
                panic!("Failed send:");
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
        //task::sleep(Duration::from_secs(1)).await;
        (self.callback)(self.ipc_buf.unwrap())
    }
}

fn callback1(_: Box<IPCBuffer>) -> usize {
    println!("callback with return gets called");
    // let dummy_buf = Box::new(IPCBuffer::default());
    // let ep = Endpoint::new(callback3);
    // ep.nb_send(dummy_buf);
    10usize
}

// fn callback3(_: Box<IPCBuffer>) {
//     println!("callback3 gets called")
// }

fn callback2(_: Box<IPCBuffer>) {
    // let dummy_buf = Box::new(IPCBuffer::default());
    // let ep = Endpoint::new(callback1);
    // ep.send(dummy_buf);
    println!("trivial callback gets called")
}

pub fn test_ep() {
    println!("starting test");
    let dummy_buf = Box::new(IPCBuffer::default());
    let ep1 = Endpoint::new(callback1);
    let ep2 = Endpoint::new(callback2);
    println!(
        "return value from callback1: {}",
        ep1.send(dummy_buf.clone())
    );
    let _ = ep2.nb_send(dummy_buf);
    //KERNEL_EXECUTOR.nb_exec();
}
