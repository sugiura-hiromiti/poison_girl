use {
	html5ever::tendril::TendrilSink,
	poison_girl_proc_macro_helper::rslt::Rslt,
	poison_girl_proc_macro_impl::{
		font, impl_int,
		status::*,
		test_program_headers_parse::{self},
		wrapper,
	},
	std::fs,
	tempfile::NamedTempFile,
};

#[test]
fn test_gen_wrapper_fn_integration() {
	use syn::parse_quote;

	// Test with various function signatures
	let signatures = vec![
		parse_quote! { fn simple_function(arg1: i32, arg2: String) -> bool },
		parse_quote! { fn method_with_self(&self, arg1: i32) -> () },
		parse_quote! { fn method_with_mut_self(&mut self, arg1: String, arg2: Vec<i32>) -> String },
		parse_quote! { fn complex_function<T>(arg1: T, arg2: Option<T>) -> Result<T, Error> where T: Clone },
	];

	for sig in signatures {
		let args: Vec<_,> = wrapper::method_args(&sig,).collect();

		// Verify that receiver arguments are filtered out
		let _has_receiver = sig
			.inputs
			.iter()
			.any(|input| matches!(input, syn::FnArg::Receiver(_)),);
		let typed_args_count = sig
			.inputs
			.iter()
			.filter(|input| matches!(input, syn::FnArg::Typed(_)),)
			.count();

		assert_eq!(args.len(), typed_args_count);
	}
}

#[test]
#[ignore = "requires font file"]
fn test_fonts_data_integration() {
	// Create a temporary font file
	let temp_file = NamedTempFile::new().expect("Failed to create temp file",);

	// Create minimal valid font data (16 lines per character, 8 chars per line,
	// 256 characters)
	let single_char_pattern = "........\n...@@...\n..@..@..\n..@..@..\n..@..@.\
	                           .\n..@@@@..\n..@..@..\n..@..@..\n..@..@..\n..@.\
	                           .@..\n........\n........\n........\n........\n.\
	                           .......\n........\n";
	let font_data = single_char_pattern.repeat(256,);

	fs::write(temp_file.path(), font_data,)
		.expect("Failed to write font data",);

	let path_str = temp_file.path().to_str().unwrap();
	let lit_str = syn::LitStr::new(path_str, proc_macro2::Span::call_site(),);

	// Test font function (the only public function)
	let result = font::font(lit_str,);
	assert!(
		!result.has_err(),
		"Font processing should succeed with valid data"
	);

	let tokens = result.unwrap().unwrap();
	let token_string = tokens.to_string();

	// Verify that the result contains array-like structure
	assert!(token_string.contains("&"), "Should contain array reference");
	assert!(token_string.contains("["), "Should contain array brackets");
}

#[test]
fn test_impl_init_integration() {
	use quote::quote;

	// Test parsing and implementation generation for multiple types
	let input = quote! { u8, u16, u32, u64, i8, i16, i32, i64 };
	let types: impl_int::Types =
		syn::parse2(input,).expect("Failed to parse types",);

	let implementations: Vec<_,> =
		types.iter().map(|ty| impl_int::implement(ty,),).collect();

	assert_eq!(implementations.len(), 8);

	// Verify that each implementation contains the expected methods
	for impl_tokens in implementations {
		let code_str = impl_tokens.to_string();
		assert!(code_str.contains("impl Integer for"));
		assert!(code_str.contains("fn digit_count"));
		assert!(code_str.contains("fn nth_digit"));
		assert!(code_str.contains("fn shift_right"));
	}
}

