pub mod executor;
use crate::capability::cap::{Cap, CapInvLable};
use crate::println;
use crate::scheduler::batch::BatchScheduler;
use crate::sync::Mutex;
use alloc::sync::Arc;
use core::default;
pub use executor::KERNEL_EXECUTOR;
use lazy_static::lazy_static;
mod waker;
use alloc::boxed::Box;
use executor::{CapsuleHandle, IntoCapsule};

lazy_static! {
    static ref FAKE_SCHED : Arc<Mutex<BatchScheduler>> = Arc::new(Mutex::new(BatchScheduler::new()));
    // TODO: lazy static init of KERNEL_STACK somehow fails
}

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
#[derive(Clone)]
pub struct Endpoint<P, R> {
    pub callback: fn(P) -> R,
    pub ipc_buf: Option<Box<IPCBuffer>>,
}

impl<P, R: default::Default> Default for Endpoint<P, R> {
    fn default() -> Self {
        Self {
            callback: |_| Default::default(),
            ipc_buf: None,
        }
    }
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

fn callback1(_: Box<IPCBuffer>) -> usize {
    println!("callback with return gets called");
    10usize
}

fn callback2(_: Box<IPCBuffer>) {
    println!("trivial callback gets called")
}

pub fn test_ep() {
    let mut s = FAKE_SCHED.lock();
    let c = s.current_id;
    let tcb = &mut s.tasks[c];
    let uc1 = Cap::get_root_untpye();

    // FIXME: Need to guarantee these 3 line block is atomic
    // TODO:  Need to init Kobj after retype
    // NOTE:  Can only retype root untype in to other kobj now
    //        Since it is the only kobj has actual value 😂

    tcb.mr.regs[0] = 1; // NOTE: Make a new PageObj
    Option::as_ref(&uc1.0)
        .unwrap()
        .lock()
        .decode_capinvok(CapInvLable::RETYPE, &tcb);
    //NOTE: get the last children which is the EndPoint just created
    let pc1 = Option::as_ref(&uc1.0).unwrap().lock().get_new_child();

    tcb.mr.regs[0] = 2; // NOTE: Make a new PageObj
    Option::as_ref(&uc1.0)
        .unwrap()
        .lock()
        .decode_capinvok(CapInvLable::RETYPE, &tcb);
    //NOTE: get the last children which is the EndPoint just created
    let ec2 = Option::as_ref(&uc1.0).unwrap().lock().get_new_child();

    Option::as_ref(&pc1.upgrade().unwrap())
        .unwrap()
        .lock()
        .decode_capinvok(CapInvLable::PG_CLR, &tcb);

    Option::as_ref(&ec2.upgrade().unwrap()) // using endpoint cap to invoke kobj funcition
        .unwrap()
        .lock()
        .decode_capinvok(CapInvLable::NB_SEND, &tcb);

    Option::as_ref(&uc1.0).unwrap().lock().revoke();

    println!("finish");

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
