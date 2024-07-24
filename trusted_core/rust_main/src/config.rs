pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = PAGE_SIZE;
pub const USER_STACK_SIZE: usize = PAGE_SIZE;



// task configuration
pub const MAX_NUM_TASK: usize = 16;
pub const TASK_TEXT_LIMIT: usize = 0x20000;
pub const TASK_TEXT_BASE_ADDR: usize = 0x8040_0000;