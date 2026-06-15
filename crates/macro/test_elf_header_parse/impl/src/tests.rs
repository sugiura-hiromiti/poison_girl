use {
	poison_girl_dev_test::{PoisonGirlTestB, success},
	poison_girl_macro_error::rslt::Rslt,
};

use super::*;

#[test]
fn test_readelf_h_default()
{
	let header = ReadElfH::default();

	// All fields should be empty strings by default
	assert_eq!(header.file_class, "");
	assert_eq!(header.endianness, "");
	assert_eq!(header.elf_version, "");
	assert_eq!(header.target_os_abi, "");
	assert_eq!(header.abi_version, "");
	assert_eq!(header.ty, "");
	assert_eq!(header.machine, "");
	assert_eq!(header.version, "");
	assert_eq!(header.entry, "");
	assert_eq!(header.program_header_offset, "");
	assert_eq!(header.section_header_offset, "");
	assert_eq!(header.flags, "");
	assert_eq!(header.elf_header_size, "");
	assert_eq!(header.program_header_entry_size, "");
	assert_eq!(header.program_header_count, "");
	assert_eq!(header.section_header_entry_size, "");
	assert_eq!(header.section_header_count, "");
	assert_eq!(header.section_header_index_of_section_name_string_table, "");
}

#[test]
fn test_readelf_h_fix_method()
{
	let mut header = ReadElfH {
		file_class: "ELF64 (64-bit)".to_string(),
		endianness: "little endian".to_string(),
		elf_version: "1 (current)".to_string(),
		target_os_abi: "UNIX - System V".to_string(), /* This one should
		                                               * not be split */
		abi_version: "0 (default)".to_string(),
		ty: "EXEC (Executable file)".to_string(),
		machine: "Advanced Micro Devices X86-64".to_string(),
		version: "0x1 (current)".to_string(),
		entry: "0x401000 (entry point)".to_string(),
		program_header_offset: "64 (bytes into file)".to_string(),
		section_header_offset: "1234 (bytes into file)".to_string(),
		flags: "0x0 (no flags)".to_string(),
		elf_header_size: "64 (bytes)".to_string(),
		program_header_entry_size: "56 (bytes)".to_string(),
		program_header_count: "2 (program headers)".to_string(),
		section_header_entry_size: "64 (bytes)".to_string(),
		section_header_count: "10 (section headers)".to_string(),
		section_header_index_of_section_name_string_table: "9 (string table \
		                                                    index)"
			.to_string(),
	};

	header.fix();

	// Check that only the first word is kept for most fields
	assert_eq!(header.file_class, "ELF64");
	assert_eq!(header.endianness, "little");
	assert_eq!(header.elf_version, "1");
	assert_eq!(header.target_os_abi, "UNIX - System V"); // Should remain unchanged
	assert_eq!(header.abi_version, "0");
	assert_eq!(header.ty, "EXEC");
	assert_eq!(header.machine, "Advanced");
	assert_eq!(header.version, "0x1");
	assert_eq!(header.entry, "0x401000");
	assert_eq!(header.program_header_offset, "64");
	assert_eq!(header.section_header_offset, "1234");
	assert_eq!(header.flags, "0x0");
	assert_eq!(header.elf_header_size, "64");
	assert_eq!(header.program_header_entry_size, "56");
	assert_eq!(header.program_header_count, "2");
	assert_eq!(header.section_header_entry_size, "64");
	assert_eq!(header.section_header_count, "10");
	assert_eq!(header.section_header_index_of_section_name_string_table, "9");
}

#[test]
fn test_property_trait_positive()
{
	let key_value = vec!["Class", "ELF64"];
	assert!(key_value.is_peoperty_of("Class"));
}

#[test]
fn test_property_trait_negative()
{
	let key_value = vec!["Class", "ELF64"];
	assert!(!key_value.is_peoperty_of("Data"));
}

#[test]
fn test_property_trait_single_element()
{
	let key_value = vec!["Class"];
	assert!(key_value.is_peoperty_of("Class"));
}

#[test]
fn test_property_trait_multiple_elements()
{
	let key_value =
		vec!["Entry point address", "0x401000", "additional", "info"];
	assert!(key_value.is_peoperty_of("Entry point address"));
	assert!(!key_value.is_peoperty_of("0x401000"));
}

#[test]
fn test_debug_trait_implementation()
{
	let header = ReadElfH::default();

	// Should be able to debug print the struct
	let debug_str = format!("{:?}", header);
	assert!(debug_str.contains("ReadElfH"));
}

