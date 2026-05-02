use {
	crate::elf::{
		Context, Dynamic, Elf, ElfHeader, RelocationSection, SHT_REL, SHT_RELA,
		SHT_SYMTAB, SectionHeader, StringTable, SymbolTable,
		SymbolVersionSection, VersionDefinitionSection, VersionNeededSection,
		get_string_table,
		hash::{gnu_hash_len, hash_len},
		program_header::interpreter,
	},
	core::cmp,
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

		let ctx = &Context::default();
		let (
			section_header_string_table,
			symbol_table,
			string_table_for_symbol_table,
		) = header.section_tables(ctx, binary, &section_headers,)?;

		let mut is_position_independent_executable = false;
		let mut shared_object_name = None;
		let mut libraries = alloc::vec![];
		let mut runtime_search_path_deprecated = alloc::vec![];
		let mut runtime_search_path = alloc::vec![];
		let mut dynamic_symbol_table = SymbolTable::default();
		let mut dynamic_relocation_with_addend = RelocationSection::default();
		let mut dynamic_relocation = RelocationSection::default();
		let mut procedure_linkage_table_relocation =
			RelocationSection::default();
		let mut dynamic_string_table = StringTable::default();

		let dynamic_info = Dynamic::parse(binary, &program_headers,)?;
		if let Some(ref dynamic,) = dynamic_info {
			let dyn_info = &dynamic.info;
			is_position_independent_executable =
				dyn_info.extended_flags & Dynamic::DF_EXTEND_PIE != 0;
			dynamic_string_table = StringTable::parse(
				binary,
				dyn_info.string_table_address,
				dyn_info.string_table_size,
				0x0,
			)?;

			if dyn_info.shared_object_name_offset != 0 {
				shared_object_name = dynamic_string_table
					.get_at(dyn_info.shared_object_name_offset,);
			}
			if dyn_info.version_need_count > 0 {
				libraries = dynamic.get_libraries(&dynamic_string_table,);
			}

			for dynamic in &dynamic.dyns {
				if dynamic.tag == Dynamic::DT_RPATH {
					if let Some(path,) =
						dynamic_string_table.get_at(dynamic.val as usize,)
					{
						runtime_search_path_deprecated.push(path,);
					}
				} else if dynamic.tag == Dynamic::DT_RUNPATH
					&& let Some(path,) =
						dynamic_string_table.get_at(dynamic.val as usize,)
				{
					runtime_search_path.push(path,);
				}
			}

			dynamic_relocation_with_addend = RelocationSection::parse(
				binary,
				dyn_info.relocation_addend,
				dyn_info.relocation_addend_size,
				true,
				ctx,
			)?;
			dynamic_relocation = RelocationSection::parse(
				binary,
				dyn_info.relocation,
				dyn_info.relocation_size,
				false,
				ctx,
			)?;
			let is_relocation_addrend =
				dyn_info.plt_relocation_type == Dynamic::DT_RELA;
			procedure_linkage_table_relocation = RelocationSection::parse(
				binary,
				dyn_info.jmp_relocation_address,
				dyn_info.plt_relocation_size,
				is_relocation_addrend,
				ctx,
			)?;

			let mut symbols_count = if let Some(gnu_hash,) = dyn_info.gnu_hash {
				gnu_hash_len(binary, gnu_hash as usize, ctx,)?
			} else if let Some(hash,) = dyn_info.hash {
				hash_len(binary, hash as usize, header.machine, ctx,)?
			} else {
				0
			};

			let max_relocation_symbol = dynamic_relocation_with_addend
				.iter()
				.chain(dynamic_relocation.iter(),)
				.chain(procedure_linkage_table_relocation.iter(),)
				.fold(0, |count, relocation| {
					cmp::max(count, relocation.symbol_index,)
				},);
			if max_relocation_symbol != 0 {
				symbols_count =
					cmp::max(symbols_count, max_relocation_symbol + 1,);
			}
			dynamic_symbol_table = SymbolTable::parse(
				binary,
				dyn_info.symbol_table,
				symbols_count,
				ctx,
			)?;
		}

		let mut section_relocations = alloc::vec![];
		for (index, section,) in section_headers.iter().enumerate() {
			let is_relocation_addrend = section.ty == SHT_RELA;
			if is_relocation_addrend || section.ty == SHT_REL {
				section.check_size(binary.len(),)?;
				let section_header_relocation_section =
					RelocationSection::parse(
						binary,
						section.offset as usize,
						section.size as usize,
						is_relocation_addrend,
						ctx,
					)?;
				section_relocations
					.push((index, section_header_relocation_section,),);
			}
		}

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
