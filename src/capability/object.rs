use super::structs::IPCBuffer;

#[derive(PartialEq, Copy, Clone, Eq, Debug)]
pub enum EPState {
    Idle = 0,
    Send = 1,
    Recv = 2,
}

#[derive(Copy, Clone)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone)]
pub struct UntypedObj {
    pub region: Region,
    used: Region,
    pub inited: bool,
}

pub enum Kobj {
    UntypedObj(UntypedObj),
    EndPointObj(EndPointObj<Box<IPCBuffer>, usize>),
}

impl UntypedObj {
    pub fn new(start: usize, end: usize) -> Self {
        UntypedObj {
            region: Region {
                start: start,
                end: end,
            },
            used: Region {
                start: 0x0,
                end: 0x0,
            },
            inited: false,
        }
    }

    pub fn get_region(&self) {
        println!("start {} end {}", self.region.start, self.region.end)
    }

    pub fn get_watermark(&self) {
        println!("start {} end {}", self.used.start, self.used.end)
    }
}

pub struct EndPointObj<P, R> {
    callback: fn(P) -> R,
    ipc_buf: Option<Box<IPCBuffer>>,
}

impl<R> EndPointObj<Box<IPCBuffer>, R> {
    pub fn new(callback: fn(Box<IPCBuffer>) -> R) -> Self {
        Self {
            callback,
            ipc_buf: None,
        }
    }
    pub fn dummy_send(&self) {
        println!("edp send");
    }

    // non-blocking send todo:implement real nb_send which returns a fut
    // pub fn nb_send(mut self, buf_ptr: Box<IPCBuffer>) {
    //     self.ipc_buf = Some(buf_ptr);
    //     IntoCapsule::add_to_job_queue(self);
    // }
    pub fn nb_send(&self, buf_ptr: Box<IPCBuffer>) {
        //let capsule = Self {
        //    callback: self.callback,
        //    ipc_buf: Some(buf_ptr),
        //};
        //// IntoCapsule::add_to_job_queue(capsule);
        //ReturnDataHook::new(IntoCapsule::add_to_job_queue(capsule))
    }

    /// blocking send
    /// in rCore,send will block this thread and schedule->nb_exec immdiately
    pub fn send(mut self, buf_ptr: Box<IPCBuffer>) {
        //self.ipc_buf = Some(buf_ptr);
        //match executor::block_on(IntoCapsule::add_to_job_queue(self)) {
        //    Ok(output) => output,
        //    Err(_) => {
        //        panic!("Failed send:");
        //    }
        //}
    }
}