#[test]
fn test_readelf_h_field_parsing_simulation() -> PoisonGirlTestB
{
	// Simulate parsing different types of readelf output lines
	let test_cases = vec![
		("Class:                             ELF64", "Class", "ELF64",),
		(
			"Data:                              2's complement, little endian",
			"Data",
			"2's",
		),
		("Version:                           1 (current)", "Version", "1",),
		(
			"OS/ABI:                            UNIX - System V",
			"OS/ABI",
			"UNIX - System V",
		),
		(
			"Type:                              EXEC (Executable file)",
			"Type",
			"EXEC",
		),
		(
			"Machine:                           Advanced Micro Devices X86-64",
			"Machine",
			"Advanced",
		),
		(
			"Entry point address:               0x401000",
			"Entry point address",
			"0x401000",
		),
	];

	for (line, expected_key, expected_first_word,) in test_cases {
		let key_value: Vec<_,> = line.split(':',).map(|s| s.trim(),).collect();

		if key_value.len() >= 2 {
			assert_eq!(key_value[0], expected_key);

			if expected_key != "OS/ABI" {
				// OS/ABI is special case
				let first_word = key_value[1].split(' ',).next()?;
				assert_eq!(first_word, expected_first_word);
			}
		}
	}

	success!()
}

#[test]
fn test_readelf_h_version_field_handling()
{
	// Test the special case where Version field can be either ELF version
	// or object version
	let elf_version_line = "Version:                           1 (current)";
	let object_version_line = "Version:                           0x1";

	let elf_key_value: Vec<_,> =
		elf_version_line.split(':',).map(|s| s.trim(),).collect();
	let obj_key_value: Vec<_,> =
		object_version_line.split(':',).map(|s| s.trim(),).collect();

	// ELF version doesn't contain 0x
	assert!(!elf_key_value[1].contains("0x"));

	// Object version contains 0x
	assert!(obj_key_value[1].contains("0x"));
}

#[test]
fn test_readelf_h_fix_method_edge_cases()
{
	let mut header = ReadElfH {
		file_class: "ELF32".to_string(), // No extra text
		endianness: "big".to_string(),   // Single word
		elf_version: "".to_string(),     // Empty string
		target_os_abi: "Multiple words here".to_string(),
		abi_version: "1".to_string(), // Already clean
		ty: "DYN".to_string(),
		machine: "ARM".to_string(),
		version: "0x2".to_string(),
		entry: "0x8000".to_string(),
		program_header_offset: "52".to_string(),
		section_header_offset: "0".to_string(),
		flags: "0x5000000".to_string(),
		elf_header_size: "52".to_string(),
		program_header_entry_size: "32".to_string(),
		program_header_count: "2".to_string(),
		section_header_entry_size: "40".to_string(),
		section_header_count: "0".to_string(),
		section_header_index_of_section_name_string_table: "0".to_string(),
	};

	header.fix();

	assert_eq!(header.file_class, "ELF32");
	assert_eq!(header.endianness, "big");
	assert_eq!(header.elf_version, ""); // Empty string should remain empty
	assert_eq!(header.target_os_abi, "Multiple words here"); // Not processed
	assert_eq!(header.abi_version, "1");
	assert_eq!(header.ty, "DYN");
	assert_eq!(header.machine, "ARM");
	assert_eq!(header.version, "0x2");
	assert_eq!(header.entry, "0x8000");
	assert_eq!(header.program_header_offset, "52");
	assert_eq!(header.section_header_offset, "0");
	assert_eq!(header.flags, "0x5000000");
	assert_eq!(header.elf_header_size, "52");
	assert_eq!(header.program_header_entry_size, "32");
	assert_eq!(header.program_header_count, "2");
	assert_eq!(header.section_header_entry_size, "40");
	assert_eq!(header.section_header_count, "0");
	assert_eq!(header.section_header_index_of_section_name_string_table, "0");
}

#[test]
fn test_property_trait_case_sensitivity()
{
	let key_value = vec!["Class", "ELF64"];

	// Should match exact case
	assert!(key_value.is_peoperty_of("Class"));

	// Should not match different case
	assert!(!key_value.is_peoperty_of("class"));
	assert!(!key_value.is_peoperty_of("CLASS"));
}

