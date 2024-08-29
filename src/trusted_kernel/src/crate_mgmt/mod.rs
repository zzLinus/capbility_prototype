#![allow(unused)]
use crate::elf_parser::consts::{
    self, R_RISCV_32, R_RISCV_64, R_RISCV_ADD16, R_RISCV_ADD32, R_RISCV_ADD64, R_RISCV_ADD8,
    R_RISCV_CALL, R_RISCV_CALL_PLT, R_RISCV_GOT_HI20, R_RISCV_HI20, R_RISCV_LO12_I, R_RISCV_LO12_S,
    R_RISCV_PCREL_HI20, R_RISCV_PCREL_LO12_I, R_RISCV_PCREL_LO12_S, R_RISCV_SUB16, R_RISCV_SUB32,
    R_RISCV_SUB64, R_RISCV_SUB8, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE, STT_NOTYPE,
};
use crate::elf_parser::header::ElfSectionHeader64Uncopied;
use crate::elf_parser::reloc::{ElfRel64UncopiedTab, ElfRela64UncopiedTab};
use crate::elf_parser::symtab::ElfSym64Uncopied;
use crate::elf_parser::{is_elf, ElfFile64Uncopied, ElfUncopied};
use crate::kmem::{KMEM, PAGE_SIZE};
use crate::sync::Mutex;
use crate::{println, VirtAddr};
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::mem;
use log::{error, info, warn};
use spin::Once;
use util::{get_rela_target_section_name, get_relocation_value, set_relocation_value};

mod archive;
mod util;

static KERNEL_NAMESPACE: Once<Arc<Namespace>> = Once::new();
static KERNEL_CRATE: Once<Arc<LoadedCrate>> = Once::new();

pub struct Namespace {
    name: String,
    crates: Mutex<BTreeMap<String, Arc<LoadedCrate>>>,
    lost_interface: Mutex<BTreeMap<String, Vec<Arc<LoadedSymbolEntry>>>>,
}

pub struct LoadedCrate {
    name: String,
    rmeta: Option<usize>,
    objects: Mutex<BTreeMap<String, Arc<LoadedObject>>>,
    namespace: Arc<Namespace>,
    inner_lost_symtab: Mutex<BTreeMap<String, Arc<LoadedSymbolEntry>>>,
    export_symtab: Mutex<BTreeMap<String, Arc<LoadedSymbolEntry>>>,
    global_symtab: Mutex<BTreeMap<String, Arc<LoadedSymbolEntry>>>,
}

struct LoadedObject {
    name: String,
    sections: Mutex<BTreeMap<String, Arc<LoadedSection>>>,
    symtab: Mutex<BTreeMap<String, Arc<LoadedSymbolEntry>>>,
}

struct LoadedSegment {
    vaddr: VirtAddr,
    size: usize,
    permissions: u32,
    sections: Vec<(usize, Arc<LoadedSection>)>,
}

struct LoadedSection {
    name: String,
    vaddr: Mutex<VirtAddr>,
    size: usize,
}

struct LoadedSymbolEntry {
    name: String,
    offset: usize,
    visibility: bool,
    global: bool,
    export: bool,
    section: Mutex<Weak<LoadedSection>>,
}

struct RelocationEntry {
    r_type: usize,
    r_sym: usize,
    r_offset: usize,
    symbol_name: String,
    section_name: String,
}

impl LoadedSymbolEntry {
    fn get_vaddr(&self) -> Option<VirtAddr> {
        let section = self.section.lock().upgrade()?;
        let vaddr = section.vaddr.lock();
        Some(VirtAddr(vaddr.0 + self.offset))
    }
}

impl RelocationEntry {
    fn new(
        r_type: usize,
        r_sym: usize,
        r_offset: usize,
        symbol_name: String,
        section_name: String,
    ) -> Self {
        Self {
            r_type,
            r_sym,
            r_offset,
            symbol_name,
            section_name,
        }
    }
}

impl LoadedCrate {
    fn new(name: String, namespace: Arc<Namespace>) -> Arc<Self> {
        Arc::new(Self {
            name,
            rmeta: None,
            objects: Mutex::new(BTreeMap::new()),
            namespace,
            export_symtab: Mutex::new(BTreeMap::new()),
            global_symtab: Mutex::new(BTreeMap::new()),
            inner_lost_symtab: Mutex::new(BTreeMap::new()),
        })
    }

    fn add_inner_lost_interface(&self, symbol: Arc<LoadedSymbolEntry>) {
        self.inner_lost_symtab
            .lock()
            .insert(symbol.name.clone(), symbol);
    }

