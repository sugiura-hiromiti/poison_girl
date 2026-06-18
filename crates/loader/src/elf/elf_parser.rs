use {
	crate::elf::{
		Elf, ElfHeader,
		dynamic::dynamic::Dynamic,
		elf_context::ElfContext,
		program_header::interpreter,
		section_header::section_relocations,
		string_table::StringTable,
		version_sections::{
			SymbolVersionSection, VersionDefinitionSection,
			VersionNeededSection,
		},
	},
	poison_girl_no_std_error::{PoisonGirlB, X},
};

impl Elf
{
	pub fn parse(binary: &[u8],) -> PoisonGirlB<Self,>
	{
		let header = ElfHeader::parse(binary,)?;

		let program_headers = header.program_headers(binary,)?;
		let interpreter = interpreter(&program_headers, binary,)?;
		let section_headers = header.section_headers(binary,)?;

		let ctx = &ElfContext::default();
		let (
			section_header_string_table,
			symbol_table,
			string_table_for_symbol_table,
		) = header.section_tables(ctx, binary, &section_headers,)?;

		let dynamic_string_table = StringTable::default();

		let dynamic_info = Dynamic::parse(binary, &program_headers,)?;
		let is_position_independent_executable =
			dynamic_info.is_position_independent_executable();
		let shared_object_name =
			dynamic_info.shared_object_name(&dynamic_string_table,);
		let libraries = dynamic_info.libraries(&dynamic_string_table,);
		let runtime_search_path_deprecated =
			dynamic_info.runtime_search_path_deprecated(&dynamic_string_table,);
		let runtime_search_path =
			dynamic_info.runtime_search_path(&dynamic_string_table,);
		let (
			dynamic_relocation_with_addend,
			dynamic_relocation,
			procedure_linkage_table_relocation,
			dynamic_symbol_table,
		) = dynamic_info.dynamic_relocations_and_symbol_table(
			ctx,
			binary,
			header.machine,
		)?;

		let section_relocations =
			section_relocations(&section_headers, binary, ctx,)?;

		let symbol_version_section =
			SymbolVersionSection::parse(binary, &section_headers, ctx,)?;
		let version_definition_section =
			VersionDefinitionSection::parse(binary, &section_headers, ctx,)?;
		let version_needed_section =
			VersionNeededSection::parse(binary, &section_headers, ctx,)?;

		X(Self {
			header,
			program_headers,
			section_headers,
			section_header_string_table,
			dynamic_string_table,
			dynamic_symbol_table,
			symbol_table,
			string_table_for_symbol_table,
			dynamic_info,
			dynamic_relocation_with_addend,
			dynamic_relocation,
			procedure_linkage_table_relocation,
			section_relocations,
			shared_object_name,
			interpreter,
			libraries,
			runtime_search_path_deprecated,
			runtime_search_path,
			symbol_version_section,
			version_definition_section,
			version_needed_section,
			is_position_independent_executable,
		},)
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::{
			elf_type::ElfType,
			program_header::ProgramHeaderType,
			test_helpers::{self, ELF_HEADER_SIZE},
		},
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	#[test]
	fn parses_minimal_elf64_without_tables() -> PoisonGirlTestB
	{
		let elf = Elf::parse(&test_helpers::minimal_elf64(),)?;

		assert!(elf.is_64());
		assert!(elf.is_little_endian());
		assert_eq!(elf.entry_point_address(), 0x100000);
		assert!(elf.program_headers.is_empty());
		assert!(elf.section_headers.is_empty());
		assert!(elf.interpreter.as_ref().is_none());
		assert!(elf.libraries.is_empty());
		assert!(elf.runtime_search_path.is_empty());
		assert!(elf.runtime_search_path_deprecated.is_empty());
		assert!(!elf.is_lib());
		success!()
	}

	#[test]
	fn extracts_interpreter_from_program_header() -> PoisonGirlTestB
	{
		let interp_offset = 0x80;
		let interp = b"/lib64/ld-linux-x86-64.so.2\0";
		let mut binary = test_helpers::elf64_header(
			ElfType::Executable,
			0x100000,
			ELF_HEADER_SIZE as u64,
			1,
			ELF_HEADER_SIZE as u64,
			0,
			0,
		);
		binary.extend_from_slice(&test_helpers::program_header(
			ProgramHeaderType::Interp,
			0,
			interp_offset,
			0,
			0,
			interp.len() as u64,
			interp.len() as u64,
			1,
		),);
		binary.resize(interp_offset as usize, 0,);
		binary.extend_from_slice(interp,);

		let elf = Elf::parse(&binary,)?;

		assert_eq!(
			elf.interpreter.as_ref().as_ref().map(|bytes| bytes.as_slice()),
			Some(&interp[..interp.len() - 1])
		);
		success!()
	}

	#[test]
	fn invalid_magic_returns_error()
	{
		let mut binary = test_helpers::minimal_elf64();
		binary[0] = 0;

		assert!(matches!(Elf::parse(&binary,), Y(_)));
	}

	#[test]
	fn program_header_beyond_binary_returns_error()
	{
		let binary = test_helpers::elf64_header(
			ElfType::Executable,
			0x100000,
			ELF_HEADER_SIZE as u64,
			1,
			ELF_HEADER_SIZE as u64,
			0,
			0,
		);

		assert!(matches!(Elf::parse(&binary,), Y(_)));
	}
}
