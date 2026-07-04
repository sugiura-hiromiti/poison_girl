use {
	crate::elf::{
		Elf, ElfHeader,
		dynamic::dynamic::Dynamic,
		elf_context::ElfContext,
		program_header::interpreter,
		section_header::section_relocations,
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

		let dynamic_info = Dynamic::parse(binary, &program_headers,)?;
		let dynamic_string_table = dynamic_info.dynamic_string_table(binary,)?;
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
		alloc::{vec, vec::Vec},
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	fn push_cstr(strings: &mut Vec<u8,>, value: &[u8],) -> usize
	{
		let offset = strings.len();
		strings.extend_from_slice(value,);
		strings.push(0,);
		offset
	}

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

	#[test]
	fn parses_dynamic_libraries_names_paths_and_pie_flag() -> PoisonGirlTestB
	{
		let load_vaddr = 0x400000u64;
		let dynamic_offset = 0x100usize;
		let dynstr_offset = 0x200usize;
		let mut dynstr = Vec::new();
		dynstr.push(0,);
		let needed_offset = push_cstr(&mut dynstr, b"libdep.so",);
		let second_needed_offset = push_cstr(&mut dynstr, b"libextra.so",);
		let soname_offset = push_cstr(&mut dynstr, b"libself.so",);
		let rpath_offset = push_cstr(&mut dynstr, b"/old/lib",);
		let runpath_offset = push_cstr(&mut dynstr, b"/new/lib",);

		let mut dynamic = Vec::new();
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_STRTAB,
			load_vaddr + dynstr_offset as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_STRSZ,
			dynstr.len() as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_NEEDED,
			needed_offset as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_NEEDED,
			second_needed_offset as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_VERNEEDNUM,
			1,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_SONAME,
			soname_offset as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_RPATH,
			rpath_offset as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_RUNPATH,
			runpath_offset as u64,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_FLAGS_1,
			Dynamic::DF_EXTEND_PIE,
		),);
		dynamic.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_NULL,
			0,
		),);

		let mut binary = test_helpers::elf64_header(
			ElfType::SharedObject,
			load_vaddr,
			ELF_HEADER_SIZE as u64,
			2,
			ELF_HEADER_SIZE as u64,
			0,
			0,
		);
		binary.extend_from_slice(&test_helpers::program_header(
			ProgramHeaderType::Load,
			0,
			0,
			load_vaddr,
			load_vaddr,
			(dynstr_offset + dynstr.len()) as u64,
			(dynstr_offset + dynstr.len()) as u64,
			0x1000,
		),);
		binary.extend_from_slice(&test_helpers::program_header(
			ProgramHeaderType::Dynamic,
			0,
			dynamic_offset as u64,
			load_vaddr + dynamic_offset as u64,
			load_vaddr + dynamic_offset as u64,
			dynamic.len() as u64,
			dynamic.len() as u64,
			8,
		),);
		binary.resize(dynamic_offset, 0,);
		binary.extend_from_slice(&dynamic,);
		binary.resize(dynstr_offset, 0,);
		binary.extend_from_slice(&dynstr,);

		let elf = Elf::parse(&binary,)?;

		assert_eq!(
			elf.libraries.iter().map(|s| s.as_str(),).collect::<Vec<_,>>(),
			vec!["libdep.so", "libextra.so"]
		);
		assert_eq!(elf.shared_object_name.as_deref(), Some("libself.so"));
		assert_eq!(
			elf.runtime_search_path_deprecated
				.iter()
				.map(|s| s.as_str(),)
				.collect::<Vec<_,>>(),
			vec!["/old/lib"]
		);
		assert_eq!(
			elf.runtime_search_path
				.iter()
				.map(|s| s.as_str(),)
				.collect::<Vec<_,>>(),
			vec!["/new/lib"]
		);
		assert!(elf.is_position_independent_executable);
		success!()
	}
}
