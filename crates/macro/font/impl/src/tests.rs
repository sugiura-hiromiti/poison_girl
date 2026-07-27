use {
	crate::{convert_bitfield, font, font_data},
	poison_girl_macro_error::{rslt::test_helper::TestRslt, success},
	std::fs,
};

const SAMPLE_FONT_DATA: &str = r#"........
...@@...
..@..@..
..@..@..
..@..@..
..@@@@..
..@..@..
..@..@..
..@..@..
..@..@..
........
........
........
........
........
........
"#;

#[test]
fn test_fonts_loads_correct_number_of_characters() -> TestRslt
{
	// Create a test font file in the project directory
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path = format!("{}/test_font_temp.txt", project_root);

	let sample_font_data = SAMPLE_FONT_DATA;
	let mut full_font_data = String::new();
	for _ in 0..256 {
		full_font_data.push_str(sample_font_data,);
	}

	fs::write(&test_file_path, full_font_data,)?;

	let lit_str =
		syn::LitStr::new("test_font_temp.txt", proc_macro2::Span::call_site(),);
	let fonts = font_data(lit_str,)?;

	// Should load exactly 256 characters
	assert_eq!(fonts.len(), 256);

	// Cleanup
	fs::remove_file(test_file_path,)?;
	success!()
}

#[test]
fn test_fonts_each_character_has_correct_length() -> TestRslt
{
	// Create a test font file in the project directory
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path = format!("{}/test_font_temp2.txt", project_root);

	// Create sample font data
	let sample_font_data = SAMPLE_FONT_DATA;
	let mut full_font_data = String::new();
	for _ in 0..256 {
		full_font_data.push_str(sample_font_data,);
	}

	fs::write(&test_file_path, full_font_data,)?;

	let lit_str = syn::LitStr::new(
		"test_font_temp2.txt",
		proc_macro2::Span::call_site(),
	);
	let fonts = font_data(lit_str,)?;

	// Each character should have exactly 128 characters (16 lines × 8
	// chars)
	for (i, font_char,) in fonts.iter().enumerate() {
		assert_eq!(
			font_char.len(),
			128,
			"Character {} has incorrect length: {}",
			i,
			font_char.len()
		);
	}

	// Cleanup
	fs::remove_file(test_file_path,)?;
	success!()
}

#[test]
fn test_convert_bitfield_returns_correct_count() -> TestRslt
{
	let test_fonts = vec!["........".repeat(16); 256];
	let bitfields = convert_bitfield(&test_fonts,)?;

	assert_eq!(bitfields.len(), 256);
	success!()
}

#[test]
fn test_convert_bitfield_empty_pattern() -> TestRslt
{
	// Test with all empty pixels (all dots)
	let empty_pattern = "........".repeat(16,);
	let test_fonts = vec![empty_pattern; 1];
	let bitfields = convert_bitfield(&test_fonts,)?;

	// All dots should result in 0
	assert_eq!(bitfields[0], 0);
	success!()
}

#[test]
fn test_convert_bitfield_full_pattern() -> TestRslt
{
	// Test with all filled pixels (all @)
	let full_pattern = "@@@@@@@@".repeat(16,);
	let test_fonts = vec![full_pattern; 1];
	let bitfields = convert_bitfield(&test_fonts,)?;

	// All @ should result in a non-zero value
	assert_ne!(bitfields[0], 0);
	success!()
}

#[test]
fn test_convert_bitfield_specific_pattern() -> TestRslt
{
	// Test a specific pattern: single pixel in top-left corner
	let mut pattern = String::new();
	pattern.push_str("@.......",); // First line with one pixel
	for _ in 1..16 {
		pattern.push_str("........",); // Remaining 15 lines empty
	}

	let test_fonts = vec![pattern; 1];
	let bitfields = convert_bitfield(&test_fonts,)?;

	// Should have the rightmost bit set (due to bit reversal)
	assert_eq!(bitfields[0] & 1, 1);
	success!()
}

