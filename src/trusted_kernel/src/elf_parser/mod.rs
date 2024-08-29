#![allow(clippy::bad_bit_mask)]
#![allow(unused)]
//! this module provides supports for both Elf32 and Elf64 structure.
//! it has relocation types,symbol types,section types and header
//! types which are all implemented with copied and uncopied version.
extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use consts::*;
use core::fmt::{self, Display, Formatter};
use core::mem::size_of;
use core::ptr::copy;
use header::*;
use reloc::*;
use symtab::*;

pub mod consts;
pub mod header;
pub mod reloc;
pub mod symtab;

type Shrdx = usize;

#[macro_export]
macro_rules! define_uncopied_struct {
    ($name:ident) => {
        pub struct $name<'a> {
            raw_data: &'a [u8],
        }
    };
}

#[macro_export]
macro_rules! define_structs {
    ($name:ident, { $( $field_name:ident : $field_type:ty ),*  $(,)? }) => {
        #[repr(C)]
        pub struct $name {
            $( $field_name: $field_type ),*
        }
        impl $name {
            $(
                pub fn $field_name(&self) -> $field_type {
                    self.$field_name
                }
            )*
            pub fn offset_of(field: &str) -> usize {
                let mut offset = 0;
                $(
                    if stringify!($field_name) == field {
                        return offset;
                    }
                    offset += size_of::<$field_type>();
                )*
                offset
            }
            pub const fn length() -> usize {
                core::mem::size_of::<$name>()
            }
            pub fn from_bytes(buffer: &[u8]) -> Option<Self> {
                if buffer.len() < core::mem::size_of::<$name>() {
                    return None;
                }

                let mut offset = 0;
                Some(Self {
                    $(
                        $field_name: {
                            let size = core::mem::size_of::<$field_type>();
                            let data = &buffer[offset..offset + size];
                            offset += size;
                            unsafe { core::ptr::read_unaligned(data.as_ptr() as *const $field_type) }
                        },
                    )*
                })
            }
        }

        paste::item! {
            pub struct [<$name Uncopied>]<'a> {
                raw_data: &'a [u8],
            }
            pub enum [<$name Enum>]<'a> {
                $name($name),
                [<$name Uncopied>]([<$name Uncopied>]<'a>),
            }
            impl <'a> [<$name Uncopied>]<'a> {
                pub fn from_bytes(buffer: &'a [u8]) -> Option<Self> {
                    if buffer.len() < Self::length() {
                        return None;
                    }
                    Some([<$name Uncopied>] { raw_data: buffer })
                }
                pub const fn length() -> usize {
                    core::mem::size_of::<$name>()
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
        }
    };
}

macro_rules! define_elf_file {
    ($name:ident,$hty:ty,$shty:ty,$phty:ty,$symty:ty,$relty:ty,$relaty:ty) => {
        paste::item! {
            pub struct $name<'a> {
                header: $hty<'a>,
                section_headers: Vec<$shty<'a>>,
                program_headers: Vec<$phty<'a>>,
                symtab: Vec<$symty<'a>>,
                reltab: Vec<[<$relty Tab>]<'a>>,
                rela_tab: Vec<[<$relaty Tab>]<'a>>,
                section_names: Vec<String>,
                strtab_off: usize,
                data: &'a [u8],
            }

            impl<'a> $name<'a> {
                pub fn strtab(&self) -> &'a [u8] {
                    &self.data[self.strtab_off..]
                }

                pub fn reltab(&self) -> &Vec<[<$relty Tab>]<'a>> {
                    &self.reltab
                }

                pub fn rela_tab(&self) -> &Vec<[<$relaty Tab>]<'a>> {
                    &self.rela_tab
                }

                pub fn copy_section(&self, sh: &$shty<'a>, vaddr: usize) -> Result<(), &'static str> {
                    let data = &self.data[sh.sh_offset() as usize..];
                    let size = sh.sh_size() as usize;
                    unsafe {
                        copy(data.as_ptr(), vaddr as *mut u8, size);
                    }
                    Ok(())
                }

                pub fn symbol_name(&self, st_name: usize) -> Option<String> {
                    let name = Self::section_name(&self.data[self.strtab_off..], st_name)?;
                    Some(name)
                }

                pub fn symtab(&self) -> &Vec<$symty<'a>> {
                    &self.symtab
                }

                pub fn section_name(data: &[u8], sh_name: usize) -> Option<String> {
                    let name = String::from_utf8(data[sh_name as usize..].split(|&x| x == 0).next()?.to_vec());
                    Some(name.expect("Invalid UTF-8 from ELF's section name."))
                }

                pub fn section_headers(&self) -> (Vec<&$shty<'a>>, Vec<String>) {
                    (
                        self.section_headers.iter().collect(),
                        self.section_names.clone(),
                    )
                }

                pub fn program_headers(&self) -> Vec<&$phty<'a>> {
                    self.program_headers.iter().collect()
                }

                pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
                    let header = <$hty>::from_bytes(data)?;
                    let section_headers = <$shty>::vec_parse(
                        &data[header.e_shoff() as usize..],
                        header.e_shnum() as usize,
                    )?;
                    let program_headers = <$phty>::vec_parse(
                        &data[header.e_phoff() as usize..],
                        header.e_phnum() as usize,
                    )?;
                    let mut section_names = Vec::<String>::new();
                    let mut strtab_off = 0;
                    let mut reltab = Vec::new();
                    let mut rela_tab = Vec::new();
                    let mut symtab = None;
                    let shstrtab_off =
                        section_headers[header.e_shstrndx() as usize].sh_offset() as usize;
                    section_headers.iter().for_each(|sh| {
                        let section_name =
                            Self::section_name(&data[shstrtab_off..], sh.sh_name() as usize)
                                .unwrap_or_else(|| "Unknown".to_string());
                        // currently not thinking of self-defined sections
                        if sh.sh_type() == consts::SHT_STRTAB {
                            strtab_off = sh.sh_offset() as usize;
                        } else if sh.sh_type() == consts::SHT_RELA {
                            let reltab_off = sh.sh_offset() as usize;
                            let reltab_size = sh.sh_size() as usize;
                            let _relatab = [<$relaty Tab>] {
                                inner: $relaty::vec_parse(&data[reltab_off..], reltab_size)
                                    .expect("Failed to parse rela table."),
                                target_section: section_name.clone(),
                            };
                            rela_tab.push(
                                _relatab
                            );
                        } else if sh.sh_type() == consts::SHT_REL {
                            let rel_tab_off = sh.sh_offset() as usize;
                            let rel_tab_size = sh.sh_size() as usize;
                            let _reltab = [<$relty Tab>] {
                                inner: $relty::vec_parse(&data[rel_tab_off..], rel_tab_size)
                                    .expect("Failed to parse rel table."),
                                target_section: section_name.clone(),
                            };
                            reltab.push(_reltab);
                        } else if sh.sh_type() == consts::SHT_SYMTAB {
                            let symtab_off = sh.sh_offset() as usize;
                            let symtab_size = sh.sh_size() as usize;
                            symtab = Some(
                                $symty::vec_parse(&data[symtab_off..], symtab_size)
                                    .expect("Failed to parse symtab."),
                            );
                        }
                        section_names.push(section_name);
                    });
                    Some(Self {
                        header,
                        section_headers,
                        program_headers,
                        data,
                        rela_tab,
                        reltab,
                        section_names,
                        symtab: symtab.unwrap_or_else(|| Vec::new()),
                        strtab_off,
                    })
                }
            }
        }
    }
}

define_elf_file!(
    ElfFile32Uncopied,
    ElfHeader32Uncopied,
    ElfSectionHeader32Uncopied,
    ElfProgramHeader32Uncopied,
    ElfSym32Uncopied,
    ElfRel32Uncopied,
    ElfRela32Uncopied
);

define_elf_file!(
    ElfFile64Uncopied,
    ElfHeader64Uncopied,
    ElfSectionHeader64Uncopied,
    ElfProgramHeader64Uncopied,
    ElfSym64Uncopied,
    ElfRel64Uncopied,
    ElfRela64Uncopied
);

pub enum ElfUncopied<'a> {
    Elf32(ElfFile32Uncopied<'a>),
    Elf64(ElfFile64Uncopied<'a>),
}

impl<'a> ElfUncopied<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if !is_elf(data) {
            return None;
        }
        if data[4] == ELF_32 {
            Some(ElfUncopied::Elf32(ElfFile32Uncopied::from_bytes(data)?))
        } else if data[4] == ELF_64 {
            Some(ElfUncopied::Elf64(ElfFile64Uncopied::from_bytes(data)?))
        } else {
            None
        }
    }
}

pub fn is_elf(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == ELF_MAGIC
}
