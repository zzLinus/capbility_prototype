pub(crate) const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

pub(crate) const REL_SECTION_NAME: &str = ".rel";
pub(crate) const RELA_SECTION_NAME: &str = ".rela";
pub(crate) const DYN_SECTION_NAME: &str = ".dyn";
pub(crate) const STRTAB_SECTION_NAME: &str = ".strtab";
pub(crate) const SYMTAB_SECTION_NAME: &str = ".symtab";
pub(crate) const SHSTRTAB_SECTION_NAME: &str = ".shstrtab";
pub(crate) const TEXT_SECTION_NAME: &str = ".text";
pub(crate) const DATA_SECTION_NAME: &str = ".data";
pub(crate) const BSS_SECTION_NAME: &str = ".bss";
pub(crate) const RODATA_SECTION_NAME: &str = ".rodata";
pub(crate) const COMMENT_SECTION_NAME: &str = ".comment";
pub(crate) const NOTE_SECTION_NAME: &str = ".note";
pub(crate) const RELRO_SECTION_NAME: &str = ".relro";
pub(crate) const EH_FRAME_SECTION_NAME: &str = ".eh_frame";

// RISC-V ELF Relocation Types
pub const R_RISCV_NONE: u32 = 0;
pub const R_RISCV_32: u32 = 1;
pub const R_RISCV_64: u32 = 2;
pub const R_RISCV_RELATIVE: u32 = 3;
pub const R_RISCV_COPY: u32 = 4;
pub const R_RISCV_JUMP_SLOT: u32 = 5;
pub const R_RISCV_TLS_DTPMOD32: u32 = 6;
pub const R_RISCV_TLS_DTPMOD64: u32 = 7;
pub const R_RISCV_TLS_DTPREL32: u32 = 8;
pub const R_RISCV_TLS_DTPREL64: u32 = 9;
pub const R_RISCV_TLS_TPREL32: u32 = 10;
pub const R_RISCV_TLS_TPREL64: u32 = 11;
pub const R_RISCV_BRANCH: u32 = 16;
pub const R_RISCV_JAL: u32 = 17;
pub const R_RISCV_CALL: u32 = 18;
pub const R_RISCV_CALL_PLT: u32 = 19;
pub const R_RISCV_GOT_HI20: u32 = 20;
pub const R_RISCV_TLS_GOT_HI20: u32 = 21;
pub const R_RISCV_TLS_GD_HI20: u32 = 22;
pub const R_RISCV_PCREL_HI20: u32 = 23;
pub const R_RISCV_PCREL_LO12_I: u32 = 24;
pub const R_RISCV_PCREL_LO12_S: u32 = 25;
pub const R_RISCV_HI20: u32 = 26;
pub const R_RISCV_LO12_I: u32 = 27;
pub const R_RISCV_LO12_S: u32 = 28;
pub const R_RISCV_TPREL_HI20: u32 = 29;
pub const R_RISCV_TPREL_LO12_I: u32 = 30;
pub const R_RISCV_TPREL_LO12_S: u32 = 31;
pub const R_RISCV_TPREL_ADD: u32 = 32;
pub const R_RISCV_ADD8: u32 = 33;
pub const R_RISCV_ADD16: u32 = 34;
pub const R_RISCV_ADD32: u32 = 35;
pub const R_RISCV_ADD64: u32 = 36;
pub const R_RISCV_SUB8: u32 = 37;
pub const R_RISCV_SUB16: u32 = 38;
pub const R_RISCV_SUB32: u32 = 39;
pub const R_RISCV_SUB64: u32 = 40;
pub const R_RISCV_GNU_VTINHERIT: u32 = 41;
pub const R_RISCV_GNU_VTENTRY: u32 = 42;
pub const R_RISCV_ALIGN: u32 = 43;
pub const R_RISCV_RVC_BRANCH: u32 = 44;
pub const R_RISCV_RVC_JUMP: u32 = 45;
pub const R_RISCV_RVC_LUI: u32 = 46;
pub const R_RISCV_GPREL_I: u32 = 47;
pub const R_RISCV_GPREL_S: u32 = 48;
pub const R_RISCV_TPREL_I: u32 = 49;
pub const R_RISCV_TPREL_S: u32 = 50;
pub const R_RISCV_RELAX: u32 = 51;
pub const R_RISCV_SUB6: u32 = 52;
pub const R_RISCV_SET6: u32 = 53;
pub const R_RISCV_SET8: u32 = 54;
pub const R_RISCV_SET16: u32 = 55;
pub const R_RISCV_SET32: u32 = 56;

//e_flags
pub(crate) const EF_RISCV_RVC: u32 = 1;
pub(crate) const EF_RISCV_FLOAT_ABI_SINGLE: u32 = 2;
pub(crate) const EF_RISCV_FLOAT_ABI_DOUBLE: u32 = 4;
pub(crate) const EF_RISCV_FLOAT_ABI_QUAD: u32 = 6;
pub(crate) const EF_RISCV_RVE: u32 = 8;
pub(crate) const EF_RISCV_TSO: u32 = 16;

//e_types
pub(crate) const ET_NONE: u32 = 0;
pub(crate) const ET_REL: u32 = 1;
pub(crate) const ET_EXEC: u32 = 2;
pub(crate) const ET_DYN: u32 = 3;
pub(crate) const ET_CORE: u32 = 4;