#[test]
fn test_property_trait_partial_matches()
{
	let key_value = vec!["Entry point address", "0x401000"];

	// Should match full string
	assert!(key_value.is_peoperty_of("Entry point address"));

	// Should not match partial strings
	assert!(!key_value.is_peoperty_of("Entry"));
	assert!(!key_value.is_peoperty_of("point"));
	assert!(!key_value.is_peoperty_of("address"));
}

#[test]
fn test_property_trait_empty_vector()
{
	let key_value: Vec<&str,> = vec![];

	// Should not match anything with empty vector
	assert!(!key_value.is_peoperty_of("Class"));
	assert!(!key_value.is_peoperty_of(""));
}

#[test]
fn test_property_trait_whitespace_handling()
{
	let key_value = vec!["  Class  ", "ELF64"];

	// Should not match due to whitespace differences
	assert!(!key_value.is_peoperty_of("Class"));

	// Should match with exact whitespace
	assert!(key_value.is_peoperty_of("  Class  "));
}

#[test]
fn test_readelf_h_with_whitespace_variations()
{
	let mut header = ReadElfH {
		file_class: "  ELF64   (64-bit)  ".to_string(),
		endianness: "\tlittle\tendian\t".to_string(),
		elf_version: " 1  (current) ".to_string(),
		target_os_abi: "UNIX - System V".to_string(),
		abi_version: "0".to_string(),
		ty: "EXEC".to_string(),
		machine: "x86-64".to_string(),
		version: "0x1".to_string(),
		entry: "0x401000".to_string(),
		program_header_offset: "64".to_string(),
		section_header_offset: "4096".to_string(),
		flags: "0x0".to_string(),
		elf_header_size: "64".to_string(),
		program_header_entry_size: "56".to_string(),
		program_header_count: "4".to_string(),
		section_header_entry_size: "64".to_string(),
		section_header_count: "10".to_string(),
		section_header_index_of_section_name_string_table: "9".to_string(),
	};

	header.fix();

	// The fix method should handle leading/trailing whitespace by taking
	// first word
	assert_eq!(header.file_class, "ELF64");
	assert_eq!(header.endianness, "little");
	assert_eq!(header.elf_version, "1");
}

#[test]
fn test_readelf_h_memory_efficiency()
{
	// Test that creating many ReadElfH instances doesn't cause issues
	let mut headers = Vec::new();

	for i in 0..1000 {
		let header = ReadElfH {
			file_class: format!("ELF{}", i % 2 + 32),
			endianness: if i % 2 == 0 {
				"little".to_string()
			} else {
				"big".to_string()
			},
			entry: format!("0x{:x}", i * 0x1000),
			..Default::default()
		};
		headers.push(header,);
	}

	assert_eq!(headers.len(), 1000);
	assert_eq!(headers[0].file_class, "ELF32");
	assert_eq!(headers[999].file_class, "ELF33");
}

#[test]
fn test_readelf_h_clone_behavior()
{
	let original = ReadElfH {
		file_class: "ELF64".to_string(),
		endianness: "little".to_string(),
		entry: "0x401000".to_string(),
		..Default::default()
	};

	// Test that we can create copies with the same data
	let copy = ReadElfH {
		file_class: original.file_class.clone(),
		endianness: original.endianness.clone(),
		entry: original.entry.clone(),
		..Default::default()
	};

	assert_eq!(original.file_class, copy.file_class);
	assert_eq!(original.endianness, copy.endianness);
	assert_eq!(original.entry, copy.entry);
}

#[test]
fn test_readelf_h_field_independence()
{
	let mut header = ReadElfH {
		file_class: "ELF64 (64-bit)".to_string(),
		endianness: "little endian".to_string(),
		elf_version: "".to_string(),
		target_os_abi: "".to_string(),
		abi_version: "".to_string(),
		ty: "".to_string(),
		machine: "".to_string(),
		version: "".to_string(),
		entry: "".to_string(),
		program_header_offset: "".to_string(),
		section_header_offset: "".to_string(),
		flags: "".to_string(),
		elf_header_size: "".to_string(),
		program_header_entry_size: "".to_string(),
		program_header_count: "".to_string(),
		section_header_entry_size: "".to_string(),
		section_header_count: "".to_string(),
		section_header_index_of_section_name_string_table: "".to_string(),
	};

	// Before fix
	assert_eq!(header.file_class, "ELF64 (64-bit)");
	assert_eq!(header.endianness, "little endian");
	assert_eq!(header.elf_version, ""); // Should remain empty

	header.fix();

	// After fix
	assert_eq!(header.file_class, "ELF64");
	assert_eq!(header.endianness, "little");
	assert_eq!(header.elf_version, ""); // Should still be empty
}