    fn add_lost_interface(
        &self,
        symbol: Arc<LoadedSymbolEntry>,
        crate_name: &str,
        function_name: &str,
    ) {
        let namespace = self.namespace.clone();
        let mut lost_interface = namespace.lost_interface.lock();
        let export_name = format!("{}___{}", crate_name, function_name);
        if !lost_interface.contains_key(&export_name) {
            lost_interface.insert(export_name.clone(), Vec::new());
        }
        let lost_interfaces = lost_interface.get_mut(&export_name).unwrap();
        lost_interfaces.push(symbol);
    }

    fn add_object(&self, object: Arc<LoadedObject>) {
        self.objects.lock().insert(object.name.clone(), object);
    }

    fn add_global_symbol(&self, symbol: Arc<LoadedSymbolEntry>) {
        self.global_symtab
            .lock()
            .insert(symbol.name.clone(), symbol);
    }

    fn get_global_symbol(&self, symbol_name: &str) -> Option<Arc<LoadedSymbolEntry>> {
        self.global_symtab.lock().get(symbol_name).cloned()
    }

    fn add_export_symbol(&self, symbol: Arc<LoadedSymbolEntry>) {
        self.export_symtab
            .lock()
            .insert(symbol.name.clone(), symbol);
    }

    fn get_export_func(&self, function_name: &str) -> Option<VirtAddr> {
        let locked_symtab = self.export_symtab.lock();
        let symbol = locked_symtab.get(function_name)?;
        let section = symbol.section.lock().upgrade()?;
        let vaddr = section.vaddr.lock();
        Some(VirtAddr(vaddr.0 + symbol.offset))
    }
}

impl LoadedObject {
    fn add_symbol(&self, symbol: Arc<LoadedSymbolEntry>) {
        self.symtab.lock().insert(symbol.name.clone(), symbol);
    }
    fn add_section(&self, section: Arc<LoadedSection>) {
        self.sections.lock().insert(section.name.clone(), section);
    }
    fn get_section(&self, section_name: &str) -> Option<Arc<LoadedSection>> {
        self.sections.lock().get(section_name).cloned()
    }
    fn get_symbol(&self, symbol_name: &str) -> Option<Arc<LoadedSymbolEntry>> {
        self.symtab.lock().get(symbol_name).cloned()
    }
    fn new(name: &str) -> Arc<Self> {
        Arc::new(Self {
            name: name.to_string(),
            sections: Mutex::new(BTreeMap::new()),
            symtab: Mutex::new(BTreeMap::new()),
        })
    }
}

impl LoadedSegment {
    fn new(permissions: u32) -> Box<Self> {
        Box::new(Self {
            vaddr: VirtAddr(0),
            size: 0,
            permissions,
            sections: Vec::new(),
        })
    }
    fn add_section(&mut self, offset: usize, section: Arc<LoadedSection>) {
        let mut align = mem::size_of::<usize>();
        if self.size % align != 0 {
            self.size += align - (self.size % align);
        }
        self.size += section.size;
        self.sections.push((offset, section));
    }
    fn set_section_vaddr(&self) {
        let mut vaddr = self.vaddr.0;
        let mut align = mem::size_of::<usize>();
        for (_, section) in self.sections.iter() {
            section.set_vaddr(VirtAddr(vaddr));
            vaddr += section.size;
            if vaddr % align != 0 {
                vaddr += align - (vaddr % align);
            }
        }
    }
    fn copy_to_memory(&self, elf_data: &[u8]) -> Result<(), &'static str> {
        for (offset, section) in self.sections.iter() {
            section.copy_to_memory(&elf_data[*offset..*offset + section.size])?;
        }
        Ok(())
    }
    fn alloc_vaddr(&mut self) -> Result<(), &'static str> {
        if self.size == 0 {
            return Ok(());
        }
        let vaddr = match KMEM.lock().palloc((self.size + PAGE_SIZE - 1) / PAGE_SIZE) {
            Some(vaddr) => vaddr,
            None => return Err("failed to allocate virtual address."),
        };
        self.vaddr = VirtAddr(vaddr);
        Ok(())
    }
    fn map_pages(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

impl Namespace {
    fn add_crate(&self, loaded_crate: Arc<LoadedCrate>) {
        self.crates
            .lock()
            .insert(loaded_crate.name.clone(), loaded_crate);
    }

    fn remove_crate(&self, crate_name: &str) {
        self.crates.lock().remove(crate_name);
    }

    /// get the **exported** function virtual address by crate name and function
    /// name in the namespace. if the function is not found, return None.
    pub fn get_func(&self, crate_name: &str, function_name: &str) -> Option<VirtAddr> {
        self.crates
            .lock()
            .get(crate_name)
            .and_then(|crate_| crate_.get_export_func(function_name))
    }
}

impl LoadedSection {
    fn new(name: String, vaddr: VirtAddr, size: usize) -> Arc<Self> {
        Arc::new(Self {
            name,
            vaddr: Mutex::new(vaddr),
            size,
        })
    }
    fn set_vaddr(&self, vaddr: VirtAddr) {
        warn!("section {} vaddr is {:x}", self.name, vaddr.0);
        *self.vaddr.lock() = vaddr;
    }
    fn copy_to_memory(&self, section_data: &[u8]) -> Result<(), &'static str> {
        if section_data.len() < self.size {
            return Err("section data is not enough.");
        }
        let mut vaddr = self.vaddr.lock().0;
        unsafe {
            core::ptr::copy(section_data.as_ptr(), vaddr as *mut u8, self.size);
        }
        Ok(())
    }
}

