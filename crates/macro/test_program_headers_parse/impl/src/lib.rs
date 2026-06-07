#![feature(iter_array_chunks)]

use {
	poison_girl_dev_cargo::{Arch, BuildMode, Opts},
	poison_girl_dev_orchestrate::decl_manage::{
		CargoCrate, PoisonGirlCargoInterface, crate_::PoisonGirlCrateChart,
	},
	poison_girl_proc_macro_helper::rslt::Rslt,
	proc_macro2::{Span, TokenStream},
	std::{process::Command, str::FromStr},
};

pub trait IntField: Sized
{
	fn parse(hex: &str,) -> Rslt<Self,>;
}

impl IntField for u32
{
	fn parse(hex: &str,) -> Rslt<Self,>
	{
		let rslt = Self::from_str_radix(hex, 16,)?;
		Rslt::new(rslt,)
	}
}

impl IntField for u64
{
	fn parse(hex: &str,) -> Rslt<Self,>
	{
		let rslt = Self::from_str_radix(hex, 16,)?;
		Rslt::new(rslt,)
	}
}

#[derive(Default, Debug,)]
pub struct ReadElfL
{
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

pub fn test_program_headers_parse(
	rslt: syn::punctuated::Punctuated<syn::Ident, syn::Token![,],>,
) -> Rslt<TokenStream,>
{
	let var_name = rslt.get(0,).expect("expect asserted variable name",);
	let arch = rslt.get(1,).expect("arch not specified",).to_string();
	let build_mode =
		rslt.get(2,).expect("build mode not specified",).to_string();
	program_headers_info(arch, build_mode,).replace_by(|v| {
		Rslt::new(quote::quote! {
			if cfg!(debug_assertions) {
				assert_eq!(#v, #var_name);
			}
		},)
	},)
}

pub fn program_headers_info(
	arch: String,
	build_mode: String,
) -> Rslt<TokenStream,>
{
	readelf_l(arch, build_mode,).replace_by(|program_headers| {
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
		Rslt::new(quote::quote! {
			alloc::vec![
				#(#program_headers, )*
			]
		},)
	},)
}

fn parse_program_header_type(
	program_header: &ReadElfL,
) -> proc_macro2::TokenStream
{
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

pub fn readelf_l(arch: String, build_mode: String,) -> Rslt<Vec<ReadElfL,>,>
{
	readelf_l_out(arch, build_mode,)
		.replace_by(|program_headers_info| {
			program_headers_count(&program_headers_info[0],)
				.replace_by(|count| Rslt::new((count, program_headers_info,),),)
		},)
		.replace_by(|(count, info,)| {
			program_headers_fields(&info, count,)
				.map(|s| {
					let fields_info: Vec<_,> =
						s.split(" ",).filter(|s| !s.is_empty(),).collect();

					let ty = fields_info[0].to_string();
					let offset = parse_str_hex_repr(fields_info[1],)??;
					let virtual_address = parse_str_hex_repr(fields_info[2],)??;
					let physical_address =
						parse_str_hex_repr(fields_info[3],)??;
					let file_size = parse_str_hex_repr(fields_info[4],)??;
					let memory_size = parse_str_hex_repr(fields_info[5],)??;
					let (flags, align,) =
						parse_flags_and_align(&fields_info,)??;

					Rslt::new(ReadElfL {
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
				.fold(Rslt::new(vec![],), |acc, field| acc.push_elem(field,),)
		},)
}

fn readelf_l_out(arch: String, build_mode: String,) -> Rslt<Vec<String,>,>
{
	let arch = Arch::from_str(&arch,)?;
	let build_mode = BuildMode::from_str(&build_mode,)?;
	let kernel_crate = PoisonGirlCargoInterface::new(
		PoisonGirlCrateChart::Kernel,
		Opts { arch, build_mode, ..Default::default() },
	);
	let kernel_bin_path = kernel_crate.build_artifact()?;

	let program_headers_info = Command::new("readelf",)
		.arg("-l",)
		.arg(kernel_bin_path,)
		.output()?
		.stdout;
	let program_headers_info = String::from_utf8(program_headers_info,)?;
	let program_headers_info: Vec<_,> = program_headers_info
		.split("Program Headers:",)
		.map(|s| s.to_string(),)
		.collect();

	Rslt::new(program_headers_info,)
}

fn program_headers_count(info: &str,) -> Rslt<usize,>
{
	let desc_lines_count = info.lines().count();
	if desc_lines_count < 2 {
		return Rslt::new_err(
			"Insufficient lines to parse program header count",
		);
	}
	let program_header_count: usize = info
		.lines()
		.nth(desc_lines_count - 2,)
		.unwrap()
		.split(" ",)
		.nth(2,)
		.unwrap()
		.parse()?;
	Rslt::new(program_header_count,)
}

fn program_headers_fields(
	infos: &[String],
	count: usize,
) -> impl Iterator<Item = std::string::String,>
{
	infos[1]
		.lines()
		.skip(3,)
		.array_chunks::<2>()
		.map(|s| s.concat(),)
		.take(count,)
}

fn parse_str_hex_repr<I: IntField,>(hex: &str,) -> Rslt<I,>
{
	let hex_repr = if hex.len() < 2 {
		// we can assume that `hex` is not prefixed by `0x`
		hex
	} else {
		let prefix = &hex[..2];
		if "0x" == prefix || "0X" == prefix { &hex[2..] } else { hex }
	};
	I::parse(hex_repr,)
}

fn parse_flags_and_align(fields_info: &[&str],) -> Rslt<(u32, u64,),>
{
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

		let align = parse_str_hex_repr(fields_info[7],)??;
		(flags, align,)
	} else if fields_info.len() == 9 {
		let align = parse_str_hex_repr(fields_info[8],)??;
		(0b101, align,)
	} else {
		return Rslt::new_err(format!(
			"fields_info length should be 8 or 9, get {}",
			fields_info.len()
		),);
	};

	Rslt::new(rslt,)
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_slice_range()
	{
		let a = &"0x1"[2..];
		assert_eq!(a, "1");
	}

	#[test]
	fn test_int_field_u32_parse_valid_hex() -> Rslt<(),>
	{
		let result = u32::parse("1a2b",)??;
		assert_eq!(result, 0x1a2b);
		Rslt::new((),)
	}

	#[test]
	fn test_int_field_u32_parse_zero() -> Rslt<(),>
	{
		let result = u32::parse("0",)??;
		assert_eq!(result, 0);
		Rslt::new((),)
	}

	#[test]
	fn test_int_field_u32_parse_max_value() -> Rslt<(),>
	{
		let result = u32::parse("ffffffff",)??;
		assert_eq!(result, u32::MAX);
		Rslt::new((),)
	}

	#[test]
	fn test_int_field_u32_parse_invalid()
	{
		let result = u32::parse("invalid",);
		assert!(result.has_err());
	}

	#[test]
	fn test_int_field_u32_parse_overflow()
	{
		// This should fail because it's too large for u32
		let result = u32::parse("100000000",); // 9 hex digits
		assert!(result.has_err());
	}

	#[test]
	fn test_int_field_u64_parse_valid_hex() -> Rslt<(),>
	{
		let result = u64::parse("1a2b3c4d5e6f",)??;
		assert_eq!(result, 0x1a2b3c4d5e6f);
		Rslt::new((),)
	}

	#[test]
	fn test_int_field_u64_parse_zero() -> Rslt<(),>
	{
		let result = u64::parse("0",)??;
		assert_eq!(result, 0);
		Rslt::new((),)
	}

	#[test]
	fn test_int_field_u64_parse_max_value() -> Rslt<(),>
	{
		let result = u64::parse("ffffffffffffffff",)??;
		assert_eq!(result, u64::MAX);
		Rslt::new((),)
	}

	#[test]
	fn test_int_field_u64_parse_invalid()
	{
		let result = u64::parse("invalid",);
		assert!(result.has_err());
	}

	#[test]
	fn test_readelf_l_default()
	{
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
	fn test_readelf_l_debug()
	{
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
	fn test_parse_str_hex_repr_with_0x_prefix() -> Rslt<(),>
	{
		let result: u64 = parse_str_hex_repr("0x1000",)??;
		assert_eq!(result, 0x1000);
		Rslt::new((),)
	}

	#[test]
	fn test_parse_str_hex_repr_without_0x_prefix() -> Rslt<(),>
	{
		let result: u64 = parse_str_hex_repr("1000",)??;
		assert_eq!(result, 0x1000);
		Rslt::new((),)
	}

	#[test]
	fn test_parse_str_hex_repr_zero() -> Rslt<(),>
	{
		let result: u64 = parse_str_hex_repr("0x0",)??;
		assert_eq!(result, 0);
		Rslt::new((),)
	}

	#[test]
	fn test_parse_str_hex_repr_invalid()
	{
		let result: Rslt<u64,> = parse_str_hex_repr("invalid",);
		assert!(result.has_err());
	}

	#[test]
	fn test_parse_str_hex_repr_empty()
	{
		let result: Rslt<u64,> = parse_str_hex_repr("",);
		assert!(result.has_err());
	}

	#[test]
	fn test_parse_str_hex_repr_only_0x()
	{
		let result: Rslt<u64,> = parse_str_hex_repr("0x",);
		assert!(result.has_err());
	}

	#[test]
	fn test_program_headers_count_parsing() -> Rslt<(),>
	{
		let test_line = "There are 4 program headers, starting at offset \
		                 64\n\n"
			.to_string();
		let count = program_headers_count(&test_line,)??;
		assert_eq!(count, 4);
		Rslt::new((),)
	}

	#[test]
	fn test_program_headers_count_different_format() -> Rslt<(),>
	{
		let test_line = "There are 2 program headers, starting at offset \
		                 128\n\n"
			.to_string();
		let count = program_headers_count(&test_line,)??;
		assert_eq!(count, 2);
		Rslt::new((),)
	}

	#[test]
	fn test_program_headers_count_invalid_format()
	{
		let test_line = "Invalid format without numbers\n\n".to_string();
		let result = program_headers_count(&test_line,);
		assert!(result.has_err());
	}

	#[test]
	fn test_program_headers_count_no_numbers()
	{
		let test_line = "There are no program headers\n\n".to_string();
		let result = program_headers_count(&test_line,);
		assert!(result.has_err());
	}

	#[test]
	fn test_program_headers_fields_iterator()
	{
		let test_lines = [
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
	fn test_program_headers_fields_insufficient_lines()
	{
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
	fn test_hex_string_edge_cases() -> Rslt<(),>
	{
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
			let result: u64 = parse_str_hex_repr(input,)??;
			assert_eq!(result, expected, "Failed for input: {}", input);
		}

		Rslt::new((),)
	}

	#[test]
	fn test_program_header_parsing_complete_flow() -> Rslt<(),>
	{
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
		let count = program_headers_count(&mock_readelf_output[3],)??;
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

		Rslt::new((),)
	}
}
