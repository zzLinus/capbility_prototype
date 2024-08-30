use crate::VirtAddr;
use alloc::string::String;
use alloc::string::ToString;

pub(super) fn get_rela_target_section_name(name: &str) -> String {
    let (rela_prefix, target_section) = name.split_at(5);
    if rela_prefix != ".rela" {
        panic!("Invalid rela section name");
    }
    target_section.to_string()
}

pub(super) fn get_rel_target_section_name(name: &str) -> String {
    let (rel_prefix, target_section) = name.split_at(4);
    if rel_prefix != ".rel" {
        panic!("Invalid rel section name");
    }
    target_section.to_string()
}

pub(super) fn get_dyn_target_section_name(name: &str) -> String {
    let (dyn_prefix, target_section) = name.split_at(4);
    if dyn_prefix != ".dyn" {
        panic!("Invalid dyn section name");
    }
    target_section.to_string()
}

pub(super) fn set_relocation_value<T>(vaddr: VirtAddr, data: T) {
    let ptr = vaddr.0 as *mut T;
    unsafe {
        ptr.write_unaligned(data);
    }
}

pub(super) fn get_relocation_value<T>(vaddr: VirtAddr) -> T {
    let v = vaddr.0 as *const T;
    let ptr = vaddr.0 as *const T;
    unsafe { ptr.read_unaligned() }
}
