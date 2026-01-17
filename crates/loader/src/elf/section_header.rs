use super::StringTable;
use crate::elf::read_le_bytes;
use alloc::format;
use alloc::vec::Vec;
use oso_error::Rslt;
use oso_error::loader::EfiParseError;
use oso_error::loader::EfiParseStage;
use oso_error::oso_err;

/// Undefined section.
pub const SHN_UNDEF: u32 = 0;
/// Start of reserved indices.
pub const SHN_LORESERVE: u32 = 0xff00;
/// Start of processor-specific.
pub const SHN_LOPROC: u32 = 0xff00;
/// Order section before all others (Solaris).
pub const SHN_BEFORE: u32 = 0xff00;
/// Order section after all others (Solaris).
pub const SHN_AFTER: u32 = 0xff01;
/// End of processor-specific.
pub const SHN_HIPROC: u32 = 0xff1f;
/// Start of OS-specific.
pub const SHN_LOOS: u32 = 0xff20;
/// End of OS-specific.
pub const SHN_HIOS: u32 = 0xff3f;
/// Associated symbol is absolute.
pub const SHN_ABS: u32 = 0xfff1;
/// Associated symbol is common.
pub const SHN_COMMON: u32 = 0xfff2;
/// Index is in extra table.
pub const SHN_XINDEX: u32 = 0xffff;
/// End of reserved indices.
pub const SHN_HIRESERVE: u32 = 0xffff;

// === Legal values for sh_type (section type). ===
/// Section header table entry unused.
pub const SHT_NULL: u32 = 0;
/// Program data.
pub const SHT_PROGBITS: u32 = 1;
/// Symbol table.
pub const SHT_SYMTAB: u32 = 2;
/// String table.
pub const SHT_STRTAB: u32 = 3;
/// Relocation entries with addends.
pub const SHT_RELA: u32 = 4;
/// Symbol hash table.
pub const SHT_HASH: u32 = 5;
/// Dynamic linking information.
pub const SHT_DYNAMIC: u32 = 6;
/// Notes.
pub const SHT_NOTE: u32 = 7;
/// Program space with no data (bss).
pub const SHT_NOBITS: u32 = 8;
/// Relocation entries, no addends.
pub const SHT_REL: u32 = 9;
/// Reserved.
pub const SHT_SHLIB: u32 = 10;
/// Dynamic linker symbol table.
pub const SHT_DYNSYM: u32 = 11;
/// Array of constructors.
pub const SHT_INIT_ARRAY: u32 = 14;
/// Array of destructors.
pub const SHT_FINI_ARRAY: u32 = 15;
/// Array of pre-constructors.
pub const SHT_PREINIT_ARRAY: u32 = 16;
/// Section group.
pub const SHT_GROUP: u32 = 17;
/// Extended section indeces.
pub const SHT_SYMTAB_SHNDX: u32 = 18;
/// Number of defined types.
pub const SHT_NUM: u32 = 19;
/// Start OS-specific.
pub const SHT_LOOS: u32 = 0x6000_0000;
/// Object attributes.
pub const SHT_GNU_ATTRIBUTES: u32 = 0x6fff_fff5;
/// GNU-style hash table.
pub const SHT_GNU_HASH: u32 = 0x6fff_fff6;
/// Prelink library list.
pub const SHT_GNU_LIBLIST: u32 = 0x6fff_fff7;
/// Checksum for DSO content.
pub const SHT_CHECKSUM: u32 = 0x6fff_fff8;
/// Sun-specific low bound.
pub const SHT_LOSUNW: u32 = 0x6fff_fffa;
pub const SHT_SUNW_MOVE: u32 = 0x6fff_fffa;
pub const SHT_SUNW_COMDAT: u32 = 0x6fff_fffb;
pub const SHT_SUNW_SYMINFO: u32 = 0x6fff_fffc;
/// Version definition section.
pub const SHT_GNU_VERDEF: u32 = 0x6fff_fffd;
/// Version needs section.
pub const SHT_GNU_VERNEED: u32 = 0x6fff_fffe;
/// Version symbol table.
pub const SHT_GNU_VERSYM: u32 = 0x6fff_ffff;
/// Sun-specific high bound.
pub const SHT_HISUNW: u32 = 0x6fff_ffff;
/// End OS-specific type.
pub const SHT_HIOS: u32 = 0x6fff_ffff;
/// Start of processor-specific.
pub const SHT_LOPROC: u32 = 0x7000_0000;
/// X86-64 unwind information.
pub const SHT_X86_64_UNWIND: u32 = 0x7000_0001;
/// End of processor-specific.
pub const SHT_HIPROC: u32 = 0x7fff_ffff;
/// Start of application-specific.
pub const SHT_LOUSER: u32 = 0x8000_0000;
/// End of application-specific.
pub const SHT_HIUSER: u32 = 0x8fff_ffff;