pub fn load_kernel(kernel_data: &[u8], kernel_name: &str) -> Result<(), &'static str> {
    match ElfUncopied::from_bytes(kernel_data) {
        Some(ElfUncopied::Elf64(elf64)) => {
            let kenrel_namespace = KERNEL_NAMESPACE.get().unwrap().clone();
            let kernel_crate = LoadedCrate::new(kernel_name.to_string(), kenrel_namespace);
            let kernel_object = LoadedObject::new("kernel_object.o/");
            let (section_headers, section_names) = elf64.section_headers();
            let elf_symtab = elf64.symtab();
            let (mut loading_symbols, mut sym_idx) = (Vec::new(), 0);
            for sym in elf_symtab.iter() {
                let symbol_name = elf64.symbol_name(sym.st_name() as usize).unwrap();
                if symbol_name == "" {
                    continue;
                }
                loading_symbols.push((sym.st_shndx(), sym));
            }
            loading_symbols.sort_unstable_by_key(|(shndx, _)| *shndx);
            for (shrdx, section_header) in section_headers.iter().enumerate() {
                if section_names[shrdx] == ""
                    || section_header.sh_size() == 0
                    || section_header.sh_flags() & SHF_ALLOC as u64 == 0
                {
                    continue;
                }
                let loaded_sections = LoadedSection::new(
                    section_names[shrdx].clone(),
                    VirtAddr(section_header.sh_addr() as usize),
                    section_header.sh_size() as usize,
                );
                while sym_idx < loading_symbols.len() && loading_symbols[sym_idx].0 == shrdx as u16
                {
                    let sym = loading_symbols[sym_idx].1;
                    let mut symbol_name = elf64.symbol_name(sym.st_name() as usize).unwrap();
                    let global = sym.st_global();
                    let loaded_symbol = Arc::new(LoadedSymbolEntry {
                        name: symbol_name.clone(),
                        offset: sym.st_value() as usize,
                        visibility: sym.st_visibility(),
                        global,
                        export: false,
                        section: Mutex::new(Arc::downgrade(&loaded_sections)),
                    });
                    if global {
                        kernel_crate.add_global_symbol(loaded_symbol.clone());
                        kernel_crate.add_export_symbol(loaded_symbol.clone());
                    }
                    kernel_object.add_symbol(loaded_symbol);
                    sym_idx += 1;
                }
            }
            Ok(())
        }
        Some(ElfUncopied::Elf32(_)) => return Err("currently only support 64-bit elf file."),
        _ => return Err("failed to parse elf file."),
    }
}

pub fn load_service(
    namespace: Arc<Namespace>,
    crate_name: &str,
    crate_data: &[u8],
) -> Result<(), &'static str> {
    warn!("loading crate {}...", crate_name);
    match archive::parse_archive(crate_data) {
        Some(archive_headers) => {
            let (mut archive_symtab, mut archive_lfntab) = (None, None);
            let loaded_crate = LoadedCrate::new(crate_name.to_string(), namespace.clone());
            namespace.add_crate(loaded_crate.clone());
            for archive_header in archive_headers.iter() {
                if archive_header.is_symtab() {
                    archive_symtab = Some(&crate_data[archive_header.data_offset()..]);
                } else if archive_header.is_lfntab() {
                    archive_lfntab = Some(&crate_data[archive_header.data_offset()..]);
                } else {
                    let object_name =
                        archive::get_file_name_from_archive(&archive_header, archive_lfntab)?;
                    if !object_name.ends_with(".o/")
                        || !is_elf(&crate_data[archive_header.data_offset()..])
                    {
                        continue;
                    }
                    load_object(
                        loaded_crate.clone(),
                        &object_name,
                        &crate_data[archive_header.data_offset()..],
                    )?;
                }
            }
            Ok(())
        }
        None => return Err("failed to parse archive file."),
    }
}

