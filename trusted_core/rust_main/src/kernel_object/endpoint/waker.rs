use super::executor::CapsuleNode;

use alloc::sync::Arc;
use alloc::task::Wake;

pub struct ChannelWaker(Arc<CapsuleNode>);
impl ChannelWaker {
    pub(crate) fn new(node: Arc<CapsuleNode>) -> Self {
        Self(node)
    }
}

impl Wake for ChannelWaker {
    fn wake(self: Arc<Self>) {
        self.0.tx.send(Arc::clone(&self.0))
    }
}
