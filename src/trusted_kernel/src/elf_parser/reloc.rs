use crate::define_structs;
use crate::define_uncopied_struct;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;

macro_rules! define_reloc_default_impl {
    ($name:ident ,{ $( $field_name:ident : $field_type:ty ),*  $(,)?})  => {
        impl <'a>$name<'a> {
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
            pub fn vec_parse(data: &'a [u8], count: usize) -> Option<Vec<Self>> {
                let mut elems = Vec::new();
                let type_size = Self::length();
                for i in 0..count {
                    elems
                        .push(Self::from_bytes(&data[i * type_size..(i + 1) * type_size])?);
                }
                Some(elems)
            }
        }
    };
}

macro_rules! define_tab {
    ($name:ident) => {
        paste::item! {
            pub struct [<$name Tab>]<'a> {
                pub(super) inner: Vec<[<$name>]<'a>>,
                pub(super) target_section : String,
            }
            impl <'a>[<$name Tab>]<'a> {
                pub fn target_section(&self) -> String {
                    self.target_section.clone()
                }
                pub fn inner(&self) -> &Vec<[<$name>]<'a>> {
                    &self.inner
                }
            }
        }
    };
}

define_uncopied_struct!(ElfRel32Uncopied);
define_uncopied_struct!(ElfRel64Uncopied);
define_uncopied_struct!(ElfRela32Uncopied);
define_uncopied_struct!(ElfRela64Uncopied);
define_uncopied_struct!(ElfDyn32Uncopied);
define_uncopied_struct!(ElfDyn64Uncopied);
define_tab!(ElfRel32Uncopied);
define_tab!(ElfRel64Uncopied);
define_tab!(ElfRela32Uncopied);
define_tab!(ElfRela64Uncopied);
define_tab!(ElfDyn32Uncopied);
define_tab!(ElfDyn64Uncopied);

define_reloc_default_impl!(ElfRel32Uncopied, {
    r_offset: u32,
    r_info: u32,
});

define_reloc_default_impl!(ElfRel64Uncopied, {
    r_offset: u64,
    r_info: u64,
});

define_reloc_default_impl!(ElfRela32Uncopied, {
    r_offset: u32,
    r_info: u32,
    r_addend: u32,
});

define_reloc_default_impl!(ElfRela64Uncopied, {
    r_offset: u64,
    r_info: u64,
    r_addend: u64,
});

define_reloc_default_impl!(ElfDyn32Uncopied, {
    d_tag: u32,
    d_val: u32,
});

define_reloc_default_impl!(ElfDyn64Uncopied, {
    d_tag: u64,
    d_val: u64,
});

impl<'a> ElfRel32Uncopied<'a> {
    pub fn r_sym(&self) -> u32 {
        self.r_info() >> 8
    }

    pub fn r_type(&self) -> u32 {
        self.r_info() & 0xff
    }
}

impl<'a> ElfRel64Uncopied<'a> {
    pub fn r_sym(&self) -> u32 {
        (self.r_info() >> 32) as u32
    }

    pub fn r_type(&self) -> u32 {
        self.r_info() as u32
    }
}

impl<'a> ElfRela32Uncopied<'a> {
    pub fn r_sym(&self) -> u32 {
        self.r_info() >> 8
    }

    pub fn r_type(&self) -> u32 {
        self.r_info() & 0xff
    }
}

impl<'a> ElfRela64Uncopied<'a> {
    pub fn r_sym(&self) -> u32 {
        (self.r_info() >> 32) as u32
    }

    pub fn r_type(&self) -> u32 {
        self.r_info() as u32
    }
}