pub struct SectionHeader {
	pub name:          u32,
	pub ty:            u32,
	pub flags:         u64,
	pub address:       u64,
	pub offset:        u64,
	pub size:          u64,
	pub link:          u32,
	pub info:          u32,
	pub section_align: u64,
	pub entry_size:    u64,
}

impl SectionHeader {
	const SIZE_64: usize = 64;

	pub fn parse(
		binary: &[u8],
		offset: &mut usize,
		count: usize,
	) -> Rslt<Vec<Self,>, EfiParseError,> {
		assert!(count <= binary.len() / Self::SIZE_64, "binary is too small");

		let mut section_headers = Vec::with_capacity(count,);
		// section_headers.push(Self::empty_section(offset,),);

		for _i in 0..count {
			let section_header = Self::parse_fields(binary, offset,)?;
			section_headers.push(section_header,);
		}

		Ok(section_headers,)
	}

	fn parse_fields(
		binary: &[u8],
		offset: &mut usize,
	) -> Rslt<Self, EfiParseError,> {
		macro_rules! fields {
			($field:ident) => {
				let Some($field,) = read_le_bytes(offset, binary,) else {
					return Err(oso_err!(EfiParseError::EndOfBinary {
						parser_pos: stringify!($field),
						stage: oso_error::loader::EfiParseStage::SectionHeader
					}),);
				};
			};
			($($fields:ident,)*) => {
				$(
					fields!($fields);
				)*
			};
		}

		fields!(
			name,
			ty,
			flags,
			address,
			segment_offset,
			size,
			link,
			info,
			section_align,
			entry_size,
		);

		let section_header = Self {
			name,
			ty,
			flags,
			address,
			offset: segment_offset,
			size,
			link,
			info,
			section_align,
			entry_size,
		};

		Ok(section_header,)
	}

	pub fn empty_section(_offset: &mut usize,) -> Self {
		todo!()
	}

	pub fn check_size(&self, size: usize,) -> Rslt<(), EfiParseError,> {
		if self.ty == SHT_NOBITS || self.size == 0 {
			return Ok((),);
		}

		let (end, overflow,) = self.offset.overflowing_add(self.size,);
		if overflow || end > size as u64 {
			return Err(oso_err!(EfiParseError::SizeOverflow {
				stage:    EfiParseStage::SectionHeader,
				name:     self.name as u64,
				expected: size as u64,
				base:     self.offset,
				size:     self.size,
			}),);
		}

		let (_, overflow,) = self.address.overflowing_add(self.size,);
		if overflow {
			return Err(oso_err!(EfiParseError::SizeOverflow {
				stage:    EfiParseStage::SectionHeader,
				name:     self.name as u64,
				expected: size as u64,
				base:     self.address,
				size:     self.size,
			}),);
		}

		Ok((),)
	}
}

impl core::fmt::Debug for SectionHeader {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_,>,) -> core::fmt::Result {
		f.debug_struct("SectionHeader",)
			.field("name", &format!("{:#x}", self.name),)
			.field("ty", &format!("{:#x}", self.ty),)
			.field("flags", &format!("{:#x}", self.flags),)
			.field("address", &format!("{:#x}", self.address),)
			.field("offset", &format!("{:#x}", self.offset),)
			.field("size", &format!("{:#x}", self.size),)
			.field("link", &format!("{:#x}", self.link),)
			.field("info", &format!("{:#x}", self.info),)
			.field("section_align", &format!("{:#x}", self.section_align),)
			.field("entry_size", &format!("{:#x}", self.entry_size),)
			.finish()
	}
}

pub fn get_string_table(
	section_headers: &[SectionHeader],
	mut idx: usize,
	binary: &[u8],
) -> Rslt<StringTable, EfiParseError,> {
	if idx == SHN_XINDEX as usize {
		if section_headers.is_empty() {
			return Ok(StringTable::default(),);
		}

		idx = section_headers[0].link as usize;
	}

	if idx >= section_headers.len() {
		Ok(StringTable::default(),)
	} else {
		let section_header = &section_headers[idx];
		section_header.check_size(binary.len(),)?;
		StringTable::parse(
			binary,
			section_header.offset as usize,
			section_header.size as usize,
			0x0,
		)
	}
}
