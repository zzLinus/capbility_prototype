#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmAttributes {
    pub words: [usize; 1],
}

impl VmAttributes {
    pub fn new(value: usize) -> Self {
        Self {
            words: [value & 0x1usize],
        }
    }

    pub fn from_word(w: usize) -> Self {
        Self { words: [w] }
    }

    pub fn get_execute_never(&self) -> usize {
        self.words[0] & 0x1usize
    }

    pub fn set_execute_never(&mut self, v64: usize) {
        self.words[0] &= !0x1usize;
        self.words[0] |= v64 & 0x1usize;
    }
}

pub const VM_KERNEL_ONLY: usize = 1;
pub const VM_READ_ONLY: usize = 2;
pub const VM_READ_WRITE: usize = 3;

pub fn riscvget_write_from_vmrights(vm_rights: usize) -> bool {
    vm_rights == VM_READ_WRITE
}

pub fn riscvget_read_from_vmrights(vm_rights: usize) -> bool {
    vm_rights != VM_READ_ONLY
}
