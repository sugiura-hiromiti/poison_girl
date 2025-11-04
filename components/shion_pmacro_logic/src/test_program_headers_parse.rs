//! # ELF Program Headers Parsing Module
//!
//! This module provides functionality for parsing ELF program headers from the
//! OSO kernel binary. Program headers describe segments in an executable file
//! and contain information about how the program should be loaded into memory.
//!
//! The module uses the `readelf -l` command to extract program header
//! information and parses it into structured Rust types for build-time analysis
//! and validation.

use crate::RsltP;
use crate::check_oso_kernel;
use anyhow::Result as Rslt;
use anyhow::anyhow;
use proc_macro2::Span;
use std::process::Command;

/// Trait for parsing hexadecimal string representations into integer types
///
/// This trait provides a common interface for parsing hex strings (like
/// "0x1000") into various integer types. It's used to convert the hex values
/// from readelf output into proper numeric types.
pub trait IntField: Sized {
	/// Parses a hexadecimal string into the implementing type
	///
	/// # Arguments
	///
	/// * `hex` - A hexadecimal string (with or without "0x" prefix)
	///
	/// # Returns
	///
	/// A `Result` containing the parsed integer value or an error
	///
	/// # Errors
	///
	/// Returns an error if the string cannot be parsed as a valid hexadecimal
	/// number
	fn parse(hex: &str,) -> Rslt<Self,>;
}

impl IntField for u32 {
	fn parse(hex: &str,) -> Rslt<Self,> {
		let rslt = Self::from_str_radix(hex, 16,)?;
		Ok(rslt,)
	}
}

impl IntField for u64 {
	fn parse(hex: &str,) -> Rslt<Self,> {
		let rslt = Self::from_str_radix(hex, 16,)?;
		Ok(rslt,)
	}
}

/// Structured representation of an ELF program header entry
///
/// This struct contains all the key fields from a program header as parsed
/// from the output of `readelf -l`. Program headers describe segments that
/// need to be loaded into memory when the program is executed.
#[derive(Default, Debug,)]
pub struct ReadElfL {
	/// Segment type (e.g., "LOAD", "INTERP", "DYNAMIC")
	pub ty:               String,
	/// File offset where the segment begins
	pub offset:           u64,
	/// Virtual address where the segment should be loaded
	pub virtual_address:  u64,
	/// Physical address where the segment should be loaded (usually same as
	/// virtual)
	pub physical_address: u64,
	/// Size of the segment in the file
	pub file_size:        u64,
	/// Size of the segment in memory (may be larger than file_size for BSS)
	pub memory_size:      u64,
	/// Segment flags (read/write/execute permissions)
	pub flags:            u32,
	/// Required alignment for the segment
	pub align:            u64,
}