#[test]
fn test_readelf_h_string_operations() -> Rslt<(),>
{
	let header = ReadElfH {
		file_class: "ELF64".to_string(),
		endianness: "little".to_string(),
		elf_version: "1".to_string(),
		target_os_abi: "UNIX - System V".to_string(),
		abi_version: "0".to_string(),
		ty: "EXEC".to_string(),
		machine: "Advanced".to_string(),
		version: "0x1".to_string(),
		entry: "0x401000".to_string(),
		program_header_offset: "64".to_string(),
		section_header_offset: "4096".to_string(),
		flags: "0x0".to_string(),
		elf_header_size: "64".to_string(),
		program_header_entry_size: "56".to_string(),
		program_header_count: "4".to_string(),
		section_header_entry_size: "64".to_string(),
		section_header_count: "10".to_string(),
		section_header_index_of_section_name_string_table: "9".to_string(),
	};

	assert_eq!(
		parse_file_class(&header,)?.to_string(),
		quote::quote! { FileClass::Bit64 }.to_string()
	);
	assert_eq!(
		parse_endianness(&header,)?.to_string(),
		quote::quote! { Endian::Little }.to_string()
	);
	assert_eq!(
		parse_elf_version(&header,)?.to_string(),
		quote::quote! { ElfVersion::ONE }.to_string()
	);
	assert_eq!(
		parse_target_os_abi(&header,)?.to_string(),
		quote::quote! { TargetOsAbi::SysV }.to_string()
	);
	let abi_version = 0u8;
	assert_eq!(
		parse_abi_version(&header,)?.to_string(),
		quote::quote! { AbiVersion(#abi_version) }.to_string()
	);
	assert_eq!(
		parse_ty(&header,)?.to_string(),
		quote::quote! { ElfType::Executable }.to_string()
	);
	assert_eq!(
		parse_machine(&header,).to_string(),
		quote::quote! { ElfHeader::EM_ADVANCED }.to_string()
	);
	let version = 1u32;
	assert_eq!(
		parse_version(&header,)?.to_string(),
		quote::quote! { #version }.to_string()
	);
	let entry = 0x401000u64;
	assert_eq!(
		parse_entry(&header,)?.to_string(),
		quote::quote! { #entry }.to_string()
	);
	let program_header_offset = 64u64;
	assert_eq!(
		parse_program_header_offset(&header,)?.to_string(),
		quote::quote! { #program_header_offset }.to_string()
	);
	let section_header_offset = 4096u64;
	assert_eq!(
		parse_section_header_offset(&header,)?.to_string(),
		quote::quote! { #section_header_offset }.to_string()
	);
	let flags = 0u32;
	assert_eq!(
		parse_flags(&header,)?.to_string(),
		quote::quote! { #flags }.to_string()
	);
	Rslt::new((),)
}

#[test]
fn test_readelf_h_with_unicode_content()
{
	let mut header = ReadElfH {
		target_os_abi: "UNIX - System V with unicode: αβγ".to_string(),
		..Default::default()
	};

	header.fix();

	// target_os_abi is not processed by fix(), so unicode should remain
	assert!(header.target_os_abi.contains("αβγ"));
}

#[test]
fn test_readelf_h_empty_string_handling()
{
	let mut header = ReadElfH {
		file_class: "".to_string(),
		endianness: " ".to_string(),    // Just whitespace
		elf_version: "   ".to_string(), // Multiple spaces
		..Default::default()
	};

	header.fix();

	// Empty strings should remain empty after fix
	assert_eq!(header.file_class, "");
	// Whitespace-only strings should become empty after split
	assert_eq!(header.endianness, "");
	assert_eq!(header.elf_version, "");
}

#[test]
fn test_readelf_h_numeric_field_formats()
{
	let mut header = ReadElfH {
		entry: "0x401000 (entry point)".to_string(),
		program_header_offset: "64 (bytes into file)".to_string(),
		section_header_offset: "4096 (bytes into file)".to_string(),
		flags: "0x0 (no flags)".to_string(),
		elf_header_size: "64 (bytes)".to_string(),
		program_header_entry_size: "56 (bytes)".to_string(),
		program_header_count: "4 (entries)".to_string(),
		section_header_entry_size: "64 (bytes)".to_string(),
		section_header_count: "10 (entries)".to_string(),
		section_header_index_of_section_name_string_table: "9 (section name \
		                                                    string table)"
			.to_string(),
		..Default::default()
	};

	header.fix();

	// All numeric fields should have only the number part
	assert_eq!(header.entry, "0x401000");
	assert_eq!(header.program_header_offset, "64");
	assert_eq!(header.section_header_offset, "4096");
	assert_eq!(header.flags, "0x0");
	assert_eq!(header.elf_header_size, "64");
	assert_eq!(header.program_header_entry_size, "56");
	assert_eq!(header.program_header_count, "4");
	assert_eq!(header.section_header_entry_size, "64");
	assert_eq!(header.section_header_count, "10");
	assert_eq!(header.section_header_index_of_section_name_string_table, "9");
}

#[test]
fn test_readelf_h_architecture_variations()
{
	let architectures = vec![
		("Advanced Micro Devices X86-64", "Advanced",),
		("ARM", "ARM",),
		("Intel 80386", "Intel",),
		("MIPS R3000", "MIPS",),
		("PowerPC", "PowerPC",),
		("SPARC", "SPARC",),
	];

	for (full_arch, expected_first,) in architectures {
		let mut header =
			ReadElfH { machine: full_arch.to_string(), ..Default::default() };

		header.fix();
		assert_eq!(header.machine, expected_first);
	}
}

#[test]
fn test_readelf_h_type_variations()
{
	let types = vec![
		("EXEC (Executable file)", "EXEC",),
		("DYN (Shared object file)", "DYN",),
		("REL (Relocatable file)", "REL",),
		("CORE (Core file)", "CORE",),
	];

	for (full_type, expected_first,) in types {
		let mut header =
			ReadElfH { ty: full_type.to_string(), ..Default::default() };

		header.fix();
		assert_eq!(header.ty, expected_first);
	}
}

#[test]
fn test_property_trait_with_special_characters()
{
	let key_value = vec!["Entry point address", "0x401000"];

	// Should handle strings with spaces
	assert!(key_value.is_peoperty_of("Entry point address"));

	let key_value_special = vec!["OS/ABI", "UNIX - System V"];

	// Should handle strings with special characters
	assert!(key_value_special.is_peoperty_of("OS/ABI"));
}

#[test]
fn test_readelf_h_all_fields_populated() -> Rslt<(),>
{
	let mut header = ReadElfH {
		file_class: "ELF64 (64-bit)".to_string(),
		endianness: "little endian".to_string(),
		elf_version: "1 (current)".to_string(),
		target_os_abi: "UNIX - System V".to_string(),
		abi_version: "0 (current)".to_string(),
		ty: "EXEC (Executable file)".to_string(),
		machine: "Advanced Micro Devices X86-64".to_string(),
		version: "0x1 (current)".to_string(),
		entry: "0x401000 (entry point)".to_string(),
		program_header_offset: "64 (bytes into file)".to_string(),
		section_header_offset: "4096 (bytes into file)".to_string(),
		flags: "0x0 (no flags)".to_string(),
		elf_header_size: "64 (bytes)".to_string(),
		program_header_entry_size: "56 (bytes)".to_string(),
		program_header_count: "4 (entries)".to_string(),
		section_header_entry_size: "64 (bytes)".to_string(),
		section_header_count: "10 (entries)".to_string(),
		section_header_index_of_section_name_string_table: "9 (section name \
		                                                    string table)"
			.to_string(),
	};

	header.fix()?;

	assert_eq!(
		[
			header.file_class.as_str(),
			header.endianness.as_str(),
			header.elf_version.as_str(),
			header.target_os_abi.as_str(),
			header.abi_version.as_str(),
			header.ty.as_str(),
			header.machine.as_str(),
			header.version.as_str(),
			header.entry.as_str(),
			header.program_header_offset.as_str(),
			header.section_header_offset.as_str(),
			header.flags.as_str(),
			header.elf_header_size.as_str(),
			header.program_header_entry_size.as_str(),
			header.program_header_count.as_str(),
			header.section_header_entry_size.as_str(),
			header.section_header_count.as_str(),
			header.section_header_index_of_section_name_string_table.as_str(),
		],
		[
			"ELF64",
			"little",
			"1",
			"UNIX - System V",
			"0",
			"EXEC",
			"Advanced",
			"0x1",
			"0x401000",
			"64",
			"4096",
			"0x0",
			"64",
			"56",
			"4",
			"64",
			"10",
			"9",
		]
	);
	Rslt::new((),)
}
