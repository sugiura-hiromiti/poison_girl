//! # ELF Header Parsing Module
//!
//! This module provides functionality for parsing ELF (Executable and Linkable
//! Format) headers from the OSO kernel binary. It uses the `readelf`
//! command-line tool to extract header information and parses it into
//! structured Rust types.
//!
//! The module is primarily used for build-time analysis and validation of the
//! kernel binary to ensure it meets the expected format and requirements.

use crate::RsltP;
use crate::check_oso_kernel;
use crate::oso_proc_macro_helper::Diag;
use anyhow::Result as Rslt;
use anyhow::bail;
use proc_macro2::Span;
use std::process::Command;

/// Structured representation of ELF header information
///
/// This struct contains all the key fields from an ELF header as parsed
/// from the output of `readelf -h`. Each field is stored as a string
/// to preserve the original formatting from readelf.
#[derive(Default, Debug,)]
pub struct ReadElfH {
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

impl ReadElfH {
	/// Cleans up the parsed header fields by removing extra whitespace and
	/// comments
	///
	/// The `readelf` command often includes additional information after the
	/// main value (like comments or explanations). This method extracts just
	/// the first word/value from each field, which is typically the actual
	/// data we need.
	///
	/// # Examples
	///
	/// - "ELF64 (64-bit)" becomes "ELF64"
	/// - "0x401000 (entry point)" becomes "0x401000"
	/// - "little endian" becomes "little"
	fn fix(&mut self,) {
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

/// Trait for checking if a key-value pair matches a specific property name
///
/// This trait provides a method to check if the first element of a vector
/// (representing a parsed key-value pair) matches a given key string.
trait Property {
	/// Checks if this key-value pair represents the specified property
	///
	/// # Arguments
	///
	/// * `key` - The property name to check for
	///
	/// # Returns
	///
	/// `true` if the first element matches the key, `false` otherwise
	fn is_peoperty_of(&self, key: &str,) -> bool;
}

impl Property for Vec<&str,> {
	fn is_peoperty_of(&self, key: &str,) -> bool {
		// Check if the first element (index 0) matches the key
		self.first().is_some_and(|s| *s == key,)
	}
}

pub fn test_elf_header_parse(rslt: proc_macro2::TokenStream,) -> RsltP {
	// Get the expected ELF header information from readelf
	let (answer, diag,) = elf_header_info()?;

	// Generate conditional assertion for debug builds only
	Ok((
		quote::quote! {
			if cfg!(debug_assertions) {
				assert_eq!(#answer, #rslt);
			}
		},
		diag,
	),)
}

/// Generates token stream for expected ELF header information.
///
/// This function executes `readelf -h` on the target binary and parses the
/// output to create a token stream representing the expected ELF header
/// structure. This is used by the `test_elf_header_parse` macro to validate ELF
/// header parsing.
///
/// # Returns
///
/// Returns a `proc_macro2::TokenStream` representing an `ElfHeader` struct
/// initialization with all fields populated from the actual binary.
///
/// # Generated Structure
///
/// The generated token stream creates an ElfHeader with:
/// - `ident`: ELF identification information (class, endianness, version, etc.)
/// - `ty`: ELF file type (executable, shared object, etc.)
/// - `machine`: Target machine architecture
/// - `version`: ELF version
/// - `entry`: Entry point address
/// - `program_header_offset`: Offset to program header table
/// - `section_header_offset`: Offset to section header table
/// - `flags`: Processor-specific flags
/// - `elf_header_size`: Size of ELF header
/// - `program_header_entry_size`: Size of program header entries
/// - `program_header_count`: Number of program header entries
/// - `section_header_entry_size`: Size of section header entries
/// - `section_header_count`: Number of section header entries
/// - `section_header_index_of_section_name_string_table`: Index of section name
///   string table
///
/// # Panics
///
/// This function will panic if:
/// - The `readelf -h` command fails to execute
/// - The readelf output cannot be parsed
/// - Any required ELF header field is missing or malformed
///
/// # Dependencies
///
/// Requires the `readelf` command to be available in the system PATH.
pub fn elf_header_info() -> RsltP {
	// Execute readelf -h and parse the output
	let header = &readelf_h()?;

	// Parse individual ELF header components
	let (ident, mut diag,) = elf_header_ident_build(header,)?;
	let (ty, mut diag_ty,) = parse_ty(header,)?;
	let machine = parse_machine(header,);
	let version = parse_version(header,);
	let entry = parse_entry(header,);
	let program_header_offset = parse_program_header_offset(header,);
	let section_header_offset = parse_section_header_offset(header,);
	let flags = parse_flags(header,);
	let elf_header_size = parse_elf_header_size(header,);
	let program_header_entry_size = parse_program_header_entry_size(header,);
	let program_header_count = parse_program_header_count(header,);
	let section_header_entry_size = parse_section_header_entry_size(header,);
	let section_header_count = parse_section_header_count(header,);
	let section_header_index_of_section_name_string_table =
		parse_section_header_index_of_section_name_string_table(header,);

	diag.append(&mut diag_ty,);

	// Generate the complete ElfHeader struct initialization
	Ok((
		quote::quote! {
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
				section_header_index_of_section_name_string_table : #section_header_index_of_section_name_string_table,
			}
		},
		diag,
	),)
}

/// Builds the ELF header identification structure.
///
/// This function parses the ELF identification fields from the readelf output
/// and generates a token stream representing the `ElfHeaderIdent` structure.
/// The ELF identification contains metadata about the ELF file format.
///
/// # Parameters
///
/// * `header` - Parsed readelf -h output containing ELF header information
///
/// # Returns
///
/// Returns a token stream representing an `ElfHeaderIdent` struct
/// initialization with all identification fields populated.
///
/// # ELF Identification Fields
///
/// - `file_class`: Whether the file is 32-bit or 64-bit
/// - `endianness`: Byte order (little-endian or big-endian)
/// - `elf_version`: ELF format version
/// - `target_os_abi`: Target operating system ABI
/// - `abi_version`: ABI version number
fn elf_header_ident_build(header: &ReadElfH,) -> RsltP {
	let (file_class, mut diag,) = parse_file_class(header,)?;
	let (endianness, mut diag_endianness,) = parse_endianness(header,)?;
	let (elf_version, mut diag_elf_version,) = parse_elf_version(header,)?;
	let (target_os_abi, mut diag_target_os_abi,) =
		parse_target_os_abi(header,)?;
	let (abi_version, mut diag_abi_version,) = parse_abi_version(header,)?;

	diag.append(&mut diag_endianness,);
	diag.append(&mut diag_elf_version,);
	diag.append(&mut diag_target_os_abi,);
	diag.append(&mut diag_abi_version,);

	Ok((
		quote::quote! {
			ElfHeaderIdent {
				file_class: #file_class,
				endianness: #endianness,
				elf_version: #elf_version,
				target_os_abi: #target_os_abi,
				abi_version: #abi_version,
			}
		},
		diag,
	),)
}

/// Parses the ELF file class from readelf output.
///
/// Converts the file class string from readelf into the appropriate enum
/// variant. The file class indicates whether the ELF file is 32-bit or 64-bit.
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing file class information
///
/// # Returns
///
/// Returns a token stream representing the FileClass enum variant
///
/// # Supported Values
///
/// - "ELF64" -> `FileClass::Bit64`
/// - "ELF32" -> `FileClass::Bit32`
///
/// # Panics
///
/// Panics if the file class is not recognized
fn parse_file_class(header: &ReadElfH,) -> RsltP {
	let file_class = header.file_class.as_str();

	let file_class = match file_class {
		"ELF64" => quote::quote! {
			FileClass::Bit64
		},
		"ELF32" => quote::quote! {
			FileClass::Bit32
		},
		_ => {
			bail!("failed to parse file_class info: {file_class}")
		},
	};

	Ok((file_class, vec![],),)
}

/// Parses the endianness from readelf output.
///
/// Converts the endianness string into the appropriate enum variant.
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing endianness information
///
/// # Returns
///
/// Returns a token stream representing the Endian enum variant
///
/// # Supported Values
///
/// - "little" -> `Endian::Little`
/// - "big" -> `Endian::Big`
///
/// # Panics
///
/// Panics if the endianness is not recognized
fn parse_endianness(header: &ReadElfH,) -> RsltP {
	let endianness = header.endianness.as_str();

	let endianness = match endianness {
		"little" => quote::quote! {
			Endian::Little
		},
		"big" => quote::quote! {
			Endian::Big
		},
		_ => {
			bail!("failed to parse endianness info: {endianness}")
		},
	};

	Ok((endianness, vec![],),)
}

/// Parses the ELF version from readelf output.
///
/// Converts the ELF version string into the appropriate constant or creates
/// a new version variant for unrecognized versions.
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing ELF version information
///
/// # Returns
///
/// Returns a token stream representing the ElfVersion constant or variant
///
/// # Behavior
///
/// - Version "1" maps to `ElfVersion::ONE`
/// - Other versions generate a warning and create `ElfVersion(n)` variant
fn parse_elf_version(header: &ReadElfH,) -> RsltP {
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

	Ok((
		elf_version.clone(),
		vec![Diag::Warn(format!("unrecognized elf version: {elf_version}"),)],
	),)
}

/// Parses the target OS ABI from readelf output.
///
/// Converts the target OS ABI string into the appropriate enum variant.
/// The target OS ABI indicates the operating system and ABI for which
/// the ELF file was created.
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing target OS ABI information
///
/// # Returns
///
/// Returns a token stream representing the TargetOsAbi enum variant
///
/// # Supported Values
///
/// - Contains "UNIX - System V" -> `TargetOsAbi::SysV`
/// - Contains "Arm" -> `TargetOsAbi::Arm`
/// - Contains "Standalone" -> `TargetOsAbi::Standalone`
///
/// # Panics
///
/// Panics if the target OS ABI is not recognized
fn parse_target_os_abi(header: &ReadElfH,) -> RsltP {
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
		bail!("unrecognized target_os_abi : {target_os_abi}");
	};

	Ok((target_os_abi, vec![],),)
}

/// Parses the ABI version from readelf output.
///
/// Converts the ABI version string into the appropriate constant or creates
/// a new version variant for unrecognized versions.
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing ABI version information
///
/// # Returns
///
/// Returns a token stream representing the AbiVersion constant or variant
///
/// # Behavior
///
/// - Version "1" maps to `AbiVersion::ONE`
/// - Other versions generate a warning and create `AbiVersion(n)` variant
fn parse_abi_version(header: &ReadElfH,) -> RsltP {
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

	Ok((
		abi_version.clone(),
		vec![Diag::Warn(format!("unrecognized abi version: {abi_version}"),)],
	),)
}

/// Parses the ELF file type from readelf output.
///
/// Converts the ELF type string into the appropriate enum variant.
/// For the OSO kernel, this function specifically validates that the
/// file type is executable.
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing ELF type information
///
/// # Returns
///
/// Returns a token stream representing the ElfType enum variant
///
/// # Behavior
///
/// - Only "EXEC" (executable) type is supported for OSO kernel
/// - Other types will cause a compile-time error
///
/// # Panics
///
/// Panics if the ELF type is not "EXEC" (executable)
fn parse_ty(header: &ReadElfH,) -> RsltP {
	let ty = header.ty.as_str();

	if ty != "EXEC" {
		bail!("oso_kernel.elf type must be executable: {ty}")
	}

	Ok((
		quote::quote! {
			ElfType::Executable
		},
		vec![],
	),)
}

/// Parses the target machine architecture from readelf output.
///
/// Converts the machine string into the appropriate ELF machine constant.
/// The function normalizes the machine name by converting to uppercase
/// and replacing spaces with underscores, then prefixes with "EM_".
///
/// # Parameters
///
/// * `header` - Parsed readelf output containing machine information
///
/// # Returns
///
/// Returns a token stream representing the machine constant
///
/// # Examples
///
/// - "Advanced Micro Devices X86-64" ->
///   `ElfHeader::EM_ADVANCED_MICRO_DEVICES_X86-64`
/// - "AArch64" -> `ElfHeader::EM_AARCH64`
fn parse_machine(header: &ReadElfH,) -> proc_macro2::TokenStream {
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

/// Parses the ELF version field (different from ELF format version).
/// Expects a hexadecimal string prefixed with "0x".
fn parse_version(header: &ReadElfH,) -> proc_macro2::TokenStream {
	let version = header.version.as_str();
	let version = &version[2..]; // Remove "0x" prefix
	let version = u32::from_str_radix(version, 16,).unwrap_or_else(|_| {
		panic!("version must be valid hex number: {version}")
	},);

	quote::quote! {
		#version
	}
}

/// Parses the entry point address from readelf output.
/// Expects a hexadecimal string prefixed with "0x".
fn parse_entry(header: &ReadElfH,) -> proc_macro2::TokenStream {
	let entry = header.entry.as_str();
	let entry = &entry[2..]; // Remove "0x" prefix
	let entry = u64::from_str_radix(entry, 16,).unwrap_or_else(|_| {
		panic!("entry point address must be valid hex number: {entry}")
	},);

	quote::quote! {
		#entry
	}
}

/// Parses the program header table offset from readelf output.
/// Expects a decimal string.
fn parse_program_header_offset(header: &ReadElfH,) -> proc_macro2::TokenStream {
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

/// Parses the section header table offset from readelf output.
/// Expects a decimal string.
fn parse_section_header_offset(header: &ReadElfH,) -> proc_macro2::TokenStream {
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

/// Parses processor-specific flags from readelf output.
/// Expects a hexadecimal string prefixed with "0x".
fn parse_flags(header: &ReadElfH,) -> proc_macro2::TokenStream {
	let flags = header.flags.as_str();
	let flags = &flags[2..]; // Remove "0x" prefix
	let flags = u32::from_str_radix(flags, 16,)
		.unwrap_or_else(|_| panic!("flags must be valid hex number: {flags}"),);

	quote::quote! {
		#flags
	}
}

/// Parses the ELF header size from readelf output.
/// Expects a decimal string.
fn parse_elf_header_size(header: &ReadElfH,) -> proc_macro2::TokenStream {
	let elf_header_size = header.elf_header_size.as_str();
	let elf_header_size = elf_header_size.parse::<u16>().unwrap_or_else(|_| {
		panic!("elf_header_size must be valid hex number: {elf_header_size}")
	},);

	quote::quote! {
		#elf_header_size
	}
}

/// Parses the program header entry size from readelf output.
/// Expects a decimal string.
fn parse_program_header_entry_size(
	header: &ReadElfH,
) -> proc_macro2::TokenStream {
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

/// Parses the number of program header entries from readelf output.
/// Expects a decimal string.
fn parse_program_header_count(header: &ReadElfH,) -> proc_macro2::TokenStream {
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

/// Parses the section header entry size from readelf output.
/// Expects a decimal string.
fn parse_section_header_entry_size(
	header: &ReadElfH,
) -> proc_macro2::TokenStream {
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

/// Parses the number of section header entries from readelf output.
/// Expects a decimal string.
fn parse_section_header_count(header: &ReadElfH,) -> proc_macro2::TokenStream {
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

/// Parses the section header string table index from readelf output.
/// This index points to the section containing section names.
/// Expects a decimal string.
fn parse_section_header_index_of_section_name_string_table(
	header: &ReadElfH,
) -> proc_macro2::TokenStream {
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

/// Parses ELF header information from the OSO kernel binary
///
/// This function executes `readelf -h` on the kernel binary and parses
/// the output to extract all ELF header fields into a structured format.
/// It performs validation to ensure the kernel file exists before parsing.
///
/// # Returns
///
/// A `Result<ReadElfH>` containing the parsed ELF header information,
/// or an error if the kernel file doesn't exist or parsing fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The OSO kernel ELF file doesn't exist (checked via `check_oso_kernel`)
/// - The `readelf` command fails to execute
/// - The command output cannot be parsed as UTF-8
///
/// # Examples
///
/// ```ignore
/// let header = readelf_h()?;
/// println!("Entry point: {}", header.entry);
/// println!("Machine: {}", header.machine);
/// ```
pub fn readelf_h() -> Rslt<ReadElfH,> {
	// Ensure the kernel file exists before attempting to parse it
	check_oso_kernel()?;

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

	Ok(header,)
}
#[cfg(test)]
mod tests {

	use super::*;
	use std::env::current_dir;
	use std::env::set_current_dir;
	use std::path::PathBuf;

	/// Helper function to navigate to the project root for tests
	fn go_root() -> Rslt<PathBuf,> {
		let mut cwd = current_dir()?;
		while let Some(oso_root,) = cwd.parent()
			&& oso_root.file_name().unwrap() != "oso"
		{
			cwd = oso_root.to_owned();
		}
		set_current_dir(&cwd,)?;
		Ok(cwd,)
	}

	#[test]
	fn test_readelf_h_default() {
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
	fn test_readelf_h_fix_method() {
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
	fn test_property_trait_positive() {
		let key_value = vec!["Class", "ELF64"];
		assert!(key_value.is_peoperty_of("Class"));
	}

	#[test]
	fn test_property_trait_negative() {
		let key_value = vec!["Class", "ELF64"];
		assert!(!key_value.is_peoperty_of("Data"));
	}

	#[test]
	fn test_property_trait_single_element() {
		let key_value = vec!["Class"];
		assert!(key_value.is_peoperty_of("Class"));
	}

	#[test]
	fn test_property_trait_multiple_elements() {
		let key_value =
			vec!["Entry point address", "0x401000", "additional", "info"];
		assert!(key_value.is_peoperty_of("Entry point address"));
		assert!(!key_value.is_peoperty_of("0x401000"));
	}

	#[test]
	fn test_debug_trait_implementation() {
		let header = ReadElfH::default();

		// Should be able to debug print the struct
		let debug_str = format!("{:?}", header);
		assert!(debug_str.contains("ReadElfH"));
	}

	#[test]
	fn test_readelf_h_field_parsing_simulation() {
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
	fn test_readelf_h_version_field_handling() {
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
	fn test_readelf_h_fix_method_edge_cases() {
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
	fn test_property_trait_case_sensitivity() {
		let key_value = vec!["Class", "ELF64"];

		// Should match exact case
		assert!(key_value.is_peoperty_of("Class"));

		// Should not match different case
		assert!(!key_value.is_peoperty_of("class"));
		assert!(!key_value.is_peoperty_of("CLASS"));
	}

	#[test]
	fn test_property_trait_partial_matches() {
		let key_value = vec!["Entry point address", "0x401000"];

		// Should match full string
		assert!(key_value.is_peoperty_of("Entry point address"));

		// Should not match partial strings
		assert!(!key_value.is_peoperty_of("Entry"));
		assert!(!key_value.is_peoperty_of("point"));
		assert!(!key_value.is_peoperty_of("address"));
	}

	#[test]
	fn test_property_trait_empty_vector() {
		let key_value: Vec<&str,> = vec![];

		// Should not match anything with empty vector
		assert!(!key_value.is_peoperty_of("Class"));
		assert!(!key_value.is_peoperty_of(""));
	}

	#[test]
	fn test_property_trait_whitespace_handling() {
		let key_value = vec!["  Class  ", "ELF64"];

		// Should not match due to whitespace differences
		assert!(!key_value.is_peoperty_of("Class"));

		// Should match with exact whitespace
		assert!(key_value.is_peoperty_of("  Class  "));
	}

	#[test]
	fn test_readelf_h_with_whitespace_variations() {
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
	fn test_readelf_h_memory_efficiency() {
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
	fn test_readelf_h_clone_behavior() {
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
	fn test_readelf_h_field_independence() {
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
	fn test_readelf_h_string_operations() {
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
	fn test_readelf_h_with_unicode_content() {
		let mut header = ReadElfH {
			target_os_abi: "UNIX - System V with unicode: αβγ".to_string(),
			..Default::default()
		};

		header.fix();

		// target_os_abi is not processed by fix(), so unicode should remain
		assert!(header.target_os_abi.contains("αβγ"));
	}

	#[test]
	fn test_readelf_h_empty_string_handling() {
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
	fn test_readelf_h_numeric_field_formats() {
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
	fn test_readelf_h_architecture_variations() {
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
	fn test_readelf_h_type_variations() {
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
	fn test_property_trait_with_special_characters() {
		let key_value = vec!["Entry point address", "0x401000"];

		// Should handle strings with spaces
		assert!(key_value.is_peoperty_of("Entry point address"));

		let key_value_special = vec!["OS/ABI", "UNIX - System V"];

		// Should handle strings with special characters
		assert!(key_value_special.is_peoperty_of("OS/ABI"));
	}

	#[test]
	fn test_readelf_h_all_fields_populated() {
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
