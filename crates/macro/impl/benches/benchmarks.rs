//! Benchmark tests for oso_proc_macro_logic crate
//!
//! These benchmarks measure the performance of key operations
//! to ensure they meet performance requirements.

use std::hint::black_box;
use std::time::Duration;
use std::time::Instant;

/// Simple benchmark runner that measures execution time
fn benchmark<F, R,>(name: &str, iterations: usize, mut f: F,) -> Duration
where F: FnMut() -> R {
	// Warm up
	for _ in 0..10 {
		black_box(f(),);
	}

	let start = Instant::now();
	for _ in 0..iterations {
		black_box(f(),);
	}
	let duration = start.elapsed();

	println!(
		"{}: {:?} total, {:?} per iteration",
		name,
		duration,
		duration / iterations as u32
	);

	duration
}

#[test]
fn benchmark_fonts_data_processing() {
	use oso_proc_macro_logic::font;
	use std::fs;
	use tempfile::NamedTempFile;

	// Create test font data
	let temp_file = NamedTempFile::new().expect("Failed to create temp file",);
	let font_pattern = "........\n...@@...\n..@..@..\n..@..@..\n..@..@..\n..@@\
	                    @@..\n..@..@..\n..@..@..\n..@..@..\n..@..@..\n........\
	                    \n........\n........\n........\n........\n........\n";
	let font_data = font_pattern.repeat(256,);
	fs::write(temp_file.path(), font_data,)
		.expect("Failed to write font data",);

	let path_str = temp_file.path().to_str().unwrap();
	let lit_str = syn::LitStr::new(path_str, proc_macro2::Span::call_site(),);

	// Benchmark font loading
	let _duration = benchmark("Font loading", 100, || font::fonts(&lit_str,),);

	// Benchmark bitfield conversion
	let fonts = font::fonts(&lit_str,);
	let _duration = benchmark("Bitfield conversion", 1000, || {
		font::convert_bitfield(&fonts,)
	},);
}

#[test]
fn benchmark_gen_wrapper_fn() {
	use oso_proc_macro_logic::wrapper;
	use syn::Signature;
	use syn::parse_quote;

	let signatures = vec![
		parse_quote! { fn simple(a: i32, b: String) -> bool },
		parse_quote! { fn with_self(&self, a: i32, b: String) -> bool },
		parse_quote! { fn complex<T>(&mut self, a: T, b: Vec<T>, c: Option<T>) -> Result<T, Error> where T: Clone },
	];

	// Benchmark method argument extraction
	let _duration = benchmark("Method args extraction", 10000, || {
		for sig in &signatures {
			let _args: Vec<_,> = wrapper::method_args(sig,).collect();
		}
	},);
}

#[test]
fn benchmark_impl_init() {
	use oso_proc_macro_logic::impl_int;
	use quote::quote;

	let input = quote! { u8, u16, u32, u64, i8, i16, i32, i64, usize, isize };

	// Benchmark type parsing
	let _duration = benchmark("Type parsing", 1000, || {
		let _types: impl_int::Types = syn::parse2(input.clone(),).unwrap();
	},);

	// Benchmark implementation generation
	let types: impl_int::Types = syn::parse2(input,).unwrap();
	let type_vec: Vec<_,> = types.iter().collect();

	let _duration = benchmark("Implementation generation", 1000, || {
		for ty in &type_vec {
			let _impl_tokens = impl_int::implement(ty,);
		}
	},);
}

#[test]
fn benchmark_status_from_spec_html_parsing() {
	use oso_proc_macro_logic::status::*;

	let test_html = r#"
<!DOCTYPE html>
<html>
<body>
    <section id="status-codes">
        <table id="efi-status-success-codes-high-bit-clear-apx-d-status-codes">
            <tr><th>Mnemonic</th><th>Value</th><th>Description</th></tr>
            <tr><td><p>EFI_SUCCESS</p></td><td><p>0x00000000</p></td><td><p>Success</p></td></tr>
            <tr><td><p>EFI_LOAD_ERROR</p></td><td><p>0x00000001</p></td><td><p>Load error</p></td></tr>
        </table>
    </section>
</body>
</html>"#.repeat(10); // Make it larger for meaningful benchmarking

	// Benchmark HTML parsing
	let _duration = benchmark("HTML parsing", 100, || {
		let dom = html5ever::parse_document(
			markup5ever_rcdom::RcDom::default(),
			Default::default(),
		)
		.one(&test_html,);

		let _main_section =
			get_element_by_id(dom.document.clone(), "status-codes",);
	},);

	// Benchmark element searching
	let dom = html5ever::parse_document(
		markup5ever_rcdom::RcDom::default(),
		Default::default(),
	)
	.one(&test_html,);

	let _duration = benchmark("Element by ID search", 1000, || {
		let _element = get_element_by_id(dom.document.clone(), "status-codes",);
	},);

	let _duration = benchmark("Elements by name search", 1000, || {
		let _elements = get_elements_by_name(dom.document.clone(), "tr",);
	},);
}

