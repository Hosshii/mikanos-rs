use core::{mem, slice};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Elf<'a> {
    elf_header: &'a ElfHeader,
    program_header: &'a [Elf64ProgramHeader],
}

impl<'a> Elf<'a> {
    pub unsafe fn from_raw_parts(buf: *const u8, len: usize) -> Result<Elf<'a>> {
        let elf_header_size = mem::size_of::<ElfHeader>();
        let header = ElfHeader::from_raw_parts(buf)?;

        if len - elf_header_size < header.ph_num as usize * mem::size_of::<Elf64ProgramHeader>() {
            return Err(Error::ElfParse("too small buffer"));
        }
        let ptr = unsafe { buf.offset(header.ph_off as isize) as *const Elf64ProgramHeader };
        let program_header = unsafe { slice::from_raw_parts(ptr, header.ph_num as usize) };

        Ok(Elf {
            elf_header: header,
            program_header,
        })
    }

    pub fn calc_loader_addr_range(&self) -> (usize, usize) {
        let mut first = usize::max_value();
        let mut last = 0;
        for ph in self.program_header {
            if ph.type_ != 1 {
                continue;
            }
            first = first.min(ph.vaddr);
            last = last.max(ph.vaddr + ph.memsz as usize);
        }
        (first, last)
    }

    pub fn elf_header(&self) -> &ElfHeader {
        self.elf_header
    }

    pub fn program_header(&self) -> &[Elf64ProgramHeader] {
        self.program_header
    }
}

type ELF64Addr = usize;
type ELF64Off = u64;
type ELF64Half = u16;
type ELF64Word = u32;
type ELF64Sword = i32;
type ELF64Xword = u64;
type ELF64Sxword = i64;

const EI_NIDENT: usize = 16;
const ELF_MAGIC_SIGNATURE: &[u8; 4] = b"\x7fELF";

pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_SHLIB: u32 = 5;
pub const PT_PHDR: u32 = 6;
pub const PT_TLS: u32 = 7;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ElfHeader {
    ident: [u8; EI_NIDENT],
    type_: ELF64Half,
    machine: ELF64Half,
    version: ELF64Word,
    entry: ELF64Addr,
    ph_off: ELF64Off,
    sh_off: ELF64Off,
    flags: ELF64Word,
    eh_size: ELF64Half,
    ph_entsize: ELF64Half,
    ph_num: ELF64Half,
    sh_entsize: ELF64Half,
    sh_num: ELF64Half,
    sh_strndx: ELF64Half,
}

impl ElfHeader {
    pub fn new(buffer: &[u8]) -> Result<&ElfHeader> {
        if buffer.len() < mem::size_of::<ElfHeader>() {
            return Err(Error::ElfParse("too small buffer"));
        }
        unsafe { Self::from_raw_parts(buffer.as_ptr()) }
    }

    pub unsafe fn from_raw_parts<'a>(ptr: *const u8) -> Result<&'a ElfHeader> {
        let ptr = ptr as *const ElfHeader;
        let header = unsafe { &*ptr };
        if header.ident[0] != ELF_MAGIC_SIGNATURE[0]
            || header.ident[1] != ELF_MAGIC_SIGNATURE[1]
            || header.ident[2] != ELF_MAGIC_SIGNATURE[2]
            || header.ident[3] != ELF_MAGIC_SIGNATURE[3]
        {
            return Err(Error::ElfParse("invalid magic signature"));
        }

        Ok(header)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Elf64ProgramHeader {
    type_: ELF64Word,
    flags: ELF64Word,
    offset: ELF64Off,
    vaddr: ELF64Addr,
    paddr: ELF64Addr,
    filesz: ELF64Xword,
    memsz: ELF64Xword,
    align: ELF64Xword,
}

impl Elf64ProgramHeader {
    unsafe fn from_buf(buffer: &[u8]) -> &Elf64ProgramHeader {
        let ptr = buffer.as_ptr() as *const Elf64ProgramHeader;
        &*ptr
    }

    pub fn type_(&self) -> u32 {
        self.type_
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn file_size(&self) -> u64 {
        self.filesz
    }

    pub fn mem_size(&self) -> u64 {
        self.memsz
    }

    pub fn virtual_addr(&self) -> usize {
        self.vaddr
    }
}
