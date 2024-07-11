pub mod executor;
mod waker;
use alloc::boxed::Box;
use crate::println;

#[derive(Debug, Default,Clone)]
pub struct IPCBuffer{
    regs: [usize; 32],
    extra_caps: [usize; 32]
}
use executor::IntoCapsule;


/*
    Endpoint logic
each endpoint is statically registered with a call back function when initialized
whenever a request coming in, the callback and its associated parameters, which is provided by
the sender are wrapped into a Capsule.

Capsules will be later scheduled by a kernel executor


let mm_ep = Endpoint::new();
let value = mm_ep.send();
*/



pub struct Endpoint<P, R> {
   callback: fn(P) -> R,
   ipc_buf: Option<Box<IPCBuffer>>
} 

impl<R> Endpoint<Box<IPCBuffer>, R>
where Self: IntoCapsule,
      <Self as IntoCapsule>::Output: Send + Sync
{
    pub fn new(callback: fn(Box<IPCBuffer>) -> R) -> Self{
        Self {callback, ipc_buf: None}
    }

    // non-blocking send todo:implement real nb_send which returns a fut
    pub fn nb_send(mut self, buf_ptr: Box<IPCBuffer>) {
        self.ipc_buf = Some(buf_ptr);
        IntoCapsule::add_to_job_queue(self);
    }

    /// blocking send 
    /// in rCore,send will block this thread and schedule->nb_exec immdiately
    pub fn send(mut self, buf_ptr: Box<IPCBuffer>) -> <Self as IntoCapsule>::Output{
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
where R: Send + Sync + 'static
{
    type Output = R;
    async fn resolve(self) -> Self::Output {
        //task::sleep(Duration::from_secs(1)).await;
        (self.callback)(self.ipc_buf.unwrap())
    }
}

fn callback1(_: Box<IPCBuffer>) -> usize{
    println!("callback with return gets called");
    10usize
}

fn callback2(_: Box<IPCBuffer>){
    println!("trivial callback gets called")
}



pub fn test_ep(){
    println!("starting test");
    let dummy_buf = Box::new(IPCBuffer::default());
    let ep1 = Endpoint::new(callback1);
    let ep2 = Endpoint::new(callback2);
    println!("return value from callback1: {}", ep1.send(dummy_buf.clone()));
    ep2.nb_send(dummy_buf);
    //KERNEL_EXECUTOR.nb_exec();
}