fn load_object(
    loaded_crate: Arc<LoadedCrate>,
    object_name: &str,
    object_data: &[u8],
) -> Result<(), &'static str> {
    match ElfUncopied::from_bytes(object_data) {
        Some(ElfUncopied::Elf64(elf64)) => {
            let loaded_object = Arc::new(LoadedObject {
                name: object_name.to_string(),
                sections: Mutex::new(BTreeMap::new()),
                symtab: Mutex::new(BTreeMap::new()),
            });
            let rela_tabs = elf64.rela_tab();
            let rel_tabs = elf64.reltab();
            let elf_symtab = elf64.symtab();
            let mut empty_name_symbol_count = 0;

            let mut extern_symtab = BTreeMap::new();
            let (section_headers, section_names) = elf64.section_headers();
            let mut loaded_sym_vec = Vec::new();
            let mut loading_symbols = BTreeMap::new();
            let loaded_relatabs = collect_relatab64(&elf64, &elf_symtab, rela_tabs);
            for sym in elf_symtab.iter() {
                let symbol_name = elf64.symbol_name(sym.st_name() as usize).unwrap();
                if symbol_name.is_empty() {
                    continue;
                }
                loading_symbols.insert(sym.st_shndx(), sym);
            }
            let mut r_segment = LoadedSegment::new(0);
            let mut rw_segment = LoadedSegment::new(0);
            let mut rx_segment = LoadedSegment::new(0);
            for (shrdx, section_header) in section_headers.iter().enumerate() {
                if section_names[shrdx] == ""
                    || section_header.sh_size() == 0
                    || section_header.sh_flags() & SHF_ALLOC as u64 == 0
                {
                    continue;
                }

                let loaded_section = LoadedSection::new(
                    section_names[shrdx].clone(),
                    VirtAddr(0),
                    section_header.sh_size() as usize,
                );
                if section_names[shrdx] == ""
                    || section_header.sh_size() == 0
                    || section_header.sh_flags() & SHF_ALLOC as u64 == 0
                {
                    continue;
                }

                let loaded_section = LoadedSection::new(
                    section_names[shrdx].clone(),
                    VirtAddr(0),
                    section_header.sh_size() as usize,
                );
                loaded_object.add_section(loaded_section.clone());
                if section_header.sh_flags() & SHF_ALLOC as u64 != 0 {
                    if section_header.sh_flags() & SHF_EXECINSTR as u64 != 0 {
                        rx_segment.add_section(
                            section_header.sh_offset() as usize,
                            loaded_section.clone(),
                        );
                    } else if section_header.sh_flags() & SHF_WRITE as u64 != 0 {
                        rw_segment.add_section(
                            section_header.sh_offset() as usize,
                            loaded_section.clone(),
                        );
                    } else {
                        r_segment.add_section(
                            section_header.sh_offset() as usize,
                            loaded_section.clone(),
                        );
                    }
                }

                let mut locked_inner_export_symtab = loaded_crate.inner_lost_symtab.lock();
                let mut filled_symbol_names = Vec::new();
                for (symbol_name, symbol) in locked_inner_export_symtab.iter() {
                    if *symbol_name == section_names[shrdx] {
                        let mut locked_weak_section = symbol.section.lock();
                        *locked_weak_section = Arc::downgrade(&loaded_section);
                        filled_symbol_names.push(symbol_name.clone());
                    }
                }
                for symbol_name in filled_symbol_names.iter() {
                    locked_inner_export_symtab.remove(symbol_name);
                }
                for (sym_idx, sym) in loading_symbols.iter() {
                    if *sym_idx == shrdx as u16 {
                        let mut symbol_name = elf64.symbol_name(sym.st_name() as usize).unwrap();
                        let global = sym.st_global();
                        let compiler_builtin =
                            sym.st_type() == STT_NOTYPE || sym.st_size() == 0 || symbol_name == "";
                        let export = section_names[shrdx] == ".export_code";
                        let extern_code = section_names[shrdx] == ".extern_code";
                        let loaded_symbol = Arc::new(LoadedSymbolEntry {
                            name: symbol_name.clone(),
                            offset: sym.st_value() as usize,
                            visibility: sym.st_visibility(),
                            global,
                            export,
                            section: Mutex::new(Arc::downgrade(&loaded_section)),
                        });
                        if global {
                            loaded_crate.add_global_symbol(loaded_symbol.clone());
                        }
                        if export {
                            loaded_crate.add_export_symbol(loaded_symbol.clone());
                        }
                        if extern_code {
                            extern_symtab.insert(loaded_symbol.name.clone(), loaded_symbol.clone());
                        }
                        loaded_object.add_symbol(loaded_symbol);
                        loaded_sym_vec.push(symbol_name);
                    }
                }
            }
            for (sym_idx, sym) in loading_symbols {
                if loaded_sym_vec.contains(&elf64.symbol_name(sym.st_name() as usize).unwrap()) {
                    continue;
                }
                let symbol_name = elf64.symbol_name(sym.st_name() as usize).unwrap();
                let global = sym.st_global();
                let compiler_builtin = sym.st_type() == STT_NOTYPE || sym.st_size() == 0;
                let export = false;
                let extern_code = false;
                let loaded_symbol = Arc::new(LoadedSymbolEntry {
                    name: symbol_name.clone(),
                    offset: sym.st_value() as usize,
                    visibility: sym.st_visibility(),
                    global,
                    export,
                    section: Mutex::new(Weak::new()),
                });
                loaded_crate.add_inner_lost_interface(loaded_symbol.clone());
                if global {
                    loaded_crate.add_global_symbol(loaded_symbol.clone());
                }
                if export {
                    loaded_crate.add_export_symbol(loaded_symbol.clone());
                }
                if extern_code {
                    extern_symtab.insert(loaded_symbol.name.clone(), loaded_symbol.clone());
                }
            }
            r_segment.alloc_vaddr()?;
            r_segment.set_section_vaddr();
            r_segment.copy_to_memory(object_data)?;
            rw_segment.alloc_vaddr()?;
            rw_segment.set_section_vaddr();
            rw_segment.copy_to_memory(object_data)?;
            rx_segment.alloc_vaddr()?;
            rx_segment.set_section_vaddr();
            rx_segment.copy_to_memory(object_data)?;

            do_relocation64(
                loaded_crate.clone(),
                loaded_object.clone(),
                None,
                loaded_relatabs,
                extern_symtab,
            )?;
            r_segment.map_pages()?;
            rw_segment.map_pages()?;
            rx_segment.map_pages()?;
            loaded_crate.add_object(loaded_object);
            Ok(())
        }
        Some(ElfUncopied::Elf32(_)) => Err("currently only support 64-bit elf file."),
        None => Err("failed to parse elf file."),
    }
}