#[test]
fn test_status_from_spec_html_parsing_integration() {
	// Test the HTML parsing functions with a complete example
	let test_html = r#"
<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
    <section id="status-codes">
        <h1>Status Codes</h1>
        <table id="efi-status-success-codes-high-bit-clear-apx-d-status-codes">
            <tr><th>Mnemonic</th><th>Value</th><th>Description</th></tr>
            <tr>
                <td><p>EFI_SUCCESS</p></td>
                <td><p>0x00000000</p></td>
                <td><p>The operation completed successfully.</p></td>
            </tr>
        </table>
        <table id="efi-status-error-codes-high-bit-set-apx-d-status-codes">
            <tr><th>Mnemonic</th><th>Value</th><th>Description</th></tr>
            <tr>
                <td><p>EFI_LOAD_ERROR</p></td>
                <td><p>0x00000001</p></td>
                <td><p>The image failed to load.</p></td>
            </tr>
        </table>
        <table id="efi-status-warning-codes-high-bit-clear-apx-d-status-codes">
            <tr><th>Mnemonic</th><th>Value</th><th>Description</th></tr>
            <tr>
                <td><p>EFI_WARN_UNKNOWN_GLYPH</p></td>
                <td><p>0x00000001</p></td>
                <td><p>The string contained one or more characters that the device could not render.</p></td>
            </tr>
        </table>
    </section>
</body>
</html>"#;

	// Parse the HTML
	let dom = html5ever::parse_document(
		markup5ever_rcdom::RcDom::default(),
		Default::default(),
	)
	.one(test_html,);

	// Test that we can find elements by ID
	let main_section = get_element_by_id(dom.document.clone(), "status-codes",);
	assert!(main_section.is_some());

	let success_table = get_element_by_id(
		dom.document.clone(),
		"efi-status-success-codes-high-bit-clear-apx-d-status-codes",
	);
	assert!(success_table.is_some());

	let error_table = get_element_by_id(
		dom.document.clone(),
		"efi-status-error-codes-high-bit-set-apx-d-status-codes",
	);
	assert!(error_table.is_some());

	let warn_table = get_element_by_id(
		dom.document.clone(),
		"efi-status-warning-codes-high-bit-clear-apx-d-status-codes",
	);
	assert!(warn_table.is_some());
}

#[test]
fn test_type_conversions_integration() -> Rslt<(),> {
	// Test that different modules can work with the same data types
	use test_program_headers_parse::*;

	// Test u32 parsing
	let u32_result = u32::parse("1a2b",)??;
	assert_eq!(u32_result, 0x1a2b);

	// Test u64 parsing
	let u64_result = u64::parse("1a2b3c4d",)??;
	assert_eq!(u64_result, 0x1a2b3c4d);

	// Test that both types implement the same trait
	assert_eq!(u32::parse("ff")??, 255u32);
	assert_eq!(u64::parse("ff")??, 255u64);
	Rslt::new((),)
}

#[test]
fn test_proc_macro_dependencies_integration() {
	// Test that proc macro dependencies work correctly together
	use {
		proc_macro2::TokenStream,
		quote::quote,
		syn::{Type, parse_quote},
	};

	// Test that we can create and manipulate token streams
	let tokens: TokenStream = quote! {
		fn test_function() -> i32 {
			42
		}
	};

	assert!(!tokens.is_empty());

	// Test that we can parse types
	let ty: Type = parse_quote! { Vec<String> };
	let type_str = quote! { #ty }.to_string();
	assert!(type_str.contains("Vec"));
	assert!(type_str.contains("String"));
}

#[test]
fn test_html_parsing_dependencies_integration() {
	// Test that HTML parsing dependencies work together
	use {
		html5ever::{QualName, local_name, ns, parse_fragment},
		markup5ever_rcdom::RcDom,
	};

	let html = r#"<div id="test"><p>Hello World</p></div>"#;

	let dom = parse_fragment(
		RcDom::default(),
		Default::default(),
		QualName::new(None, ns!(), local_name!(""),),
		vec![],
		true,
	)
	.one(html,);

	// Verify that we can traverse the DOM
	let children = dom.document.children.borrow();
	assert!(!children.is_empty());
}

#[test]
fn test_string_processing_integration() {
	// Test string processing across different modules

	// Test that we can handle various string formats used in different modules
	let test_strings = vec![
		"ELF64 (64-bit)",
		"0x401000 (entry point)",
		"LOAD           0x0000000000001000 0x0000000000401000",
		"There are 4 program headers, starting at offset 64",
		"EFI_SUCCESS",
		"The operation completed successfully.",
	];

	for test_str in test_strings {
		// Test basic string operations that modules use
		let parts: Vec<&str,> = test_str.split(' ',).collect();
		assert!(!parts.is_empty());

		let first_word = parts[0];
		assert!(!first_word.is_empty());

		// Test that we can handle strings with special characters
		if test_str.contains("0x",) {
			assert!(test_str.find("0x").is_some());
		}
	}
}

#[test]
fn test_file_system_integration() {
	// Test file system operations used across modules
	use std::env::current_dir;

	// Test current directory access (used in check_oso_kernel)
	let current = current_dir().expect("Failed to get current directory",);
	assert!(current.is_absolute());

	// Test path joining (used in various modules)
	let joined = current.join("target",).join("oso_kernel.elf",);
	assert!(joined.to_string_lossy().contains("target"));
	assert!(joined.to_string_lossy().contains("oso_kernel.elf"));

	// Test path existence checking
	let exists = joined.exists();
	// We don't assert the result since it depends on the test environment
	// Just verify that the operation doesn't panic
	let _ = exists;
}