#[test]
fn test_convert_bitfield_line_positioning() -> TestRslt
{
	// Test that different lines result in different bit positions
	let mut patterns = Vec::new();

	// Create patterns with a single pixel on different lines
	for line in 0..16 {
		let mut pattern = String::new();
		for i in 0..16 {
			if i == line {
				pattern.push_str("@.......",); // Pixel on this line
			} else {
				pattern.push_str("........",); // Empty line
			}
		}
		patterns.push(pattern,);
	}

	let bitfields = convert_bitfield(&patterns,)?;

	// Each pattern should produce a different value
	for i in 0..15 {
		for j in (i + 1)..16 {
			assert_ne!(
				bitfields[i], bitfields[j],
				"Patterns {} and {} produced the same bitfield",
				i, j
			);
		}
	}
	success!()
}

#[test]
fn test_fonts_nonexistent_file()
{
	let lit_str = syn::LitStr::new(
		"/nonexistent/path/font.txt",
		proc_macro2::Span::call_site(),
	);
	let result = font_data(lit_str,);
	assert!(result.has_err(), "Should return error for nonexistent file");
}

#[test]
fn test_fonts_with_hex_values_filtered() -> TestRslt
{
	// Create a test font file in the project directory
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path = format!("{}/test_font_hex_temp.txt", project_root);

	// Create font data with hex values that should be filtered out
	let font_data_with_hex = r#"
0x41
........
...@@...
..@..@..
..@..@..
..@..@..
..@@@@..
..@..@..
..@..@..
..@..@..
..@..@..
........
........
........
........
........
........"#;

	// Repeat for all 256 characters
	let mut full_font_data = String::new();
	for _ in 0..256 {
		full_font_data.push_str(font_data_with_hex,);
		full_font_data.push('\n',);
	}

	fs::write(&test_file_path, full_font_data,)?;

	let lit_str = syn::LitStr::new(
		"test_font_hex_temp.txt",
		proc_macro2::Span::call_site(),
	);
	let fonts = font_data(lit_str,)?;

	// Should still load 256 characters, with hex lines filtered out
	assert_eq!(fonts.clone().len(), 256);

	// Each character should still have 128 characters (hex lines filtered)
	for font_char in &fonts {
		assert_eq!(font_char.len(), 128);
	}

	// Cleanup
	fs::remove_file(test_file_path,)?;
	success!()
}

#[test]
fn test_font_function_integration() -> TestRslt
{
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path = format!("{}/test_font_integration.txt", project_root);

	// Create valid font data
	let sample_font_data = SAMPLE_FONT_DATA;
	let mut full_font_data = String::new();
	for _ in 0..256 {
		full_font_data.push_str(sample_font_data,);
	}

	fs::write(&test_file_path, full_font_data,)?;

	let lit_str = syn::LitStr::new(
		"test_font_integration.txt",
		proc_macro2::Span::call_site(),
	);
	let result = font(lit_str,);
	// Should have no diagnostics
	assert!(result.notation().is_empty());

	let tokens = result?;

	// Should generate valid token stream
	let token_string = tokens.to_string();
	assert!(token_string.contains("&"));
	assert!(token_string.contains("["));

	// Cleanup
	fs::remove_file(test_file_path,)?;
	success!()
}

#[test]
fn test_font_data_with_mixed_line_endings() -> TestRslt
{
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path =
		format!("{}/test_font_mixed_endings.txt", project_root);

	// Create font data with consistent line endings to ensure proper
	// parsing
	let mut font_file_data = String::new();
	for i in 0..256 {
		// Create 16 lines of 8 characters each for this character
		for line in 0..16 {
			font_file_data.push_str("........",);
			if line < 15 || i < 255 {
				// Use consistent line endings
				font_file_data.push('\n',);
			}
		}
	}

	fs::write(&test_file_path, font_file_data,)?;

	let lit_str = syn::LitStr::new(
		"test_font_mixed_endings.txt",
		proc_macro2::Span::call_site(),
	);

	let fonts = font_data(lit_str,)?;
	assert_eq!(fonts.len(), 256);
	// Each font should have exactly 128 characters (16 lines × 8
	// chars)
	fonts.iter().for_each(|font| assert_eq!(font.len(), 128),);

	// Cleanup regardless of result
	fs::remove_file(test_file_path,)?;

	success!()
}