fn collect_relatab64(
    elf64: &ElfFile64Uncopied,
    elf_symtab: &Vec<ElfSym64Uncopied>,
    rela_tabs: &Vec<ElfRela64UncopiedTab>,
) -> BTreeMap<String, Vec<RelocationEntry>> {
    let mut loaded_relabtabs = BTreeMap::new();
    for rela_tab in rela_tabs.iter() {
        let target_section_name = get_rela_target_section_name(&rela_tab.target_section());
        let mut loaded_relatab = Vec::new();
        for rela in rela_tab.inner().iter() {
            let symbol_name = elf64
                .symbol_name(elf_symtab[rela.r_sym() as usize].st_name() as usize)
                .unwrap();
            loaded_relatab.push(RelocationEntry::new(
                rela.r_type() as usize,
                rela.r_sym() as usize,
                rela.r_offset() as usize,
                symbol_name.clone(),
                target_section_name.clone(),
            ));
        }
        loaded_relabtabs.insert(target_section_name.clone(), loaded_relatab);
    }
    loaded_relabtabs
}

fn collect_reltab64(
    elf64: &ElfFile64Uncopied,
    elf_symtab: &Vec<ElfSym64Uncopied>,
    rel_tabs: &Vec<ElfRel64UncopiedTab>,
) -> BTreeMap<String, Vec<RelocationEntry>> {
    let mut loaded_relabtabs = BTreeMap::new();
    for rel_tab in rel_tabs.iter() {
        let target_section_name = get_rela_target_section_name(&rel_tab.target_section());
        let mut loaded_relatab = Vec::new();
        for rel in rel_tab.inner().iter() {
            let symbol_name = elf64
                .symbol_name(elf_symtab[rel.r_sym() as usize].st_name() as usize)
                .unwrap();
            loaded_relatab.push(RelocationEntry::new(
                rel.r_type() as usize,
                rel.r_sym() as usize,
                rel.r_offset() as usize,
                symbol_name,
                target_section_name.clone(),
            ));
        }
        loaded_relabtabs.insert(target_section_name.clone(), loaded_relatab);
    }
    loaded_relabtabs
}

