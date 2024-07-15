#[derive(Debug, Default, Clone)]
pub struct IPCBuffer {
    regs: [usize; 32],
    extra_caps: [usize; 32],
}

#[derive(PartialEq, Copy, Clone, Eq, Debug)]
pub enum EPState {
    Idle = 0,
    Send = 1,
    Recv = 2,
}

#[derive(Copy, Clone)]
pub struct Region {
    start: usize,
    end: usize,
}

#[derive(Copy, Clone)]
pub struct UntypedObj {
    region: Region,
    used: Region,
}

#[derive(Copy, Clone)]
pub struct EndPointObj {
    status: EPState,
}

pub enum Kobj {
    UntypedObj(UntypedObj),
    EndPointObj(EndPointObj),
}

impl Kobj{
    fn drop(&mut self) {

    }
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
        }
    }

    pub fn retype(&self) {
        println!("retype!");
    }

    pub fn get_region(&self) {
        println!("start {} end {}", self.region.start, self.region.end)
    }

    pub fn get_watermark(&self) {
        println!("start {} end {}", self.used.start, self.used.end)
    }
}

impl EndPointObj {
    pub fn new(s: EPState) -> Self {
        EndPointObj { status: s }
    }

    pub fn get_queue(&self) {
        println!("get qeueue")
    }
}
