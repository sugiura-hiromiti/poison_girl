use {
	super::{elf_type::ElfType, program_header::ProgramHeaderType},
	alloc::vec::Vec,
};

pub(crate) const ELF_HEADER_SIZE: usize = 64;
pub(crate) const PROGRAM_HEADER_SIZE: usize = 56;
pub(crate) const SECTION_HEADER_SIZE: usize = 64;

pub(crate) fn push_u16(binary: &mut Vec<u8,>, value: u16,)
{
	binary.extend_from_slice(&value.to_le_bytes(),);
}

pub(crate) fn push_u32(binary: &mut Vec<u8,>, value: u32,)
{
	binary.extend_from_slice(&value.to_le_bytes(),);
}

pub(crate) fn push_u64(binary: &mut Vec<u8,>, value: u64,)
{
	binary.extend_from_slice(&value.to_le_bytes(),);
}

pub(crate) fn elf64_header(
	ty: ElfType,
	entry: u64,
	program_header_offset: u64,
	program_header_count: u16,
	section_header_offset: u64,
	section_header_count: u16,
	section_header_string_table_index: u16,
) -> Vec<u8,>
{
	let mut binary = Vec::with_capacity(ELF_HEADER_SIZE,);
	binary.extend_from_slice(b"\x7fELF",);
	binary.extend_from_slice(&[
		2, // 64-bit
		1, // little endian
		1, // ELF version
		0, // SysV ABI
		0, // ABI version
		0, 0, 0, 0, 0, 0, 0,
	],);
	push_u16(&mut binary, elf_type_value(ty,),);
	push_u16(&mut binary, 0x3e,);
	push_u32(&mut binary, 1,);
	push_u64(&mut binary, entry,);
	push_u64(&mut binary, program_header_offset,);
	push_u64(&mut binary, section_header_offset,);
	push_u32(&mut binary, 0,);
	push_u16(&mut binary, ELF_HEADER_SIZE as u16,);
	push_u16(&mut binary, PROGRAM_HEADER_SIZE as u16,);
	push_u16(&mut binary, program_header_count,);
	push_u16(&mut binary, SECTION_HEADER_SIZE as u16,);
	push_u16(&mut binary, section_header_count,);
	push_u16(&mut binary, section_header_string_table_index,);
	debug_assert_eq!(binary.len(), ELF_HEADER_SIZE);
	binary
}

pub(crate) fn minimal_elf64() -> Vec<u8,>
{
	elf64_header(
		ElfType::Executable,
		0x100000,
		ELF_HEADER_SIZE as u64,
		0,
		ELF_HEADER_SIZE as u64,
		0,
		0,
	)
}

pub(crate) fn program_header(
	ty: ProgramHeaderType,
	flags: u32,
	offset: u64,
	virtual_address: u64,
	physical_address: u64,
	file_size: u64,
	memory_size: u64,
	align: u64,
) -> Vec<u8,>
{
	program_header_raw(
		program_header_type_value(ty,),
		flags,
		offset,
		virtual_address,
		physical_address,
		file_size,
		memory_size,
		align,
	)
}

pub(crate) fn program_header_raw(
	ty: u32,
	flags: u32,
	offset: u64,
	virtual_address: u64,
	physical_address: u64,
	file_size: u64,
	memory_size: u64,
	align: u64,
) -> Vec<u8,>
{
	let mut binary = Vec::with_capacity(PROGRAM_HEADER_SIZE,);
	push_u32(&mut binary, ty,);
	push_u32(&mut binary, flags,);
	push_u64(&mut binary, offset,);
	push_u64(&mut binary, virtual_address,);
	push_u64(&mut binary, physical_address,);
	push_u64(&mut binary, file_size,);
	push_u64(&mut binary, memory_size,);
	push_u64(&mut binary, align,);
	debug_assert_eq!(binary.len(), PROGRAM_HEADER_SIZE);
	binary
}

pub(crate) fn section_header(
	name: u32,
	ty: u32,
	flags: u64,
	address: u64,
	offset: u64,
	size: u64,
	link: u32,
	info: u32,
	section_align: u64,
	entry_size: u64,
) -> Vec<u8,>
{
	let mut binary = Vec::with_capacity(SECTION_HEADER_SIZE,);
	push_u32(&mut binary, name,);
	push_u32(&mut binary, ty,);
	push_u64(&mut binary, flags,);
	push_u64(&mut binary, address,);
	push_u64(&mut binary, offset,);
	push_u64(&mut binary, size,);
	push_u32(&mut binary, link,);
	push_u32(&mut binary, info,);
	push_u64(&mut binary, section_align,);
	push_u64(&mut binary, entry_size,);
	debug_assert_eq!(binary.len(), SECTION_HEADER_SIZE);
	binary
}

fn elf_type_value(ty: ElfType,) -> u16
{
	match ty {
		ElfType::None => 0,
		ElfType::Relocatable => 1,
		ElfType::Executable => 2,
		ElfType::SharedObject => 3,
		ElfType::Core => 4,
		ElfType::NumberOfDefined => 5,
		ElfType::OsSpecificRangeStart => 0xfe00,
		ElfType::OsSpecificRangeEnd => 0xfeff,
		ElfType::ProcessorSpecificRangeStart => 0xff00,
		ElfType::ProcessorSpecificRangeEnd => 0xffff,
	}
}

fn program_header_type_value(ty: ProgramHeaderType,) -> u32
{
	match ty {
		ProgramHeaderType::ArmExidx => 0x7000_0001,
		ProgramHeaderType::Dynamic => 2,
		ProgramHeaderType::GnuEhFrame => 0x6474_e550,
		ProgramHeaderType::GnuProperty => 0x6474_e553,
		ProgramHeaderType::GnuRelro => 0x6474_e552,
		ProgramHeaderType::GnuStack => 0x6474_e551,
		ProgramHeaderType::Hios => 0x6fff_ffff,
		ProgramHeaderType::Hiproc => 0x7fff_ffff,
		ProgramHeaderType::Interp => 3,
		ProgramHeaderType::Load => 1,
		ProgramHeaderType::Loos => 0x6000_0000,
		ProgramHeaderType::Loproc => 0x7000_0000,
		ProgramHeaderType::Losunw => 0x6fff_fffa,
		ProgramHeaderType::Note => 4,
		ProgramHeaderType::Null => 0,
		ProgramHeaderType::Num => 8,
		ProgramHeaderType::Phdr => 6,
		ProgramHeaderType::Shlib => 5,
		ProgramHeaderType::Sunwstack => 0x6fff_fffb,
		ProgramHeaderType::Tls => 7,
	}
}