fn do_relocation64(
    loaded_crate: Arc<LoadedCrate>,
    loaded_object: Arc<LoadedObject>,
    rel_tabs: Option<BTreeMap<String, Vec<RelocationEntry>>>,
    rela_tabs: BTreeMap<String, Vec<RelocationEntry>>,
    extern_symtab: BTreeMap<String, Arc<LoadedSymbolEntry>>,
) -> Result<(), &'static str> {
    for (section_name, relatab) in rela_tabs {
        let section = loaded_object.get_section(section_name.as_str());
        let (mut hi_vec, mut hi_idx) = (Vec::<usize>::new(), 0);
        let (mut hi_rel_vec, mut hi_rel_idx) = (Vec::<usize>::new(), 0);
        if section.is_none() {
            error!(
                "section {} not found in object {}",
                section_name, loaded_object.name
            );
            return Err("section not found.");
        } else {
            let section = section.unwrap();
            for rela in relatab {
                let symbol_name = rela.symbol_name.clone();
                let symbol_value = if loaded_object.get_symbol(&symbol_name).is_some() {
                    let symbol = loaded_object.get_symbol(&symbol_name).unwrap();
                    let locked_weak_section = symbol.section.lock();
                    match locked_weak_section.upgrade() {
                        Some(section) => section.vaddr.lock().0 + symbol.offset,
                        None => {
                            error!(
                                "loaded symbol not found : {},section is {}.\n",
                                symbol_name, section_name
                            );
                            continue;
                        }
                    }
                } else if loaded_crate.get_global_symbol(&symbol_name).is_some() {
                    let symbol = loaded_crate.get_global_symbol(&symbol_name).unwrap();
                    let locked_weak_section = symbol.section.lock();
                    match locked_weak_section.upgrade() {
                        Some(section) => section.vaddr.lock().0 + symbol.offset,
                        None => {
                            error!(
                                "loaded symbol not found : {},section is {}.\n",
                                symbol_name, section_name
                            );
                            continue;
                        }
                    }
                } else {
                    error!(
                        "loaded symbol not found : {},section is {}.\n",
                        symbol_name, section_name
                    );
                    continue;
                };
                let reloc_type = rela.r_type;
                let reloc_vaddr = VirtAddr(section.vaddr.lock().0 + rela.r_offset);
                warn!(
                    "section name is {},symbol name is {}",
                    section_name, symbol_name
                );

                do_relocation_write(
                    reloc_vaddr,
                    symbol_value,
                    reloc_type,
                    symbol_name.clone(),
                    &mut hi_vec,
                    &mut hi_rel_vec,
                    &mut hi_idx,
                    &mut hi_rel_idx,
                )?;
            }
        }
    }
    for (symbol_name, extern_symbol) in extern_symtab {
        fill_extern_vaddr(loaded_crate.clone(), symbol_name, &extern_symbol)?;
    }
    try_fill_lost_interface(loaded_crate.clone());
    Ok(())
}

fn try_fill_lost_interface(loaded_crate: Arc<LoadedCrate>) {
    let crate_name = loaded_crate.name.clone();
    let namespace = loaded_crate.namespace.clone();
    let mut locked_lost_interfaces = namespace.lost_interface.lock();
    let locked_export_symtab = loaded_crate.export_symtab.lock();
    for (name, sym) in locked_export_symtab.iter() {
        let export_symbol_name = format!("{}___{}", crate_name, name);

        if locked_lost_interfaces.contains_key(&export_symbol_name) {
            let lost_interfaces = locked_lost_interfaces.get(&export_symbol_name).unwrap();
            for lost_interface in lost_interfaces {
                let vaddr = lost_interface.get_vaddr().unwrap();
                let func_vaddr = sym.get_vaddr().unwrap();
                info!(
                    "filling lost interface {} in crate {},the func vaddr is {:x},the static var vaddr is {:x}...",
                    lost_interface.name, crate_name, func_vaddr.0,vaddr.0
                );
                unsafe {
                    (vaddr.0 as *mut usize).write(func_vaddr.0);
                }
            }
            locked_lost_interfaces.remove(&export_symbol_name);
        }
    }
}

