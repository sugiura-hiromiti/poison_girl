use {
	poison_girl_dev_fs::fs::check_poison_girl_kernel,
	poison_girl_proc_macro_helper::{diagnostic::Diag, rslt::Rslt},
	proc_macro2::{Span, TokenStream},
	std::process::Command,
};

#[derive(Default, Debug,)]
pub struct ReadElfH
{
	/// ELF file class (32-bit or 64-bit)
	pub file_class: String,
	/// Data encoding (little-endian or big-endian)
	pub endianness: String,
	/// ELF version number
	pub elf_version: String,
	/// Target OS/ABI identification
	pub target_os_abi: String,
	/// ABI version number
	pub abi_version: String,
	/// Object file type (executable, shared object, etc.)
	pub ty: String,
	/// Target machine architecture
	pub machine: String,
	/// Object file version
	pub version: String,
	/// Entry point virtual address
	pub entry: String,
	/// Program header table file offset
	pub program_header_offset: String,
	/// Section header table file offset
	pub section_header_offset: String,
	/// Processor-specific flags
	pub flags: String,
	/// ELF header size in bytes
	pub elf_header_size: String,
	/// Program header table entry size
	pub program_header_entry_size: String,
	/// Number of program header table entries
	pub program_header_count: String,
	/// Section header table entry size
	pub section_header_entry_size: String,
	/// Number of section header table entries
	pub section_header_count: String,
	/// Section header string table index
	pub section_header_index_of_section_name_string_table: String,
}

impl ReadElfH
{
	fn fix(&mut self,)
	{
		// Extract first word from each field (split on whitespace)
		self.file_class = self
			.file_class
			.split_whitespace()
			.next()
			.unwrap_or("",)
			.to_string();
		self.endianness = self
			.endianness
			.split_whitespace()
			.next()
			.unwrap_or("",)
			.to_string();
		self.elf_version = self
			.elf_version
			.split_whitespace()
			.next()
			.unwrap_or("",)
			.to_string();
		// Note: target_os_abi is intentionally not processed as it may contain
		// spaces
		self.abi_version =
			self.abi_version.split(" ",).nth(0,).unwrap().to_string();
		self.ty = self.ty.split(" ",).nth(0,).unwrap().to_string();
		self.machine = self.machine.split(" ",).nth(0,).unwrap().to_string();
		self.version = self.version.split(" ",).nth(0,).unwrap().to_string();
		self.entry = self.entry.split(" ",).nth(0,).unwrap().to_string();
		self.program_header_offset =
			self.program_header_offset.split(" ",).nth(0,).unwrap().to_string();
		self.section_header_offset =
			self.section_header_offset.split(" ",).nth(0,).unwrap().to_string();
		self.flags = self.flags.split(" ",).nth(0,).unwrap().to_string();
		self.elf_header_size =
			self.elf_header_size.split(" ",).nth(0,).unwrap().to_string();
		self.program_header_entry_size = self
			.program_header_entry_size
			.split(" ",)
			.nth(0,)
			.unwrap()
			.to_string();
		self.program_header_count =
			self.program_header_count.split(" ",).nth(0,).unwrap().to_string();
		self.section_header_entry_size = self
			.section_header_entry_size
			.split(" ",)
			.nth(0,)
			.unwrap()
			.to_string();
		self.section_header_count =
			self.section_header_count.split(" ",).nth(0,).unwrap().to_string();
		self.section_header_index_of_section_name_string_table = self
			.section_header_index_of_section_name_string_table
			.split(" ",)
			.nth(0,)
			.unwrap()
			.to_string();
	}
}

trait Property
{
	fn is_peoperty_of(&self, key: &str,) -> bool;
}

impl Property for Vec<&str,>
{
	fn is_peoperty_of(&self, key: &str,) -> bool
	{
		// Check if the first element (index 0) matches the key
		self.first().is_some_and(|s| *s == key,)
	}
}