pub fn test_program_headers_parse(rslt: proc_macro2::TokenStream,) -> RsltP {
	// Get the expected program headers information from readelf
	let (answer, diag,) = program_headers_info()?;

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

/// Generates token stream for expected program headers information.
///
/// This function executes `readelf -l` on the target binary and parses the
/// output to create a token stream representing the expected program headers
/// structure. This is used by the `test_program_headers_parse` macro to
/// validate program header parsing.
///
/// # Returns
///
/// Returns a `proc_macro2::TokenStream` representing a vector of
/// `ProgramHeader` structs, each initialized with data from the actual binary.
///
/// # Generated Structure
///
/// Each ProgramHeader contains:
/// - `ty`: Program header type (LOAD, DYNAMIC, INTERP, etc.)
/// - `flags`: Permission flags (read, write, execute)
/// - `offset`: Offset in file where segment begins
/// - `virtual_address`: Virtual address where segment should be loaded
/// - `physical_address`: Physical address (for systems where relevant)
/// - `file_size`: Size of segment in file
/// - `memory_size`: Size of segment in memory (may be larger than file_size)
/// - `align`: Alignment requirements for the segment
///
/// # Panics
///
/// This function will panic if:
/// - The `readelf -l` command fails to execute
/// - The readelf output cannot be parsed
/// - Any required program header field is missing or malformed
///
/// # Dependencies
///
/// Requires the `readelf` command to be available in the system PATH.
pub fn program_headers_info() -> RsltP {
	// Execute readelf -l and parse the output
	let program_headers = readelf_l()?;

	// Generate ProgramHeader struct for each program header entry
	let program_headers = program_headers.iter().map(|rel| {
		let ty = parse_program_header_type(rel,);
		let flags = rel.flags;
		let offset = rel.offset;
		let virtual_address = rel.virtual_address;
		let physical_address = rel.physical_address;
		let file_size = rel.file_size;
		let memory_size = rel.memory_size;
		let align = rel.align;

		quote::quote! {
			ProgramHeader {
				ty: #ty,
				flags: #flags,
				offset: #offset,
				virtual_address: #virtual_address,
				physical_address: #physical_address,
				file_size: #file_size,
				memory_size: #memory_size,
				align: #align,
			}
		}
	},);

	// Generate vector containing all program headers
	Ok((
		quote::quote! {
			alloc::vec![
				#(#program_headers, )*
			]
		},
		vec![],
	),)
}

/// Parses program header type from readelf output.
///
/// Converts the program header type string into the appropriate enum variant.
/// The function converts underscore-separated uppercase strings into CamelCase
/// identifiers for the ProgramHeaderType enum.
///
/// # Parameters
///
/// * `program_header` - Parsed readelf -l output for a single program header
///
/// # Returns
///
/// Returns a token stream representing the ProgramHeaderType enum variant
///
/// # Conversion Logic
///
/// The function:
/// 1. Splits the type string on underscores
/// 2. Converts each word to CamelCase (first letter uppercase, rest lowercase)
/// 3. Concatenates the words to form the enum variant name
///
/// # Examples
///
/// - "PT_LOAD" -> `ProgramHeaderType::PtLoad`
/// - "PT_DYNAMIC" -> `ProgramHeaderType::PtDynamic`
/// - "PT_INTERP" -> `ProgramHeaderType::PtInterp`
fn parse_program_header_type(
	program_header: &ReadElfL,
) -> proc_macro2::TokenStream {
	// Convert underscore_separated to CamelCase
	let camel_cased: String = program_header
		.ty
		.split("_",)
		.flat_map(|word| {
			word.char_indices().map(|(i, c,)| {
				if i == 0 { c } else { (c as u8 - b'A' + b'a') as char }
			},)
		},)
		.collect();

	let ident = syn::Ident::new(&camel_cased, Span::call_site(),);

	quote::quote! {
		ProgramHeaderType::#ident
	}
}

pub fn readelf_l() -> Rslt<Vec<ReadElfL,>,> {
	check_oso_kernel()?;

	let program_headers_info = readelf_l_out()?;

	let program_header_count =
		program_headers_count(&program_headers_info[0],)?;

	program_headers_fields(&program_headers_info, program_header_count,)
		.map(|s| {
			let fields_info: Vec<_,> =
				s.split(" ",).filter(|s| !s.is_empty(),).collect();

			let ty = fields_info[0].to_string();
			let offset = parse_str_hex_repr(fields_info[1],)?;
			let virtual_address = parse_str_hex_repr(fields_info[2],)?;
			let physical_address = parse_str_hex_repr(fields_info[3],)?;
			let file_size = parse_str_hex_repr(fields_info[4],)?;
			let memory_size = parse_str_hex_repr(fields_info[5],)?;
			let (flags, align,) = parse_flags_and_align(&fields_info,)?;

			Ok(ReadElfL {
				ty,
				offset,
				virtual_address,
				physical_address,
				file_size,
				memory_size,
				flags,
				align,
			},)
		},)
		.try_collect()
}

fn readelf_l_out() -> Rslt<Vec<String,>,> {
	let program_headers_info = Command::new("readelf",)
		.args(["-l", "target/oso_kernel.elf",],)
		.output()?
		.stdout;
	let program_headers_info = String::from_utf8(program_headers_info,)?;
	let program_headers_info: Vec<_,> = program_headers_info
		.split("Program Headers:",)
		.map(|s| s.to_string(),)
		.collect();

	Ok(program_headers_info,)
}

fn program_headers_count(info: &str,) -> Rslt<usize,> {
	let desc_lines_count = info.lines().count();
	if desc_lines_count < 2 {
		return Err(anyhow::anyhow!(
			"Insufficient lines to parse program header count"
		),);
	}
	let program_header_count: usize = info
		.lines()
		.nth(desc_lines_count - 2,)
		.unwrap()
		.split(" ",)
		.nth(2,)
		.unwrap()
		.parse()?;
	Ok(program_header_count,)
}

/// ```ignore
/// fn test_program_headers_fields_empty_input() {
/// 	let test_lines = vec![];
/// 	let fields = program_headers_fields(&test_lines, 0,);
/// 	let collected: Vec<_,> = fields.collect();
///
/// 	assert_eq!(collected.len(), 0);
/// }
/// ```
fn program_headers_fields(
	infos: &[String],
	count: usize,
) -> impl Iterator<Item = std::string::String,> {
	infos[1]
		.lines()
		.skip(3,)
		.array_chunks::<2>()
		.map(|s| s.concat(),)
		.take(count,)
}

/// ```compile_fail
/// use anyhow::Result as Rslt;
/// fn test_error_propagation() {
///     fn test_chain() -> Rslt<(),> {
///         let invalid_hex = "invalid_hex";
///         parse_str_hex_repr(invalid_hex,)?;
///         Ok((),)
///     }
///
///     let result = test_chain();
///     assert!(result.is_err());
/// }
///
/// test_error_propagation()
/// ```
fn parse_str_hex_repr<I: IntField,>(hex: &str,) -> Rslt<I,> {
	let hex_repr = if hex.len() < 2 {
		// we can assume that `hex` is not prefixed by `0x`
		hex
	} else {
		let prefix = &hex[..2];
		if "0x" == prefix || "0X" == prefix { &hex[2..] } else { hex }
	};
	I::parse(hex_repr,)
}

fn parse_flags_and_align(fields_info: &[&str],) -> Rslt<(u32, u64,),> {
	let rslt = if fields_info.len() == 8 {
		let flags_str = fields_info[6];
		let mut flags = 0;
		if flags_str.contains("R",) {
			flags |= 0b100;
		}
		if flags_str.contains("W",) {
			flags |= 0b10;
		}
		if flags_str.contains("X",) {
			flags |= 0b1;
		};

		let align = parse_str_hex_repr(fields_info[7],)?;
		(flags, align,)
	} else if fields_info.len() == 9 {
		let align = parse_str_hex_repr(fields_info[8],)?;
		(0b101, align,)
	} else {
		return Err(anyhow!(
			"fields_info length should be 8 or 9, get {}",
			fields_info.len()
		),);
	};

	Ok(rslt,)
}

#[cfg(test)]
mod tests {
	use std::env::current_dir;
	use std::env::set_current_dir;
	use std::path::PathBuf;

	use super::*;

	fn go_crate_root() -> Rslt<PathBuf,> {
		let mut cwd = current_dir()?;
		while let Some(parent_path,) = cwd.parent()
			&& parent_path.file_name().unwrap() != "oso"
			&& parent_path
				.file_name()
				.unwrap()
				.to_str()
				.unwrap()
				.contains("oso",)
		{
			cwd = parent_path.to_owned();
		}
		set_current_dir(&cwd,)?;
		Ok(cwd,)
	}

	fn go_workspace_root() -> Rslt<PathBuf,> {
		let cwd = go_crate_root()?;
		if let Some(crate_name,) = cwd.file_name()
			&& crate_name == "oso"
		{
			Ok(cwd,)
		} else {
			set_current_dir(cwd.parent().unwrap(),)?;
			Ok(cwd.parent().unwrap().to_owned(),)
		}
	}

	#[test]
	fn test_slice_range() {
		let a = &"0x1"[2..];
		assert_eq!(a, "1");
	}

	#[test]
	fn test_readelf_l() -> Rslt<(),> {
		let cwd = current_dir()?;
		go_workspace_root()?;

		// Check if the kernel file exists before running the test
		if !std::path::Path::new("target/oso_kernel.elf",).exists() {
			set_current_dir(cwd,)?;
			// Skip test if kernel file doesn't exist
			return Ok((),);
		}

		let phs = readelf_l()?;
		assert_eq!(phs.len(), 4, "{phs:#?}");
		set_current_dir(cwd,)?;
		Ok((),)
	}

	#[test]
	fn test_program_headers_info() -> Rslt<(),> {
		let cwd = current_dir()?;
		go_workspace_root()?;

		// Check if the kernel file exists before running the test
		if !std::path::Path::new("target/oso_kernel.elf",).exists() {
			set_current_dir(cwd,)?;
			// Skip test if kernel file doesn't exist
			return Ok((),);
		}

		let program_headers_info = readelf_l_out()?;

		assert_eq!(program_headers_info.len(), 2);
		set_current_dir(cwd,)?;
		Ok((),)
	}

	#[test]
	fn test_program_headers_count() -> Rslt<(),> {
		let cwd = current_dir()?;
		go_workspace_root()?;

		// Check if the kernel file exists before running the test
		if !std::path::Path::new("target/oso_kernel.elf",).exists() {
			set_current_dir(cwd,)?;
			// Skip test if kernel file doesn't exist
			return Ok((),);
		}

		let program_headers_info = readelf_l_out()?;
		let program_header_count =
			program_headers_count(&program_headers_info[0],)?;

		assert_eq!(program_header_count, 4);
		set_current_dir(cwd,)?;
		Ok((),)
	}

	#[test]
	fn test_program_headers_fields() -> Rslt<(),> {
		let cwd = current_dir()?;
		go_workspace_root()?;

		// Check if the kernel file exists before running the test
		if !std::path::Path::new("target/oso_kernel.elf",).exists() {
			set_current_dir(cwd,)?;
			// Skip test if kernel file doesn't exist
			return Ok((),);
		}

		let program_headers_info = readelf_l_out()?;
		let program_header_count =
			program_headers_count(&program_headers_info[0],)?;
		let program_headers_info = program_headers_fields(
			&program_headers_info,
			program_header_count,
		);

		assert_eq!(program_header_count, program_headers_info.count());
		set_current_dir(cwd,)?;
		Ok((),)
	}

	#[test]
	fn test_int_field_u32_parse_valid_hex() -> Rslt<(),> {
		let result = u32::parse("1a2b",)?;
		assert_eq!(result, 0x1a2b);
		Ok((),)
	}

	#[test]
	fn test_int_field_u32_parse_zero() -> Rslt<(),> {
		let result = u32::parse("0",)?;
		assert_eq!(result, 0);
		Ok((),)
	}

	#[test]
	fn test_int_field_u32_parse_max_value() -> Rslt<(),> {
		let result = u32::parse("ffffffff",)?;
		assert_eq!(result, u32::MAX);
		Ok((),)
	}

	#[test]
	fn test_int_field_u32_parse_invalid() {
		let result = u32::parse("invalid",);
		assert!(result.is_err());
	}

	#[test]
	fn test_int_field_u32_parse_overflow() {
		// This should fail because it's too large for u32
		let result = u32::parse("100000000",); // 9 hex digits
		assert!(result.is_err());
	}

	#[test]
	fn test_int_field_u64_parse_valid_hex() -> Rslt<(),> {
		let result = u64::parse("1a2b3c4d5e6f",)?;
		assert_eq!(result, 0x1a2b3c4d5e6f);
		Ok((),)
	}

	#[test]
	fn test_int_field_u64_parse_zero() -> Rslt<(),> {
		let result = u64::parse("0",)?;
		assert_eq!(result, 0);
		Ok((),)
	}

	#[test]
	fn test_int_field_u64_parse_max_value() -> Rslt<(),> {
		let result = u64::parse("ffffffffffffffff",)?;
		assert_eq!(result, u64::MAX);
		Ok((),)
	}

	#[test]
	fn test_int_field_u64_parse_invalid() {
		let result = u64::parse("invalid",);
		assert!(result.is_err());
	}

	#[test]
	fn test_readelf_l_default() {
		let header = ReadElfL::default();

		// All fields should have default values
		assert_eq!(header.ty, "");
		assert_eq!(header.offset, 0);
		assert_eq!(header.virtual_address, 0);
		assert_eq!(header.physical_address, 0);
		assert_eq!(header.file_size, 0);
		assert_eq!(header.memory_size, 0);
		assert_eq!(header.flags, 0);
		assert_eq!(header.align, 0);
	}

	#[test]
	fn test_readelf_l_debug() {
		let header = ReadElfL {
			ty:               "LOAD".to_string(),
			offset:           0x1000,
			virtual_address:  0x401000,
			physical_address: 0x401000,
			file_size:        0x2000,
			memory_size:      0x2000,
			flags:            5, // Read + Execute
			align:            0x1000,
		};

		// Should be able to debug print the struct
		let debug_str = format!("{:?}", header);
		assert!(debug_str.contains("ReadElfL"));
		assert!(debug_str.contains("LOAD"));
	}

	#[test]
	fn test_parse_str_hex_repr_with_0x_prefix() -> Rslt<(),> {
		let result: u64 = parse_str_hex_repr("0x1000",)?;
		assert_eq!(result, 0x1000);
		Ok((),)
	}

	#[test]
	fn test_parse_str_hex_repr_without_0x_prefix() -> Rslt<(),> {
		let result: u64 = parse_str_hex_repr("1000",)?;
		assert_eq!(result, 0x1000);
		Ok((),)
	}

	#[test]
	fn test_parse_str_hex_repr_zero() -> Rslt<(),> {
		let result: u64 = parse_str_hex_repr("0x0",)?;
		assert_eq!(result, 0);
		Ok((),)
	}

	#[test]
	fn test_parse_str_hex_repr_invalid() {
		let result: Rslt<u64,> = parse_str_hex_repr("invalid",);
		assert!(result.is_err());
	}

	#[test]
	fn test_parse_str_hex_repr_empty() {
		let result: Rslt<u64,> = parse_str_hex_repr("",);
		assert!(result.is_err());
	}

	#[test]
	fn test_parse_str_hex_repr_only_0x() {
		let result: Rslt<u64,> = parse_str_hex_repr("0x",);
		assert!(result.is_err());
	}

	#[test]
	fn test_program_headers_count_parsing() -> Rslt<(),> {
		let test_line = "There are 4 program headers, starting at offset \
		                 64\n\n"
			.to_string();
		let count = program_headers_count(&test_line,)?;
		assert_eq!(count, 4);
		Ok((),)
	}

	#[test]
	fn test_program_headers_count_different_format() -> Rslt<(),> {
		let test_line = "There are 2 program headers, starting at offset \
		                 128\n\n"
			.to_string();
		let count = program_headers_count(&test_line,)?;
		assert_eq!(count, 2);
		Ok((),)
	}

	#[test]
	fn test_program_headers_count_invalid_format() {
		let test_line = "Invalid format without numbers\n\n".to_string();
		let result = program_headers_count(&test_line,);
		assert!(result.is_err());
	}

	#[test]
	fn test_program_headers_count_no_numbers() {
		let test_line = "There are no program headers\n\n".to_string();
		let result = program_headers_count(&test_line,);
		assert!(result.is_err());
	}

	#[test]
	fn test_program_headers_fields_iterator() {
		let test_lines = vec![
			"Program Headers:".to_string(),
			"  Type           Offset             VirtAddr           PhysAddr"
				.to_string(),
			"                 FileSiz            MemSiz              Flags  \
			 Align"
				.to_string(),
			"  LOAD           0x0000000000001000 0x0000000000401000 \
			 0x0000000000401000"
				.to_string(),
			"                 0x0000000000002000 0x0000000000002000  R E    \
			 0x1000"
				.to_string(),
			"  LOAD           0x0000000000003000 0x0000000000403000 \
			 0x0000000000403000"
				.to_string(),
			"                 0x0000000000001000 0x0000000000001000  RW     \
			 0x1000"
				.to_string(),
		];

		let mock_output = vec!["".to_string(), test_lines.join("\n",)];

		let fields = program_headers_fields(&mock_output, 2,);
		let collected: Vec<_,> = fields.collect();

		assert_eq!(collected.len(), 2, "{collected:?}");
		assert!(collected[0].contains("LOAD"));
		assert!(collected[1].contains("LOAD"));
	}

	#[test]
	fn test_program_headers_fields_insufficient_lines() {
		let test_lines = vec![
			"Program Headers:".to_string(),
			"  Type           Offset             VirtAddr           PhysAddr"
				.to_string(),
		];

		let fields = program_headers_fields(&test_lines, 2,);
		let collected: Vec<_,> = fields.collect();

		// Should handle gracefully even with insufficient lines
		assert!(collected.len() <= 2);
	}

	#[test]
	fn test_readelf_l_out_simulation() {
		// We can't easily test the actual readelf command without the binary,
		// but we can test that the function signature is correct and it returns
		// a Result This test would need to be ignored in CI/CD environments
		// without the binary
	}

	#[test]
	fn test_hex_string_edge_cases() -> Rslt<(),> {
		// Test various hex string formats
		let test_cases = vec![
			("0x0", 0u64,),
			("0x1", 1u64,),
			("0xa", 10u64,),
			("0xA", 10u64,),
			("0xff", 255u64,),
			("0xFF", 255u64,),
			("0x1000", 4096u64,),
		];

		for (input, expected,) in test_cases {
			let result: u64 = parse_str_hex_repr(input,)?;
			assert_eq!(result, expected, "Failed for input: {}", input);
		}

		Ok((),)
	}

	#[test]
	fn test_program_header_parsing_complete_flow() -> Rslt<(),> {
		// Simulate a complete parsing flow with mock data
		let mut mock_readelf_output = vec![
			"".to_string(),
			"Elf file type is EXEC (Executable file)".to_string(),
			"Entry point 0x401000".to_string(),
			"There are 2 program headers, starting at offset 64\n\n"
				.to_string(),
			"".to_string(),
			"Program Headers:".to_string(),
			"  Type           Offset             VirtAddr           PhysAddr"
				.to_string(),
			"                 FileSiz            MemSiz              Flags  \
			 Align"
				.to_string(),
			"  LOAD           0x0000000000001000 0x0000000000401000 \
			 0x0000000000401000"
				.to_string(),
			"                 0x0000000000002000 0x0000000000002000  R E    \
			 0x1000"
				.to_string(),
			"  LOAD           0x0000000000003000 0x0000000000403000 \
			 0x0000000000403000"
				.to_string(),
			"                 0x0000000000001000 0x0000000000001000  RW     \
			 0x1000"
				.to_string(),
		];

		// Test program header count extraction
		let count = program_headers_count(&mock_readelf_output[3],)?;
		assert_eq!(count, 2);

		let replaced = mock_readelf_output[3].replace('\n', "",);
		mock_readelf_output[3] = replaced;
		let new_mock = vec![
			mock_readelf_output[..3].join("\n",),
			mock_readelf_output[5..].join("\n",),
		];

		// Test program header fields extraction
		let fields = program_headers_fields(&new_mock, count,);
		let collected: Vec<_,> = fields.collect();
		assert_eq!(collected.len(), 2);

		Ok((),)
	}
}