fn fill_extern_vaddr(
    loaded_crate: Arc<LoadedCrate>,
    symbol_name: String,
    extern_symbol: &Arc<LoadedSymbolEntry>,
) -> Result<(), &'static str> {
    println!("filling extern symbol {}...", symbol_name);
    let position = match symbol_name.find("___") {
        Some(position) => position,
        None => {
            return Ok(());
        }
    };
    let (crate_name, func_name) = (&symbol_name[0..position], &symbol_name[position + 3..]);
    let vaddr = match extern_symbol.get_vaddr() {
        Some(vaddr) => vaddr,
        None => {
            return Err("extern symbol vaddr not found.");
        }
    };
    let func_vaddr = match loaded_crate.namespace.get_func(crate_name, func_name) {
        Some(func_vaddr) => func_vaddr,
        None => {
            loaded_crate.add_lost_interface(extern_symbol.clone(), crate_name, func_name);
            warn!("lost interface {} in crate {}", func_name, crate_name);
            return Ok(());
        }
    };
    println!(
        "filling extern symbol {}...,vaddr {:x},func_vaddr : {:x}",
        symbol_name, vaddr.0, func_vaddr.0
    );
    unsafe {
        (vaddr.0 as *mut usize).write(func_vaddr.0);
    }
    Ok(())
}