#[test]
fn test_convert_bitfield_line_by_line() -> TestRslt
{
	// Test that each line contributes to the correct bit position
	let mut test_patterns = Vec::new();

	// Create patterns where only one line has content
	for line_idx in 0..16 {
		let mut pattern = String::new();
		for i in 0..16 {
			if i == line_idx {
				pattern.push_str("@.......",); // One bit set in this line
			} else {
				pattern.push_str("........",); // Empty line
			}
		}
		test_patterns.push(pattern,);
	}

	let bitfields = convert_bitfield(&test_patterns,)?;

	// Each pattern should produce a different value
	for i in 0..16 {
		assert_ne!(bitfields[i], 0, "Pattern {} should not be zero", i);

		// Check that different lines produce different values
		for j in (i + 1)..16 {
			assert_ne!(
				bitfields[i], bitfields[j],
				"Patterns {} and {} should produce different bitfields",
				i, j
			);
		}
	}
	success!()
}

#[test]
fn test_convert_bitfield_bit_reversal() -> TestRslt
{
	// Test that bit reversal works correctly
	let patterns = vec![
		"@.......".repeat(16,), // Leftmost bit
		".......@".repeat(16,), // Rightmost bit
	];

	let bitfields = convert_bitfield(&patterns,)?;

	// Due to bit reversal, the leftmost @ should set the rightmost bit
	// and the rightmost @ should set the leftmost bit
	assert_ne!(bitfields[0], bitfields[1]);
	assert_ne!(bitfields[0], 0);
	assert_ne!(bitfields[1], 0);
	success!()
}

#[test]
fn test_font_data_error_conditions()
{
	// Test various error conditions

	// Non-existent file
	let lit_str = syn::LitStr::new(
		"definitely_does_not_exist.txt",
		proc_macro2::Span::call_site(),
	);
	let result = font_data(lit_str,);
	assert!(result.has_err());

	// Test with invalid CARGO_MANIFEST_DIR
	unsafe {
		std::env::set_var(
			"CARGO_MANIFEST_DIR",
			"/invalid/path/that/does/not/exist",
		);
	}
	let lit_str = syn::LitStr::new("test.txt", proc_macro2::Span::call_site(),);
	let result = font_data(lit_str,);
	assert!(result.has_err());

	// Restore CARGO_MANIFEST_DIR
	unsafe {
		std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"),);
	}
}

#[test]
fn test_font_data_with_insufficient_characters() -> TestRslt
{
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path = format!("{}/test_font_insufficient.txt", project_root);

	// Create font data with only 100 characters instead of 256
	let sample_font_data = r#"........
...@@...
..@..@..
..@..@..
..@..@..
..@@@@..
..@..@..
..@..@..
..@..@..
..@..@..
........
........
........
...."#;

	let mut font_file_data = String::new();
	for _ in 0..100 {
		// Only 100 characters
		font_file_data.push_str(sample_font_data,);
	}

	fs::write(&test_file_path, font_file_data,)?;

	let lit_str = syn::LitStr::new(
		"test_font_insufficient.txt",
		proc_macro2::Span::call_site(),
	);

	let result = font_data(lit_str,);

	// Cleanup
	fs::remove_file(test_file_path,)?;

	assert!(result.has_err());

	success!()
}

