pub const PT_SIZE_BITS: usize = 12;
pub const RISCV_NORMAL_PAGE: usize = 0;
pub const RISCV_MEGA_PAGE: usize = 1;
pub const RISCV_GIGA_PAGE: usize = 2;
pub const RISCV_TERA_PAGE: usize = 3;

pub const RISCV_PAGE_BITS: usize = 12;
pub const RISCV_MEGA_PAGE_BITS: usize = 21;
pub const RISCV_GIGA_PAGE_BITS: usize = 30;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const PA_WIDTH_SV39: usize = 56;
pub const VA_WIDTH_SV39: usize = 39;
pub const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
pub const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

pub const PT_INDEX_BITS: usize = 9;
pub const CONFIG_PT_LEVELS: usize = 3;
pub const SAFE_OS_PAGE_BITS: usize = 12;
pub const SAFE_OS_PAGE_TABLE_BITS: usize = 12;
pub const SAFE_OS_HUGE_PAGE_BITS: usize = 30;
pub const SAFE_OS_LARGE_PAGE_BITS: usize = 21;
