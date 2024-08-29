use super::{consts, STV_DEFAULT};
use crate::define_structs;
use alloc::vec::Vec;
use core::fmt::{self, Display};
use core::mem::size_of;

macro_rules! define_sym_impl {
    ($name:ident,{ $( $field_name:ident : $field_type:ty ),*  $(,)?}) => {
        impl<'a> $name<'a> {
            pub fn length() -> usize {
                let mut size = 0;
                $(
                    size += core::mem::size_of::<$field_type>();
                )*
                size
            }
            pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
                if data.len() < Self::length() {
                    return None;
                }
                Some(Self {
                    raw_data: data,
                })
            }

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

            $(
                pub fn $field_name(&self) -> $field_type {
                    let start = $name::offset_of(stringify!($field_name));
                    let end = start + size_of::<$field_type>();
                    let raw_data = &self.raw_data[start..end];
                    unsafe { core::ptr::read_unaligned(raw_data.as_ptr() as *const $field_type) }
                }
            )*
        }
    };
}

pub struct ElfSym32Uncopied<'a> {
    raw_data: &'a [u8],
}

pub struct ElfSym64Uncopied<'a> {
    raw_data: &'a [u8],
}

define_sym_impl!(ElfSym32Uncopied, {
    st_name: u32,
    st_value: u32,
    st_size: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
});

define_sym_impl!(ElfSym64Uncopied, {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    st_size: u64,
});

impl<'a> ElfSym32Uncopied<'a> {
    pub fn vec_parse(data: &'a [u8], section_size: usize) -> Option<Vec<Self>> {
        if data.len() < section_size {
            return None;
        }
        let mut syms = Vec::new();
        let entry_size = Self::length();
        for i in (0..section_size).step_by(entry_size) {
            if let Some(sym) = Self::from_bytes(&data[i..i + entry_size]) {
                syms.push(sym);
            }
        }
        syms.sort_unstable_by_key(|x| x.st_shndx());
        Some(syms)
    }
}

impl<'a> ElfSym64Uncopied<'a> {
    pub fn vec_parse(data: &'a [u8], section_size: usize) -> Option<Vec<Self>> {
        if data.len() < section_size {
            return None;
        }
        let mut syms = Vec::new();
        let entry_size = Self::length();
        for i in (0..section_size).step_by(entry_size) {
            if let Some(sym) = Self::from_bytes(&data[i..]) {
                syms.push(sym);
            }
        }
        Some(syms)
    }

    pub fn st_visibility(&self) -> bool {
        self.st_other() & STV_DEFAULT == STV_DEFAULT
    }

    pub fn st_global(&self) -> bool {
        self.st_info() >> 4 == consts::STB_GLOBAL
    }

    pub fn st_type(&self) -> u8 {
        self.st_info() & 0xf
    }
}