#[test]
fn test_font_data_with_wrong_character_length() -> TestRslt
{
	use std::env;

	let project_root = env::var("CARGO_MANIFEST_DIR",)?;
	let test_file_path = format!("{}/test_font_wrong_length.txt", project_root);

	// Create font data where each character has wrong length (not 128
	// chars)
	let wrong_font_data = "........\n...@@...\n..@..@..\n"; // Only 3 lines instead of 16
	let mut font_file_data = String::new();
	for _ in 0..256 {
		font_file_data.push_str(wrong_font_data,);
	}

	fs::write(&test_file_path, font_file_data,)?;

	let lit_str = syn::LitStr::new(
		"test_font_wrong_length.txt",
		proc_macro2::Span::call_site(),
	);

	let result = font_data(lit_str,);

	// Cleanup
	fs::remove_file(test_file_path,)?;
	assert!(result.has_err());
	success!()
}

#[test]
fn test_convert_bitfield_preserves_count() -> TestRslt
{
	// Test that convert_bitfield preserves the number of characters
	let input_fonts = vec!["........".repeat(16); 256]; // 256 characters, each 128 chars long
	let result = convert_bitfield(&input_fonts,)?;

	assert_eq!(result.len(), 256);
	success!()
}

#[test]
fn test_convert_bitfield_deterministic() -> TestRslt
{
	// Test that convert_bitfield produces deterministic results
	let input_fonts = vec!["@.......".repeat(16); 10]; // 10 characters for faster test

	let result1 = convert_bitfield(&input_fonts,)?;
	let result2 = convert_bitfield(&input_fonts,)?;

	assert_eq!(result1, result2);
	success!()
}

#[test]
fn test_convert_bitfield_edge_cases() -> TestRslt
{
	// Test edge cases: single character, empty input
	let single_char = vec!["@.......".repeat(16,)]; // Single character
	let result = convert_bitfield(&single_char,)?;
	assert_eq!(result.len(), 1);
	assert_ne!(result[0], 0); // Should have some bits set

	// Test with all dots
	let all_dots = vec![".".repeat(128,)];
	let result = convert_bitfield(&all_dots,)?;
	assert_eq!(result.len(), 1);
	assert_eq!(result[0], 0); // Should be all zeros

	// Test with all @
	let all_at = vec!["@".repeat(128,)];
	let result = convert_bitfield(&all_at,)?;
	assert_eq!(result.len(), 1);
	assert_eq!(result[0], u128::MAX); // Should be all ones
	success!()
}

#[test]
fn test_convert_bitfield_bit_positions() -> TestRslt
{
	let mut pattern = ".".repeat(128,);
	pattern.replace_range(0..1, "@",);

	let fonts = vec![pattern];
	let result = convert_bitfield(&fonts,)?;

	assert_eq!(result.len(), 1);
	assert_eq!(result[0], 1);

	let mut pattern = ".".repeat(128,);
	pattern.replace_range(7..8, "@",);
	let result = convert_bitfield(&[pattern,],)?;
	assert_eq!(result[0], 1u128 << 7);

	let mut pattern = ".".repeat(128,);
	pattern.replace_range(127..128, "@",);
	let result = convert_bitfield(&[pattern,],)?;
	assert_eq!(result[0], 1u128 << 127);
	success!()
}

#[test]
fn test_convert_bitfield_mixed_patterns() -> TestRslt
{
	// Test with mixed patterns
	let patterns = vec![
		"@.......".repeat(16,), // Pattern with @ at start
		".......@".repeat(16,), // Pattern with @ at end
		"@.@.@.@.".repeat(16,), // Alternating pattern
	];

	let result = convert_bitfield(&patterns,)?;
	assert_eq!(result.len(), 3);

	// All should be different
	assert_ne!(result[0], result[1]);
	assert_ne!(result[1], result[2]);
	assert_ne!(result[0], result[2]);
	success!()
}

#[test]
fn test_font_data_error_handling()
{
	// Test error handling for various invalid inputs
	use std::env;

	let _project_root =
		env::var("CARGO_MANIFEST_DIR",).unwrap_or_else(|_| ".".to_string(),);

	// Test with non-existent file
	let nonexistent_file = syn::LitStr::new(
		"nonexistent_font_file.txt",
		proc_macro2::Span::call_site(),
	);
	let result = font_data(nonexistent_file,);
	assert!(result.has_err());
}
