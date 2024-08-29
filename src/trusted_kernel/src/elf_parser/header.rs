use super::consts::{ELF_32, ELF_64, ELF_BIG_ENDIAN, ELF_LITTLE_ENDIAN};
use crate::define_structs;
use crate::define_uncopied_struct;
use alloc::vec::Vec;
use core::fmt::Display;
use core::mem::size_of;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ElfIdent {
    ei_magic: [u8; 4],
    ei_class: u8,
    ei_data: u8,
    ei_version: u8,
    ei_osabi: u8,
    ei_abiversion: u8,
    ei_pad: [u8; 7],
}

impl ElfIdent {
    pub fn is_elf32(&self) -> bool {
        self.ei_class == ELF_32
    }

    pub fn is_elf64(&self) -> bool {
        self.ei_class == ELF_64
    }

    pub fn is_big_endian(&self) -> bool {
        self.ei_data == ELF_BIG_ENDIAN
    }

    pub fn is_little_endian(&self) -> bool {
        self.ei_data == ELF_LITTLE_ENDIAN
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < size_of::<Self>() {
            return None;
        }
        Some(Self {
            ei_magic: data[0..4].try_into().unwrap(),
            ei_class: data[4],
            ei_data: data[5],
            ei_version: data[6],
            ei_osabi: data[7],
            ei_abiversion: data[8],
            ei_pad: data[9..16].try_into().unwrap(),
        })
    }
}

macro_rules! define_header_impl {
    ($name:ident ,{ $( $field_name:ident : $field_type:ty ),*  $(,)?}) => {
        impl<'a> $name<'a> {
            pub fn vec_parse(data: &'a [u8], count: usize) -> Option<Vec<Self>> {
                let mut section_headers = Vec::new();
                let type_size = Self::length();
                for i in 0..count {
                    section_headers
                        .push(Self::from_bytes(&data[i * type_size..(i + 1) * type_size])?);
                }
                Some(section_headers)
            }
            pub fn length() -> usize {
                let mut size = 0;
                $(
                    size += core::mem::size_of::<$field_type>();
                )*
                size
            }

            $(
                pub fn $field_name(&self) -> $field_type {
                    let start = $name::offset_of(stringify!($field_name));
                    let end = start + size_of::<$field_type>();
                    let raw_data = &self.raw_data[start..end];
                    unsafe { core::ptr::read_unaligned(raw_data.as_ptr() as *const $field_type) }
                }
            )*

            fn offset_of(field_name: &str) -> usize {
                let mut offset = 0;
                $(
                    if field_name == stringify!($field_name) {
                        return offset;
                    }
                    offset += core::mem::size_of::<$field_type>();
                )*
                panic!("Invalid field name");
            }

            pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
                if data.len() < Self::length() {
                    return None;
                }
                Some(Self {
                    raw_data: data,
                })
            }
        }
    };
}

macro_rules! define_program_header {
    ($name:ident,$type:ty) => {
        define_uncopied_struct!($name);
        define_header_impl!($name,{
            p_type: u32,
            p_flags: u32,
            p_offset: $type,
            p_vaddr: $type,
            p_paddr: $type,
            p_filesz: $type,
            p_memsz: $type,
            p_align: $type,
        });
    };
}

macro_rules! define_section_header {
    ($name:ident,$type:ty) => {
        define_uncopied_struct!($name);
        define_header_impl!($name,{
            sh_name: u32,
            sh_type: u32,
            sh_flags: $type,
            sh_addr: $type,
            sh_offset: $type,
            sh_size: $type,
            sh_link: u32,
            sh_info: u32,
            sh_addralign: $type,
            sh_entsize: $type,
        });
    };
}

macro_rules! define_elf_header {
    ($name:ident,$type:ty) => {
        define_uncopied_struct!($name);
        define_header_impl!($name,{
            e_type: u16,
            e_machine: u16,
            e_version: u32,
            e_entry: $type,
            e_phoff: $type,
            e_shoff: $type,
            e_flags: u32,
            e_ehsize: u16,
            e_phentsize: u16,
            e_phnum: u16,
            e_shentsize: u16,
            e_shnum: u16,
            e_shstrndx: u16,
        });
    };
}

define_elf_header!(ElfHeader32Uncopied, u32);
define_elf_header!(ElfHeader64Uncopied, u64);

define_section_header!(ElfSectionHeader32Uncopied, u32);
define_section_header!(ElfSectionHeader64Uncopied, u64);

define_program_header!(ElfProgramHeader32Uncopied, u32);
define_program_header!(ElfProgramHeader64Uncopied, u64);