pub fn test_elf_header_parse(
	rslt: proc_macro2::TokenStream,
) -> Rslt<TokenStream,>
{
	elf_header_info().replace_by(|ts| {
		Rslt::new(quote::quote! {
		if cfg!(debug_assertions) {
			assert_eq!(#ts, #rslt);
		}
			},)
	},)
}

pub fn elf_header_info() -> Rslt<TokenStream,>
{
	readelf_h()
		.replace_by(|header| {
			elf_header_ident_build(&header,)
				.replace_by(|ident| Rslt::new((ident, header,),),)
		},)
		.replace_by(|(ident, header,)| {
			parse_ty(&header,)
				.replace_by(|ty| Rslt::new((ident, ty, header,),),)
		},)
		.replace_by(|(ident, ty, ref header,)| {
			let machine = parse_machine(header,);
			let version = parse_version(header,);
			let entry = parse_entry(header,);
			let program_header_offset = parse_program_header_offset(header,);
			let section_header_offset = parse_section_header_offset(header,);
			let flags = parse_flags(header,);
			let elf_header_size = parse_elf_header_size(header,);
			let program_header_entry_size =
				parse_program_header_entry_size(header,);
			let program_header_count = parse_program_header_count(header,);
			let section_header_entry_size =
				parse_section_header_entry_size(header,);
			let section_header_count = parse_section_header_count(header,);
			let section_header_index_of_section_name_string_table =
				parse_section_header_index_of_section_name_string_table(header,);
			Rslt::new(quote::quote! {
					ElfHeader {
						ident: #ident,
						ty : #ty,
						machine : #machine,
						version : #version,
						entry : #entry,
						program_header_offset : #program_header_offset,
						section_header_offset : #section_header_offset,
						flags : #flags,
						elf_header_size : #elf_header_size,
						program_header_entry_size : #program_header_entry_size,
						program_header_count : #program_header_count,
						section_header_entry_size : #section_header_entry_size,
						section_header_count : #section_header_count,
						section_header_index_of_section_name_string_table :
			#section_header_index_of_section_name_string_table, 		}
				},)
		},)
}

fn elf_header_ident_build(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	parse_file_class(header,).replace_by(|file_class| {
		parse_endianness(header,).replace_by(|endianness| {
			parse_elf_version(header,).replace_by(|elf_version| {
				parse_target_os_abi(header,).replace_by(|target_os_abi| {
					parse_abi_version(header,).replace_by(|abi_version| {
						Rslt::new(quote::quote! {
							ElfHeaderIdent {
								file_class: #file_class,
								endianness: #endianness,
								elf_version: #elf_version,
								target_os_abi: #target_os_abi,
								abi_version: #abi_version,
							}
						},)
					},)
				},)
			},)
		},)
	},)
}

fn parse_file_class(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let file_class = header.file_class.as_str();

	let file_class = match file_class {
		"ELF64" => quote::quote! {
			FileClass::Bit64
		},
		"ELF32" => quote::quote! {
			FileClass::Bit32
		},
		_ => {
			return Rslt::new_err(format!(
				"failed to parse file_class info: {file_class}"
			),);
		},
	};

	Rslt::new(file_class,)
}

fn parse_endianness(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let endianness = header.endianness.as_str();

	let endianness = match endianness {
		"little" => quote::quote! {
			Endian::Little
		},
		"big" => quote::quote! {
			Endian::Big
		},
		_ => {
			return Rslt::new_err(format!(
				"failed to parse endianness info: {endianness}"
			),);
		},
	};

	Rslt::new(endianness,)
}

fn parse_elf_version(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let elf_version = header.elf_version.as_str();

	let elf_version = match elf_version {
		"1" => quote::quote! {
			ElfVersion::ONE
		},
		ver => {
			let ver: u8 = ver.parse()?;
			quote::quote! {
				ElfVersion(#ver)
			}
		},
	};

	Rslt::new(elf_version.clone(),).with_diag(Diag::note(format!(
		"unrecognized elf version: {elf_version}"
	),),)
}

fn parse_target_os_abi(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let target_os_abi = header.target_os_abi.as_str();

	let target_os_abi = if target_os_abi.contains("UNIX - System V",) {
		quote::quote! {
		TargetOsAbi::SysV
			}
	} else if target_os_abi.contains("Arm",) {
		quote::quote! {
			TargetOsAbi::Arm
		}
	} else if target_os_abi.contains("Standalone",) {
		quote::quote! {
			TargetOsAbi::Standalone
		}
	} else {
		return Rslt::new_err(format!("target_os_abi : {target_os_abi}"),);
	};

	Rslt::new(target_os_abi,)
}

fn parse_abi_version(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let abi_version = header.abi_version.as_str();

	let abi_version = match abi_version {
		"1" => quote::quote! {
			AbiVersion::ONE
		},
		ver => {
			let ver: u8 = ver.parse()?;
			quote::quote! {
				AbiVersion(#ver)
			}
		},
	};

	Rslt::new(abi_version.clone(),).with_diag(Diag::note(format!(
		"unrecognized abi version: {abi_version}"
	),),)
}

fn parse_ty(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let ty = header.ty.as_str();

	if ty != "EXEC" {
		return Rslt::new_err(format!(
			"oso_kernel.elf type must be executable: {ty}"
		),);
	}

	Rslt::new(quote::quote! {
		ElfType::Executable
	},)
}

fn parse_machine(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	// Normalize machine name: uppercase and replace spaces with underscores
	let machine: String = header
		.machine
		.as_str()
		.chars()
		.map(|c| match c {
			cap if cap.is_ascii_lowercase() => {
				(cap as u8 + b'A' - b'a') as char
			},
			' ' => '_',
			_ => c,
		},)
		.collect();

	// Create the machine constant identifier
	let mut machine_const = "EM_".to_string();
	machine_const.push_str(&machine,);
	let machine = syn::Ident::new(&machine_const, Span::call_site(),);

	quote::quote! {
		ElfHeader::#machine
	}
}

fn parse_version(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let version = header.version.as_str();
	let version = &version[2..]; // Remove "0x" prefix
	let version = u32::from_str_radix(version, 16,).unwrap_or_else(|_| {
		panic!("version must be valid hex number: {version}")
	},);

	quote::quote! {
		#version
	}
}

fn parse_entry(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let entry = header.entry.as_str();
	let entry = &entry[2..]; // Remove "0x" prefix
	let entry = u64::from_str_radix(entry, 16,).unwrap_or_else(|_| {
		panic!("entry point address must be valid hex number: {entry}")
	},);

	quote::quote! {
		#entry
	}
}

fn parse_program_header_offset(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let program_header_offset = header.program_header_offset.as_str();
	let program_header_offset =
		program_header_offset.parse::<u64>().unwrap_or_else(|_| {
			panic!(
				"program_header_offset address must be valid hex number: \
				 {program_header_offset}"
			)
		},);

	quote::quote! {
		#program_header_offset
	}
}

fn parse_section_header_offset(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let section_header_offset = header.section_header_offset.as_str();
	let section_header_offset =
		section_header_offset.parse::<u64>().unwrap_or_else(|_| {
			panic!(
				"section_header_offset address must be valid hex number: \
				 {section_header_offset}"
			)
		},);

	quote::quote! {
		#section_header_offset
	}
}

fn parse_flags(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let flags = header.flags.as_str();
	let flags = &flags[2..]; // Remove "0x" prefix
	let flags = u32::from_str_radix(flags, 16,)
		.unwrap_or_else(|_| panic!("flags must be valid hex number: {flags}"),);

	quote::quote! {
		#flags
	}
}

fn parse_elf_header_size(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let elf_header_size = header.elf_header_size.as_str();
	let elf_header_size = elf_header_size.parse::<u16>().unwrap_or_else(|_| {
		panic!("elf_header_size must be valid hex number: {elf_header_size}")
	},);

	quote::quote! {
		#elf_header_size
	}
}

fn parse_program_header_entry_size(
	header: &ReadElfH,
) -> proc_macro2::TokenStream
{
	let program_header_entry_size = header.program_header_entry_size.as_str();
	let program_header_entry_size =
		program_header_entry_size.parse::<u16>().unwrap_or_else(|_| {
			panic!(
				"program_header_entry_size must be valid hex number: \
				 {program_header_entry_size}"
			)
		},);

	quote::quote! {
		#program_header_entry_size
	}
}

fn parse_program_header_count(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let program_header_count = header.program_header_count.as_str();
	let program_header_count =
		program_header_count.parse::<u16>().unwrap_or_else(|_| {
			panic!(
				"program_header_count must be valid hex number: \
				 {program_header_count}"
			)
		},);

	quote::quote! {
		#program_header_count
	}
}

fn parse_section_header_entry_size(
	header: &ReadElfH,
) -> proc_macro2::TokenStream
{
	let section_header_entry_size = header.section_header_entry_size.as_str();
	let section_header_entry_size =
		section_header_entry_size.parse::<u16>().unwrap_or_else(|_| {
			panic!(
				"section_header_entry_size must be valid hex number: \
				 {section_header_entry_size}"
			)
		},);

	quote::quote! {
		#section_header_entry_size
	}
}

fn parse_section_header_count(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	let section_header_count = header.section_header_count.as_str();
	let section_header_count =
		section_header_count.parse::<u16>().unwrap_or_else(|_| {
			panic!(
				"section_header_count must be valid hex number: \
				 {section_header_count}"
			)
		},);

	quote::quote! {
		#section_header_count
	}
}

fn parse_section_header_index_of_section_name_string_table(
	header: &ReadElfH,
) -> proc_macro2::TokenStream
{
	let section_header_index_of_section_name_string_table =
		header.section_header_index_of_section_name_string_table.as_str();
	let section_header_index_of_section_name_string_table =
		section_header_index_of_section_name_string_table
			.parse::<u16>()
			.unwrap_or_else(|_| {
				panic!(
					"section_header_index_of_section_name_string_table must \
					 be valid hex number: \
					 {section_header_index_of_section_name_string_table}"
				)
			},);

	quote::quote! {
		#section_header_index_of_section_name_string_table
	}
}

pub fn readelf_h() -> Rslt<ReadElfH,>
{
	// Ensure the kernel file exists before attempting to parse it
	check_poison_girl_kernel()?;

	// Execute readelf command to get header information
	let header_info = Command::new("readelf",)
		.args(["-h", "target/oso_kernel.elf",],)
		.output()?
		.stdout;

	// Convert command output to string
	let header_info = String::from_utf8(header_info,)?;

	// Initialize default header struct
	let mut header = ReadElfH::default();

	// Parse each line of readelf output
	header_info.lines().for_each(|line| {
		// Split each line on ':' to get key-value pairs
		let key_value: Vec<_,> = line.split(':',).map(|s| s.trim(),).collect();

		// Parse each field based on the key name
		if key_value.is_peoperty_of("Class",) {
			header.file_class = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Data",) {
			// Extract endianness from "2's complement, little endian" format
			header.endianness =
				key_value[1].split(" ",).nth(2,).unwrap().to_string();
		}
		if key_value.is_peoperty_of("Version",) {
			// Handle both ELF version and object version fields
			if key_value[1].contains("0x",) {
				header.version = key_value[1].to_string();
			} else {
				header.elf_version =
					key_value[1].split(" ",).next().unwrap().to_string();
			}
		}
		if key_value.is_peoperty_of("OS/ABI",) {
			header.target_os_abi = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("ABI Version",) {
			header.abi_version = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Type",) {
			header.ty = key_value[1].split(" ",).next().unwrap().to_string();
		}
		if key_value.is_peoperty_of("Machine",) {
			header.machine = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Entry point address",) {
			header.entry = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Start of program headers",) {
			header.program_header_offset =
				key_value[1].split(" ",).next().unwrap().to_string();
		}
		if key_value.is_peoperty_of("Start of section headers",) {
			header.section_header_offset = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Flags",) {
			header.flags = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Size of this header",) {
			header.elf_header_size =
				key_value[1].split(" ",).next().unwrap().to_string();
		}
		if key_value.is_peoperty_of("Size of program headers",) {
			header.program_header_entry_size =
				key_value[1].split(" ",).next().unwrap().to_string();
		}
		if key_value.is_peoperty_of("Number of program headers",) {
			header.program_header_count = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Size of section headers",) {
			header.section_header_entry_size = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Number of section headers",) {
			header.section_header_count = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Section header string table index",) {
			header.section_header_index_of_section_name_string_table =
				key_value[1].to_string();
		}
	},);

	// Clean up the parsed fields by removing extra whitespace and comments
	header.fix();

	Rslt::new(header,)
}
#[cfg(test)]
mod tests
{

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
		assert_eq!(
			header.section_header_index_of_section_name_string_table,
			""
		);
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
			section_header_index_of_section_name_string_table:
				"9 (string table index)".to_string(),
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
		assert_eq!(
			header.section_header_index_of_section_name_string_table,
			"9"
		);
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
	fn test_readelf_h_field_parsing_simulation()
	{
		// Simulate parsing different types of readelf output lines
		let test_cases = vec![
			("Class:                             ELF64", "Class", "ELF64",),
			(
				"Data:                              2's complement, little \
				 endian",
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
				"Machine:                           Advanced Micro Devices \
				 X86-64",
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
			let key_value: Vec<_,> =
				line.split(':',).map(|s| s.trim(),).collect();

			if key_value.len() >= 2 {
				assert_eq!(key_value[0], expected_key);

				if expected_key != "OS/ABI" {
					// OS/ABI is special case
					let first_word = key_value[1].split(' ',).nth(0,).unwrap();
					assert_eq!(first_word, expected_first_word);
				}
			}
		}
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
		assert_eq!(
			header.section_header_index_of_section_name_string_table,
			"0"
		);
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
		let mut header = ReadElfH::default();

		// Test that modifying one field doesn't affect others
		header.file_class = "ELF64 (64-bit)".to_string();
		header.endianness = "little endian".to_string();

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
	fn test_readelf_h_string_operations()
	{
		// Test various string operations that might be used with ReadElfH
		let header = ReadElfH {
			file_class: "ELF64".to_string(),
			entry: "0x401000".to_string(),
			..Default::default()
		};

		// Test string comparisons
		assert_eq!(header.file_class, "ELF64");
		assert_ne!(header.file_class, "ELF32");

		// Test string contains
		assert!(header.entry.contains("0x"));
		assert!(header.entry.contains("401000"));

		// Test string length
		assert_eq!(header.file_class.len(), 5);
		assert_eq!(header.entry.len(), 8);
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
		let mut header =
			ReadElfH {
				entry: "0x401000 (entry point)".to_string(),
				program_header_offset: "64 (bytes into file)".to_string(),
				section_header_offset: "4096 (bytes into file)".to_string(),
				flags: "0x0 (no flags)".to_string(),
				elf_header_size: "64 (bytes)".to_string(),
				program_header_entry_size: "56 (bytes)".to_string(),
				program_header_count: "4 (entries)".to_string(),
				section_header_entry_size: "64 (bytes)".to_string(),
				section_header_count: "10 (entries)".to_string(),
				section_header_index_of_section_name_string_table:
					"9 (section name string table)".to_string(),
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
		assert_eq!(
			header.section_header_index_of_section_name_string_table,
			"9"
		);
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
			let mut header = ReadElfH {
				machine: full_arch.to_string(),
				..Default::default()
			};

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
	fn test_readelf_h_all_fields_populated()
	{
		let mut header =
			ReadElfH {
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
				section_header_index_of_section_name_string_table:
					"9 (section name string table)".to_string(),
			};

		header.fix();

		// Verify all fields are properly cleaned
		assert!(!header.file_class.is_empty());
		assert!(!header.endianness.is_empty());
		assert!(!header.elf_version.is_empty());
		assert!(!header.target_os_abi.is_empty());
		assert!(!header.abi_version.is_empty());
		assert!(!header.ty.is_empty());
		assert!(!header.machine.is_empty());
		assert!(!header.version.is_empty());
		assert!(!header.entry.is_empty());
		assert!(!header.program_header_offset.is_empty());
		assert!(!header.section_header_offset.is_empty());
		assert!(!header.flags.is_empty());
		assert!(!header.elf_header_size.is_empty());
		assert!(!header.program_header_entry_size.is_empty());
		assert!(!header.program_header_count.is_empty());
		assert!(!header.section_header_entry_size.is_empty());
		assert!(!header.section_header_count.is_empty());
		assert!(
			!header
				.section_header_index_of_section_name_string_table
				.is_empty()
		);
	}
}
