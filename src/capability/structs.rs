#[derive(Debug, Clone, Default)]
pub struct IPCBuffer {
    pub mrs: [usize; 16],
}

pub struct TCB {
    pub ipc_buf: Box<IPCBuffer>,
}

impl TCB {
    pub fn new() -> Self {
        Self {
            ipc_buf: Box::new(IPCBuffer::default()),
        }
    }
}