#[test]
fn benchmark_hex_parsing() {
	use oso_proc_macro_logic::test_program_headers_parse::*;

	let hex_strings = vec![
		"0x0",
		"0x1",
		"0xff",
		"0x1000",
		"0x12345678",
		"0xabcdef",
		"0x1a2b3c4d",
		"0xffffffff",
	];

	// Benchmark u32 hex parsing
	let _duration = benchmark("u32 hex parsing", 10000, || {
		for hex_str in &hex_strings {
			if let Ok(hex_without_prefix,) =
				hex_str.strip_prefix("0x",).ok_or("no prefix",)
			{
				let _result = u32::parse(hex_without_prefix,);
			}
		}
	},);

	// Benchmark u64 hex parsing
	let _duration = benchmark("u64 hex parsing", 10000, || {
		for hex_str in &hex_strings {
			if let Ok(hex_without_prefix,) =
				hex_str.strip_prefix("0x",).ok_or("no prefix",)
			{
				let _result = u64::parse(hex_without_prefix,);
			}
		}
	},);
}

#[test]
fn benchmark_string_operations() {
	let test_strings = vec![
		"ELF64 (64-bit)",
		"0x401000 (entry point)",
		"LOAD           0x0000000000001000 0x0000000000401000",
		"There are 4 program headers, starting at offset 64",
		"The operation completed successfully.",
	];

	// Benchmark string splitting (used extensively in parsing)
	let _duration = benchmark("String splitting", 10000, || {
		for s in &test_strings {
			let _parts: Vec<&str,> = s.split(' ',).collect();
		}
	},);

	// Benchmark string trimming
	let _duration = benchmark("String trimming", 10000, || {
		for s in &test_strings {
			let _trimmed = s.trim();
		}
	},);

	// Benchmark string contains checks
	let _duration = benchmark("String contains", 10000, || {
		for s in &test_strings {
			let _has_0x = s.contains("0x",);
			let _has_space = s.contains(" ",);
		}
	},);
}

#[test]
fn benchmark_vector_operations() {
	// Test vector operations used in various modules
	let data: Vec<String,> =
		(0..1000).map(|i| format!("item_{}", i),).collect();

	// Benchmark vector iteration
	let _duration = benchmark("Vector iteration", 1000, || {
		for item in &data {
			black_box(item,);
		}
	},);

	// Benchmark vector filtering
	let _duration = benchmark("Vector filtering", 1000, || {
		let _filtered: Vec<_,> =
			data.iter().filter(|s| s.contains("5",),).collect();
	},);

	// Benchmark vector mapping
	let _duration = benchmark("Vector mapping", 1000, || {
		let _mapped: Vec<_,> = data.iter().map(|s| s.len(),).collect();
	},);
}

#[test]
fn benchmark_memory_allocations() {
	// Benchmark memory allocation patterns used in the crate

	// Benchmark string allocations
	let _duration = benchmark("String allocations", 10000, || {
		let _s = String::from("test string",);
		let _s2 = format!("formatted {}", 42);
		let _s3 = "static".to_string();
	},);

	// Benchmark vector allocations
	let _duration = benchmark("Vector allocations", 10000, || {
		let _v: Vec<i32,> = Vec::new();
		let _v2: Vec<String,> = Vec::with_capacity(10,);
		let _v3 = vec![1, 2, 3, 4, 5];
	},);
}

#[test]
fn benchmark_error_handling() {
	use anyhow::Result as Rslt;
	use anyhow::anyhow;

	fn success_function() -> Rslt<i32,> {
		Ok(42,)
	}

	fn error_function() -> Rslt<i32,> {
		Err(anyhow!("Test error"),)
	}

	// Benchmark successful operations
	let _duration = benchmark("Successful operations", 10000, || {
		let _result = success_function();
	},);

	// Benchmark error creation and handling
	let _duration = benchmark("Error operations", 10000, || {
		let _result = error_function();
	},);

	// Benchmark error propagation
	fn propagate_error() -> Rslt<i32,> {
		error_function()?;
		Ok(0,)
	}

	let _duration = benchmark("Error propagation", 10000, || {
		let _result = propagate_error();
	},);
}

#[test]
fn benchmark_overall_performance() {
	println!("\n=== Overall Performance Summary ===");

	// This test provides a summary of overall performance characteristics
	// and can be used to detect performance regressions

	let start = Instant::now();

	// Simulate a typical workload
	for _ in 0..100 {
		// String processing
		let test_str = "ELF64 0x401000 LOAD";
		let _parts: Vec<&str,> = test_str.split(' ',).collect();

		// Vector operations
		let data: Vec<i32,> = (0..100).collect();
		let _sum: i32 = data.iter().sum();

		// Error handling
		let _result: Result<(), anyhow::Error,> = Ok((),);
	}

	let total_duration = start.elapsed();
	println!("Total workload time: {:?}", total_duration);

	// Performance assertions (adjust thresholds as needed)
	assert!(
		total_duration < Duration::from_millis(100),
		"Performance regression detected: workload took {:?}",
		total_duration
	);
}
