use {
	crate::elf::{
		Interpreter, read_le_bytes_or, string_context::StringContext,
	},
	alloc::{format, vec::Vec},
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, PoisonGirlError, X, Y,
		poison_girl_err,
	},
};

#[derive(PartialEq, Eq,)]
pub struct ProgramHeader
{
	pub ty:               ProgramHeaderType,
	pub flags:            u32,
	pub offset:           u64,
	pub virtual_address:  u64,
	pub physical_address: u64,
	pub file_size:        u64,
	pub memory_size:      u64,
	pub align:            u64,
}

impl ProgramHeader
{
	/// size of program header in 64bit architecture
	const SIZE_64: usize = 56;

	pub fn parse(
		binary: &[u8],
		offset: &mut usize,
		count: usize,
	) -> PoisonGirlB<Vec<Self,>,>
	{
		if count == 0 {
			return X(Vec::new(),);
		}

		let table_size = match count.checked_mul(Self::SIZE_64,) {
			Some(table_size,) => table_size,
			None => {
				return Y(poison_girl_err!(ElfParseError::EndOfBinary {
					parser_pos: "program header table",
					stage:      ElfParseStage::ProgramHeader,
				}),);
			},
		};
		let Some(table_end,) = offset.checked_add(table_size,) else {
			return Y(poison_girl_err!(ElfParseError::EndOfBinary {
				parser_pos: "program header table",
				stage:      ElfParseStage::ProgramHeader,
			}),);
		};
		if table_end > binary.len() {
			return Y(poison_girl_err!(ElfParseError::EndOfBinary {
				parser_pos: "program header table",
				stage:      ElfParseStage::ProgramHeader,
			}),);
		}

		let mut program_headers = Vec::with_capacity(count,);
		for _ in 0..count {
			let ty: u32 = read_le_bytes_or(
				offset,
				binary,
				"program header type",
				ElfParseStage::ProgramHeader,
			)?;
			let flags = read_le_bytes_or(
				offset,
				binary,
				"program header flags",
				ElfParseStage::ProgramHeader,
			)?;
			let segment_offset = read_le_bytes_or(
				offset,
				binary,
				"program header segment offset",
				ElfParseStage::ProgramHeader,
			)?;
			let virtual_address = read_le_bytes_or(
				offset,
				binary,
				"program header virtual address",
				ElfParseStage::ProgramHeader,
			)?;
			let physical_address = read_le_bytes_or(
				offset,
				binary,
				"program header physical address",
				ElfParseStage::ProgramHeader,
			)?;
			let file_size = read_le_bytes_or(
				offset,
				binary,
				"program header file size",
				ElfParseStage::ProgramHeader,
			)?;
			let memory_size = read_le_bytes_or(
				offset,
				binary,
				"program header memory size",
				ElfParseStage::ProgramHeader,
			)?;
			let align = read_le_bytes_or(
				offset,
				binary,
				"program header alignment",
				ElfParseStage::ProgramHeader,
			)?;

			let ty = ProgramHeaderType::try_from(ty,)?;

			let program_header = Self {
				ty,
				flags,
				offset: segment_offset,
				virtual_address,
				physical_address,
				file_size,
				memory_size,
				align,
			};

			program_headers.push(program_header,);
		}

		X(program_headers,)
	}
}

impl core::fmt::Debug for ProgramHeader
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_,>,) -> core::fmt::Result
	{
		f.debug_struct("ProgramHeader",)
			.field("ty", &self.ty,)
			.field("flags", &format!("{:#x}", self.flags),)
			.field("offset", &format!("{:#x}", self.offset),)
			.field("virtual_address", &format!("{:#x}", self.virtual_address),)
			.field("physical_address", &format!("{:#x}", self.physical_address),)
			.field("file_size", &format!("{:#x}", self.file_size),)
			.field("memory_size", &format!("{:#x}", self.memory_size),)
			.field("align", &format!("{:#x}", self.align),)
			.finish()
	}
}

#[repr(u32)]
#[derive(PartialEq, Eq, Debug,)]
pub enum ProgramHeaderType
{
	/// ARM unwind segment
	ArmExidx    = 0x7000_0001,
	/// Dynamic linking information
	Dynamic     = 2,
	/// GCC .eh_frame_hdr segment
	GnuEhFrame  = 0x6474_e550,
	/// GNU property notes for linker and run-time loaders
	GnuProperty = 0x6474_e553,
	/// Read-only after relocation
	GnuRelro    = 0x6474_e552,
	/// Indicates stack executability
	GnuStack    = 0x6474_e551,
	/// End of OS-specific
	Hios        = 0x6fff_ffff,
	/// End of processor-specific
	Hiproc      = 0x7fff_ffff,
	/// Program interpreter
	Interp      = 3,
	/// Loadable program segment
	Load        = 1,
	/// Start of OS-specific
	Loos        = 0x6000_0000,
	/// Start of processor-specific
	Loproc      = 0x7000_0000,
	/// Sun Specific segment
	Losunw      = 0x6fff_fffa,
	/// Auxiliary information
	Note        = 4,
	/// Programg header table entry unused
	Null        = 0,
	/// Number of defined types
	Num         = 8,
	/// Entry for header table itself
	Phdr        = 6,
	/// Reserved
	Shlib       = 5,
	/// Stack segment
	Sunwstack   = 0x6fff_fffb,
	/// Thread-local storage segment
	Tls         = 7,
}

