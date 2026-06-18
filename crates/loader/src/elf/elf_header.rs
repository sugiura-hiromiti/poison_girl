use {
	crate::elf::{
		ELF_IDENT_SIZE, ProgramHeader, SectionHeader,
		elf_context::ElfContext,
		elf_header_ident::ElfHeaderIdent,
		elf_type::ElfType,
		read_le_bytes,
		section_header::{SHT_SYMTAB, get_string_table},
		string_table::StringTable,
		symbol_table::SymbolTable,
	},
	alloc::vec::Vec,
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, Y, poison_girl_err,
	},
};

pub mod elf_header_consts;

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfHeader
{
	pub ident: ElfHeaderIdent,
	pub ty: ElfType,
	pub machine: u16,
	pub version: u32,
	pub entry: u64,
	pub program_header_offset: u64,
	pub section_header_offset: u64,
	pub flags: u32,
	pub elf_header_size: u16,
	pub program_header_entry_size: u16,
	pub program_header_count: u16,
	pub section_header_entry_size: u16,
	pub section_header_count: u16,
	pub section_header_index_of_section_name_string_table: u16,
}

impl ElfHeader
{
	pub fn parse(binary: &[u8],) -> PoisonGirlB<Self,>
	{
		let Some(ident,) = binary.get(..ELF_IDENT_SIZE,) else {
			return Y(poison_girl_err!(ElfParseError::InvalidIdentLen(
				binary.len()
			)),);
		};
		let ident = ElfHeaderIdent::new(ident,)?;
		let remain = &binary[ELF_IDENT_SIZE..];
		header_flag_fields(ident, remain,)
	}

	pub fn program_headers(
		&self,
		binary: &[u8],
	) -> PoisonGirlB<Vec<ProgramHeader,>,>
	{
		let mut offset = self.program_header_offset as usize;
		let count = self.program_header_count as usize;
		ProgramHeader::parse(binary, &mut offset, count,)
	}

	pub fn section_headers(
		&self,
		binary: &[u8],
	) -> PoisonGirlB<Vec<SectionHeader,>,>
	{
		let mut offset = self.section_header_offset as usize;
		let count = self.section_header_count as usize;
		SectionHeader::parse(binary, &mut offset, count,)
	}

	/// # Return
	/// 返り値は(セクションヘッダストリングテーブル, シンボルテーブル,
	/// シンボルテーブルのストリングテーブル)
	pub(crate) fn section_tables(
		&self,
		ctx: &ElfContext,
		binary: &[u8],
		section_headers: &[SectionHeader],
	) -> PoisonGirlB<(StringTable, SymbolTable, StringTable,),>
	{
		let string_table_index =
			self.section_header_index_of_section_name_string_table as usize;
		let section_header_string_table =
			get_string_table(&section_headers, string_table_index, binary,)?;

		let mut symbol_table = SymbolTable::default();
		let mut string_table_for_symbol_table = StringTable::default();
		if let Some(section_header,) = section_headers
			.iter()
			.rfind(|section_header| section_header.ty == SHT_SYMTAB,)
		{
			let size = section_header.entry_size;
			let count = if size == 0 { 0 } else { section_header.size / size };
			symbol_table = SymbolTable::parse(
				binary,
				section_header.offset as usize,
				count as usize,
				ctx,
			)?;
			string_table_for_symbol_table = get_string_table(
				&section_headers,
				section_header.link as usize,
				binary,
			)?;
		}

		X((
			section_header_string_table,
			symbol_table,
			string_table_for_symbol_table,
		),)
	}

	pub(super) fn is_64(&self,) -> bool
	{
		self.ident.is_64()
	}

	pub(super) fn is_lib(&self,) -> bool
	{
		self.ty.is_lib()
	}

	pub(super) fn is_little_endian(&self,) -> bool
	{
		self.ident.is_little_endian()
	}
}

fn header_flag_fields(
	ident: ElfHeaderIdent,
	ident_remain: &[u8],
) -> PoisonGirlB<ElfHeader,>
{
	let offset = &mut 0;

	macro_rules! fields {
		($field:ident) => {
			let $field =
				read_le_bytes(offset, ident_remain,).ok_or_else(|| {
					let field = stringify!($field);
					poison_girl_err!(poison_girl_no_std_error::ElfParseError::EndOfBinary{
						parser_pos: field,
						stage: poison_girl_no_std_error::ElfParseStage::Header
					})
				})?;
		};
		($($fields:ident,)*)=>{
			$(
				fields!($fields);
			)*
		};
	}

	let ty: u16 = read_le_bytes(offset, ident_remain,).ok_or(
		poison_girl_err!(ElfParseError::EndOfBinary {
			parser_pos: "ty",
			stage:      ElfParseStage::Header,
		}),
	)?;
	let ty = ElfType::try_from(ty,)?;
	fields!(
		machine,
		version,
		entry,
		program_header_offset,
		section_header_offset,
		flags,
		elf_header_size,
		program_header_entry_size,
		program_header_count,
		section_header_entry_size,
		section_header_count,
		section_header_index_of_section_name_string_table,
	);

	X(ElfHeader {
		ident,
		ty,
		machine,
		version,
		entry,
		program_header_offset,
		section_header_offset,
		flags,
		elf_header_size,
		program_header_entry_size,
		program_header_count,
		section_header_entry_size,
		section_header_count,
		section_header_index_of_section_name_string_table,
	},)
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::{
			abi_version::AbiVersion, elf_version::ElfVersion, endian::Endian,
			file_class::FileClass, target_os_abi::TargetOsAbi, test_helpers,
		},
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	#[test]
	fn parses_64_bit_little_endian_header_from_bytes() -> PoisonGirlTestB
	{
		let binary = test_helpers::elf64_header(
			ElfType::Executable,
			0x100000,
			0x40,
			1,
			0x200,
			0,
			0,
		);

		let header = ElfHeader::parse(&binary,)?;

		assert_eq!(
			header,
			ElfHeader {
				ident: ElfHeaderIdent {
					file_class:    FileClass::Bit64,
					endianness:    Endian::Little,
					elf_version:   ElfVersion::ONE,
					target_os_abi: TargetOsAbi::SysV,
					abi_version:   AbiVersion(0,),
				},
				ty: ElfType::Executable,
				machine: 0x3e,
				version: 1,
				entry: 0x100000,
				program_header_offset: 0x40,
				section_header_offset: 0x200,
				flags: 0,
				elf_header_size: 0x40,
				program_header_entry_size: 0x38,
				program_header_count: 1,
				section_header_entry_size: 0x40,
				section_header_count: 0,
				section_header_index_of_section_name_string_table: 0,
			},
		);
		success!()
	}

	#[test]
	fn short_header_returns_error()
	{
		let binary = [0; ELF_IDENT_SIZE - 1];

		assert!(matches!(ElfHeader::parse(&binary,), Y(_)));
	}

	#[test]
	fn truncated_header_fields_return_error()
	{
		let mut binary = test_helpers::minimal_elf64();
		binary.truncate(ELF_IDENT_SIZE + 2,);

		assert!(matches!(ElfHeader::parse(&binary,), Y(_)));
	}

	#[test]
	fn zero_program_and_section_tables_parse() -> PoisonGirlTestB
	{
		let header = ElfHeader::parse(&test_helpers::minimal_elf64(),)?;

		assert_eq!(header.program_header_count, 0);
		assert_eq!(header.section_header_count, 0);
		assert!(header.program_headers(&[],)?.is_empty());
		assert!(header.section_headers(&[],)?.is_empty());
		success!()
	}
}