fn do_relocation_write(
    vaddr: VirtAddr,
    value: usize,
    reloc_type: usize,
    symbol_name: String,
    hi_vec: &mut Vec<usize>,
    hi_rel_vec: &mut Vec<usize>,
    hi_idx: &mut usize,
    hi_rel_idx: &mut usize,
) -> Result<(), &'static str> {
    warn!("relocation type is {}", reloc_type);
    match reloc_type as u32 {
        R_RISCV_HI20 => {
            let instruction = get_relocation_value::<u32>(vaddr);
            let hi20 = (value as u32 + 0x800) >> 12;
            let new_instruction = (instruction & 0x00000fff) | (hi20 << 12);
            set_relocation_value(vaddr, new_instruction);
            hi_vec.push(value);
        }
        R_RISCV_LO12_I => {
            if *hi_idx >= hi_vec.len() {
                return Err("hi index is out of range.");
            }
            let symbol_value = hi_vec[*hi_idx];
            let instruction = get_relocation_value::<u32>(vaddr);
            let lo12 = symbol_value as u32 & 0xfff;
            let new_instruction = (instruction & 0x000fffff) | (lo12 << 20);
            set_relocation_value(vaddr, new_instruction);
            *hi_idx += 1;
        }
        R_RISCV_LO12_S => {
            if *hi_idx >= hi_vec.len() {
                return Err("hi index is out of range.");
            }
            let symbol_value = hi_vec[*hi_idx];
            let instruction = get_relocation_value::<u32>(vaddr);
            let imm_11_5 = (symbol_value as u32 & 0xfe0) << 20;
            let imm_4_0 = (symbol_value as u32 & 0x1f) << 7;
            let new_instruction = (instruction & 0xfe00707f) | imm_11_5 | imm_4_0;
            set_relocation_value(vaddr, new_instruction);
            *hi_idx += 1;
        }
        R_RISCV_ADD8 => {
            set_relocation_value::<u8>(vaddr, get_relocation_value::<u8>(vaddr) + value as u8);
        }
        R_RISCV_ADD16 => {
            set_relocation_value::<u16>(vaddr, get_relocation_value::<u16>(vaddr) + value as u16);
        }
        R_RISCV_ADD32 => {
            set_relocation_value::<u32>(vaddr, get_relocation_value::<u32>(vaddr) + value as u32);
        }
        R_RISCV_ADD64 => {
            set_relocation_value::<u64>(vaddr, get_relocation_value::<u64>(vaddr) + value as u64);
        }
        R_RISCV_PCREL_HI20 => {
            let instruction = get_relocation_value::<u32>(vaddr);
            let offset = (value as u32).wrapping_sub(vaddr.0 as u32);
            println!(
                "value is {:x},vaddr is {:x},offset is {:x}",
                value, vaddr.0, offset
            );
            let hi20 = ((offset + 0x800) >> 12) & 0xfffff;
            let new_instruction = (instruction & 0xfff) | ((hi20 as u32) << 12);
            set_relocation_value(vaddr, new_instruction);
            hi_rel_vec.push(offset as usize);
        }
        R_RISCV_PCREL_LO12_I => {
            if *hi_rel_idx >= hi_rel_vec.len() {
                error!(
                    "hi index is out of range.,{},{}",
                    hi_rel_idx,
                    hi_rel_vec.len()
                );
                return Err("hi index is out of range.");
            }
            let symbol_value = hi_rel_vec[*hi_rel_idx];
            let instruction = get_relocation_value::<u32>(vaddr);
            let lo12 = symbol_value as u32 & 0xfff;
            let new_instruction = (instruction & 0x000fffff) | (lo12 << 20);
            set_relocation_value(vaddr, new_instruction);
            *hi_rel_idx += 1;
        }

        R_RISCV_PCREL_LO12_S => {
            if *hi_rel_idx >= hi_rel_vec.len() {
                return Err("hi index is out of range.");
            }
            let symbol_value = hi_rel_vec[*hi_rel_idx];
            let instruction = get_relocation_value::<u32>(vaddr);
            let imm_11_5 = (symbol_value as u32 & 0xfe0) << 20;
            let imm_4_0 = (symbol_value as u32 & 0x1f) << 7;
            let new_instruction = (instruction & 0xfe00707f) | imm_11_5 | imm_4_0;
            set_relocation_value(vaddr, new_instruction);
            *hi_rel_idx += 1;
        }

        R_RISCV_CALL => {
            let instruction_auipc = get_relocation_value::<u32>(vaddr);
            let offset = (value as i32).wrapping_sub(vaddr.0 as i32);
            let hi20 = (offset + 0x800) >> 12;
            let new_instruction_auipc = (instruction_auipc & 0x00000fff) | ((hi20 as u32) << 12);
            set_relocation_value(vaddr, new_instruction_auipc);
            let instruction_jalr = get_relocation_value::<u32>(VirtAddr(vaddr.0 + 4));
            let lo12 = offset & 0xfff;
            let new_instruction_jalr = (instruction_jalr & 0xfffff000) | (lo12 << 20) as u32;
            set_relocation_value(VirtAddr(vaddr.0 + 4), new_instruction_jalr);
        }

        R_RISCV_CALL_PLT => {
            let instruction_auipc = get_relocation_value::<u32>(vaddr);
            let offset = (value as i32).wrapping_sub(vaddr.0 as i32);
            let hi20 = (offset + 0x800) >> 12;
            let new_instruction_auipc = (instruction_auipc & 0x00000fff) | ((hi20 as u32) << 12);
            set_relocation_value(vaddr, new_instruction_auipc);

            let instruction_jalr = get_relocation_value::<u32>(VirtAddr(vaddr.0 + 4));
            let lo12 = (offset) & 0xfff;
            let new_instruction_jalr = (instruction_jalr & 0x000fffff) | (lo12 << 20) as u32;
            set_relocation_value(VirtAddr(vaddr.0 + 4), new_instruction_jalr);
        }

        R_RISCV_SUB8 => {
            set_relocation_value::<u8>(vaddr, get_relocation_value::<u8>(vaddr) - value as u8);
        }
        R_RISCV_SUB16 => {
            set_relocation_value::<u16>(vaddr, get_relocation_value::<u16>(vaddr) - value as u16);
        }
        R_RISCV_SUB32 => {
            set_relocation_value::<u32>(vaddr, get_relocation_value::<u32>(vaddr) - value as u32);
        }
        R_RISCV_SUB64 => {
            set_relocation_value::<u64>(vaddr, get_relocation_value::<u64>(vaddr) - value as u64);
        }
        R_RISCV_32 => {
            set_relocation_value(vaddr, value as u32);
        }
        R_RISCV_64 => {
            set_relocation_value(vaddr, value as u64);
        }
        R_RISCV_GOT_HI20 => {
            let instruction = get_relocation_value::<u32>(vaddr);
            let hi20 = (value as u32 + 0x800) >> 12;
            let new_instruction = (instruction & 0x00000fff) | (hi20 << 12);
            set_relocation_value(vaddr, new_instruction);
            hi_vec.push(value);
        }
        _ => {
            return Err("unsupported relocation type.");
        }
    }
    Ok(())
}

pub fn init() {
    KERNEL_NAMESPACE.call_once(|| {
        Arc::new(Namespace {
            name: "kernel".to_string(),
            crates: Mutex::new(BTreeMap::new()),
            lost_interface: Mutex::new(BTreeMap::new()),
        })
    });
}

pub fn get_kernel_namespace() -> Option<&'static Arc<Namespace>> {
    KERNEL_NAMESPACE.get()
}