impl TryFrom<u32,> for ProgramHeaderType
{
	type Error = PoisonGirlError;

	fn try_from(value: u32,) -> Result<Self, Self::Error,>
	{
		let ty = match value {
			0x7000_0001 => Self::ArmExidx,
			2 => Self::Dynamic,
			0x6474_e550 => Self::GnuEhFrame,
			0x6474_e553 => Self::GnuProperty,
			0x6474_e552 => Self::GnuRelro,
			0x6474_e551 => Self::GnuStack,
			0x6fff_ffff => Self::Hios,
			0x7fff_ffff => Self::Hiproc,
			3 => Self::Interp,
			1 => Self::Load,
			0x6000_0000 => Self::Loos,
			0x7000_0000 => Self::Loproc,
			0x6fff_fffa => Self::Losunw,
			4 => Self::Note,
			0 => Self::Null,
			8 => Self::Num,
			6 => Self::Phdr,
			5 => Self::Shlib,
			0x6fff_fffb => Self::Sunwstack,
			7 => Self::Tls,
			_ => {
				return Err(poison_girl_err!(
					ElfParseError::InvalidProgramHeaderType(value,)
				),);
			},
		};
		Ok(ty,)
	}
}

/// elfにおけるinterpreterは通常動的linkerのことを指す
pub fn interpreter<'a,>(
	program_headers: &[ProgramHeader],
	binary: &'a [u8],
) -> PoisonGirlB<Interpreter,>
{
	let mut interpreter = None;
	for program_header in program_headers {
		if program_header.ty == ProgramHeaderType::Interp
			&& program_header.file_size != 0
		{
			let count = program_header.file_size as usize - 1;
			let offset = program_header.offset as usize;

			interpreter = Some(
				StringContext::Length(count,)
					.read_bytes(&binary[offset..],)?
					.to_vec(),
			);
		}
	}

	X(Interpreter(interpreter,),)
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::test_helpers,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	#[test]
	fn parses_64_bit_program_header_from_bytes() -> PoisonGirlTestB
	{
		let binary = test_helpers::program_header(
			ProgramHeaderType::Load,
			0b101,
			0x1000,
			0x2000,
			0x2000,
			0x300,
			0x400,
			0x1000,
		);

		let mut offset = 0;
		let program_headers = ProgramHeader::parse(&binary, &mut offset, 1,)?;

		assert_eq!(offset, ProgramHeader::SIZE_64);
		assert_eq!(
			program_headers.as_slice(),
			&[ProgramHeader {
				ty:               ProgramHeaderType::Load,
				flags:            0b101,
				offset:           0x1000,
				virtual_address:  0x2000,
				physical_address: 0x2000,
				file_size:        0x300,
				memory_size:      0x400,
				align:            0x1000,
			},][..],
		);
		success!()
	}

	#[test]
	fn zero_count_returns_empty_table() -> PoisonGirlTestB
	{
		let mut offset = 123;

		let program_headers = ProgramHeader::parse(&[], &mut offset, 0,)?;

		assert!(program_headers.is_empty());
		assert_eq!(offset, 123);
		success!()
	}

	#[test]
	fn invalid_program_header_type_returns_error()
	{
		let binary =
			test_helpers::program_header_raw(0xffff_fff0, 0, 0, 0, 0, 0, 0, 0,);
		let mut offset = 0;

		assert!(matches!(ProgramHeader::parse(&binary, &mut offset, 1,), Y(_)));
	}

	#[test]
	fn count_beyond_available_bytes_returns_error()
	{
		let binary = [0; ProgramHeader::SIZE_64 - 1];
		let mut offset = 0;

		assert!(matches!(ProgramHeader::parse(&binary, &mut offset, 1,), Y(_)));
	}

	#[test]
	fn offset_beyond_available_bytes_returns_error()
	{
		let binary = [0; ProgramHeader::SIZE_64];
		let mut offset = 1;

		assert!(matches!(ProgramHeader::parse(&binary, &mut offset, 1,), Y(_)));
	}
}
