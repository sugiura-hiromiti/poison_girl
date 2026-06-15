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
	let result = u32::parse("1a2b",)?;
	assert_eq!(result, 0x1a2b);
	Rslt::new((),)
}

#[test]
fn test_int_field_u32_parse_zero() -> Rslt<(),>
{
	let result = u32::parse("0",)?;
	assert_eq!(result, 0);
	Rslt::new((),)
}

#[test]
fn test_int_field_u32_parse_max_value() -> Rslt<(),>
{
	let result = u32::parse("ffffffff",)?;
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
	let result = u64::parse("1a2b3c4d5e6f",)?;
	assert_eq!(result, 0x1a2b3c4d5e6f);
	Rslt::new((),)
}

#[test]
fn test_int_field_u64_parse_zero() -> Rslt<(),>
{
	let result = u64::parse("0",)?;
	assert_eq!(result, 0);
	Rslt::new((),)
}

#[test]
fn test_int_field_u64_parse_max_value() -> Rslt<(),>
{
	let result = u64::parse("ffffffffffffffff",)?;
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
	let result: u64 = parse_str_hex_repr("0x1000",)?;
	assert_eq!(result, 0x1000);
	Rslt::new((),)
}

#[test]
fn test_parse_str_hex_repr_without_0x_prefix() -> Rslt<(),>
{
	let result: u64 = parse_str_hex_repr("1000",)?;
	assert_eq!(result, 0x1000);
	Rslt::new((),)
}

#[test]
fn test_parse_str_hex_repr_zero() -> Rslt<(),>
{
	let result: u64 = parse_str_hex_repr("0x0",)?;
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
	let test_line =
		"There are 4 program headers, starting at offset 64\n\n".to_string();
	let count = program_headers_count(&test_line,)?;
	assert_eq!(count, 4);
	Rslt::new((),)
}

#[test]
fn test_program_headers_count_different_format() -> Rslt<(),>
{
	let test_line =
		"There are 2 program headers, starting at offset 128\n\n".to_string();
	let count = program_headers_count(&test_line,)?;
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
		"                 FileSiz            MemSiz              Flags  Align"
			.to_string(),
		"  LOAD           0x0000000000001000 0x0000000000401000 \
		 0x0000000000401000"
			.to_string(),
		"                 0x0000000000002000 0x0000000000002000  R E    0x1000"
			.to_string(),
		"  LOAD           0x0000000000003000 0x0000000000403000 \
		 0x0000000000403000"
			.to_string(),
		"                 0x0000000000001000 0x0000000000001000  RW     0x1000"
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
		let result: u64 = parse_str_hex_repr(input,)?;
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
		"There are 2 program headers, starting at offset 64\n\n".to_string(),
		"".to_string(),
		"Program Headers:".to_string(),
		"  Type           Offset             VirtAddr           PhysAddr"
			.to_string(),
		"                 FileSiz            MemSiz              Flags  Align"
			.to_string(),
		"  LOAD           0x0000000000001000 0x0000000000401000 \
		 0x0000000000401000"
			.to_string(),
		"                 0x0000000000002000 0x0000000000002000  R E    0x1000"
			.to_string(),
		"  LOAD           0x0000000000003000 0x0000000000403000 \
		 0x0000000000403000"
			.to_string(),
		"                 0x0000000000001000 0x0000000000001000  RW     0x1000"
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

	Rslt::new((),)
}