//p_type
pub(crate) const PT_NULL: u32 = 0;
pub(crate) const PT_LOAD: u32 = 1;
pub(crate) const PT_DYNAMIC: u32 = 2;
pub(crate) const PT_INTERP: u32 = 3;
pub(crate) const PT_NOTE: u32 = 4;
pub(crate) const PT_SHLIB: u32 = 5;
pub(crate) const PT_PHDR: u32 = 6;
pub(crate) const PT_LOPROC: u32 = 0x70000000;
pub(crate) const PT_HIPROC: u32 = 0x7fffffff;
pub(crate) const PT_GNU_STACK: u32 = 0x6474e551;

//p_flags
pub(crate) const PF_X: u32 = 0x1;
pub(crate) const PF_W: u32 = 0x2;
pub(crate) const PF_R: u32 = 0x4;

//sh_type
pub(crate) const SHT_NULL: u32 = 0; // Section header table entry unused
pub(crate) const SHT_PROGBITS: u32 = 1; // Program data
pub(crate) const SHT_SYMTAB: u32 = 2; // Symbol table
pub(crate) const SHT_STRTAB: u32 = 3; // String table
pub(crate) const SHT_RELA: u32 = 4; // Relocation entries with addends
pub(crate) const SHT_HASH: u32 = 5; // Symbol hash table
pub(crate) const SHT_DYNAMIC: u32 = 6; // Dynamic linking information
pub(crate) const SHT_NOTE: u32 = 7; // Notes
pub(crate) const SHT_NOBITS: u32 = 8; // Program space with no data (bss)
pub(crate) const SHT_REL: u32 = 9; // Relocation entries, no addends
pub(crate) const SHT_SHLIB: u32 = 10; // Reserved
pub(crate) const SHT_DYNSYM: u32 = 11; // Dynamic linker symbol table
pub(crate) const SHT_INIT_ARRAY: u32 = 14; // Array of pub(crate) const ructors
pub(crate) const SHT_FINI_ARRAY: u32 = 15; // Array of destructors
pub(crate) const SHT_PREINIT_ARRAY: u32 = 16; // Array of pre-pub(crate) const ructors
pub(crate) const SHT_GROUP: u32 = 17; // Section group
pub(crate) const SHT_SYMTAB_SHNDX: u32 = 18; // Extended section indices
pub(crate) const SHT_NUM: u32 = 19; // Number of defined types
pub(crate) const SHT_LOOS: u32 = 0x60000000; // Start OS-specific
pub(crate) const SHT_GNU_ATTRIBUTES: u32 = 0x6ffffff5; // Object attributes
pub(crate) const SHT_GNU_HASH: u32 = 0x6ffffff6; // GNU-style hash table
pub(crate) const SHT_GNU_LIBLIST: u32 = 0x6ffffff7; // Prelink library list
pub(crate) const SHT_CHECKSUM: u32 = 0x6ffffff8; // Checksum for DSO content
pub(crate) const SHT_LOSUNW: u32 = 0x6ffffffa; // Sun-specific low bound
pub(crate) const SHT_SUNW_MOVE: u32 = 0x6ffffffa;
pub(crate) const SHT_SUNW_COMDAT: u32 = 0x6ffffffb;
pub(crate) const SHT_SUNW_SYMINFO: u32 = 0x6ffffffc;
pub(crate) const SHT_GNU_VERDEF: u32 = 0x6ffffffd; // Version definition section
pub(crate) const SHT_GNU_VERNEED: u32 = 0x6ffffffe; // Version needs section
pub(crate) const SHT_GNU_VERSYM: u32 = 0x6fffffff; // Version symbol table
pub(crate) const SHT_HISUNW: u32 = 0x6fffffff; // Sun-specific high bound
pub(crate) const SHT_HIOS: u32 = 0x6fffffff; // End OS-specific type
pub(crate) const SHT_LOPROC: u32 = 0x70000000; // Start of processor-specific
pub(crate) const SHT_HIPROC: u32 = 0x7fffffff; // End of processor-specific
pub(crate) const SHT_LOUSER: u32 = 0x80000000; // Start of application-specific
pub(crate) const SHT_HIUSER: u32 = 0x8fffffff; // End of application-specific

//sh_flags
pub const SHF_WRITE: u32 = 0x1;
pub const SHF_ALLOC: u32 = 0x2;
pub const SHF_EXECINSTR: u32 = 0x4;
pub const SHF_MASKPROC: u32 = 0xf0000000;

//st_info

pub const STT_NOTYPE: u8 = 0;
pub const STT_OBJECT: u8 = 1;
pub(crate) const STT_FUNC: u8 = 2;
pub(crate) const STT_SECTION: u8 = 3;
pub(crate) const STT_FILE: u8 = 4;
pub(crate) const STT_LOPROC: u8 = 13;
pub(crate) const STT_HIPROC: u8 = 15;

pub(crate) const STB_LOCAL: u8 = 0;
pub(crate) const STB_GLOBAL: u8 = 1;
pub(crate) const STB_WEAK: u8 = 2;
pub(crate) const STB_LOPROC: u8 = 13;
pub(crate) const STB_HIPROC: u8 = 15;

pub(crate) const ELF_32: u8 = 1;
pub(crate) const ELF_64: u8 = 2;
pub(crate) const ELF_BIG_ENDIAN: u8 = 2;
pub(crate) const ELF_LITTLE_ENDIAN: u8 = 1;

pub(crate) const STV_DEFAULT: u8 = 0;
pub(crate) const STV_INTERNAL: u8 = 1;
pub(crate) const STV_HIDDEN: u8 = 2;
pub(crate) const STV_PROTECTED: u8 = 3;
