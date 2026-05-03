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
		// poison_girl_macro_def_test_elf_header_parse::test_elf_header_parse!(
		// 	header
		// );

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
