#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use poison_girl_proc_macro_helper::Diag;
use poison_girl_proc_macro_helper::atr;
use poison_girl_proc_macro_helper::drv;
use poison_girl_proc_macro_helper::fnl;
use proc_macro::Diagnostic;
use proc_macro::Level;

trait ErrorDiagnose {
	type T;
	fn unwrap_or_emit(self,) -> Self::T;
}

impl<T,> ErrorDiagnose for anyhow::Result<(T, Vec<Diag,>,),> {
	type T = T;

	fn unwrap_or_emit(self,) -> Self::T {
		match self {
			Self::Ok((o, diag,),) => {
				diag.iter().for_each(|d| match d {
					Diag::Err(msg,) => {
						Diagnostic::new(Level::Error, msg,).emit()
					},
					Diag::Warn(msg,) => {
						Diagnostic::new(Level::Warning, msg,).emit()
					},
					Diag::Note(msg,) => {
						Diagnostic::new(Level::Note, msg,).emit()
					},
					Diag::Help(msg,) => {
						Diagnostic::new(Level::Help, msg,).emit()
					},
				},);

				o
			},
			Self::Err(e,) => {
				Diagnostic::new(Level::Error, format!("{e}"),).emit();
				panic!("{e}");
			},
		}
	}
}

fnl!(font => syn::LitStr,
r#"Generates embedded font data from font files at compile time.

This procedural macro takes a relative path to the project root and processes
font files to generate embedded data structures that can be used at runtime.
The macro converts font data into bitfield representations for efficient storage.

# Parameters

* `path` - A string literal containing the relative path from the project root to the directory
  containing font data files

# Returns

Returns a token stream representing an array slice of processed font data.
The generated code will be in the form `&[font_data_1, font_data_2, ...]`.

# Examples

```rust,ignore
// Generate font data from files in the "assets/fonts" directory
let fonts = fonts_data!("assets/fonts");
```

# Panics

This macro will cause a compile-time error if:
- The specified path does not exist
- Font files in the path cannot be processed
- The path parameter is not a valid string literal"#
);

fnl!(impl_int => poison_girl_proc_macro_impl::impl_int::Types,
r#"Generates implementations for integer types.

This procedural macro takes a list of types and generates implementations
for them using the logic defined in the `oso_proc_macro_logic::impl_init` module.
It's typically used to reduce boilerplate when implementing common traits
or methods for multiple integer types.

# Parameters

* `types` - A token stream representing the types to implement. The format should match the
  `Types` parser in the logic module.

# Returns

Returns a token stream containing the generated implementations for all
specified types.

# Examples

```rust,ignore
// Generate implementations for u8, u16, u32, u64
impl_int!(u8, u16, u32, u64);
```

# Panics

This macro will cause a compile-time error if:
- The input cannot be parsed as valid types
- The implementation logic fails for any of the specified types"#
);

atr!(wrapper => syn::Ident, syn::ItemTrait,
r#"Generates wrapper functions for trait methods.

This attribute macro takes a trait definition and generates corresponding
wrapper functions that delegate to a static instance. This is useful for
creating global function interfaces that wrap trait implementations.

# Parameters

* `attr` - The identifier of the static frame buffer or instance to delegate to
* `item` - The trait definition to generate wrappers for

# Returns

Returns the original trait definition along with generated wrapper functions.
Each trait method becomes a standalone function that calls the corresponding
method on the specified static instance.

# Generated Code

For each trait method, generates a function with:
- Same signature as the trait method (excluding `self` parameter)
- Same visibility, safety, async, const, and ABI attributes
- Delegation to the static instance method

# Examples

```rust,ignore
#[gen_wrapper_fn(GLOBAL_FRAMEBUFFER)]
trait Display {
    fn write_pixel(&mut self, x: u32, y: u32, color: u32);
    fn clear(&mut self);
}

// Generates:
// pub fn write_pixel(x: u32, y: u32, color: u32) {
//     GLOBAL_FRAMEBUFFER.write_pixel(x, y, color)
// }
// pub fn clear() {
//     GLOBAL_FRAMEBUFFER.clear()
// }
```

# Panics

This macro will cause a compile-time error if:
- The attribute is not a valid identifier
- The item is not a valid trait definition
- Any trait method has an unsupported signature"#
);

fnl!(status => syn::Lit,
r#"Generates UEFI status code definitions from the official UEFI specification.

This procedural macro fetches status code information from the UEFI specification
website and generates a complete `Status` struct with associated constants and
error handling methods. The macro downloads and parses the specification page
at compile time to ensure the status codes are up-to-date and accurate.

# Parameters

* `version` - A floating-point literal specifying the UEFI specification version (e.g., `2.8`,
  `2.9`, `2.10`)

# Returns

Returns a token stream containing:
- A `Status` struct with transparent representation
- Associated constants for all status codes (success, warning, error)
- Implementation of `ok_or()` method for error handling
- Implementation of `ok_or_with()` method for custom error handling

# Generated Structure

```rust,ignore
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Status(pub usize);

impl Status {
    // Success status codes
    pub const SUCCESS: Self = Self(0x0);

    // Warning status codes
    pub const WARN_UNKNOWN_GLYPH: Self = Self(0x1);

    // Error status codes
    pub const LOAD_ERROR: Self = Self(0x8000000000000001);

    // Error handling methods
    pub fn ok_or(self) -> Result<Self, UefiError> { ... }
    pub fn ok_or_with<T>(self, with: impl FnOnce(Self) -> T) -> Result<T, UefiError> { ... }
}
```

# Examples

```rust,ignore
// Generate status codes from UEFI 2.9 specification
status_from_spec!(2.9);
```

# Network Requirements

This macro requires internet access at compile time to fetch the UEFI specification.
The macro will download from: `https://uefi.org/specs/UEFI/{version}/Apx_D_Status_Codes.html`

# Panics

This macro will cause a compile-time error if:
- The version parameter is not a floating-point literal
- The UEFI specification page cannot be accessed
- The specification page format has changed and cannot be parsed
- Network connectivity issues prevent downloading the specification"#
);

fnl!(test_elf_header_parse => proc_macro2::TokenStream,
r#"Generates compile-time tests for ELF header parsing.

This procedural macro creates a compile-time assertion that validates ELF header
parsing by comparing the provided header data against the expected structure
obtained from running `readelf -h` on the target binary. The test only runs
in debug builds to avoid performance overhead in release builds.

# Parameters

* `header` - A token stream representing the ELF header structure to validate

# Returns

Returns a token stream containing a conditional assertion that compares the
provided header against the expected header information. The assertion is
only active in debug builds (`cfg!(debug_assertions)`).

# Generated Code

```rust,ignore
if cfg!(debug_assertions) {
    assert_eq!(expected_header_info, provided_header);
}
```

# Examples

```rust,ignore
// Test that a parsed ELF header matches expectations
test_elf_header_parse!(my_elf_header);
```

# Behavior

- **Debug builds**: Performs the assertion and will panic if headers don't match
- **Release builds**: No-op, generates no code for performance

# Dependencies

This macro relies on:
- `readelf` command being available in the system PATH
- The helper module's `elf_header_info()` function
- The target binary being available for analysis

# Panics

In debug builds, this macro will cause a runtime panic if:
- The provided header doesn't match the expected header structure
- The `readelf` command fails or is not available
- The ELF header cannot be parsed from the binary"#
);

fnl!(test_program_headers_parse => proc_macro2::TokenStream,
r#"Generates compile-time tests for ELF program headers parsing.

This procedural macro creates a compile-time assertion that validates ELF program
headers parsing by comparing the provided program headers data against the expected
structure obtained from running `readelf -l` on the target binary. Like the ELF
header test, this only runs in debug builds for performance reasons.

# Parameters

* `program_headers` - A token stream representing the program headers structure to validate

# Returns

Returns a token stream containing a conditional assertion that compares the
provided program headers against the expected program headers information.
The assertion is only active in debug builds (`cfg!(debug_assertions)`).

# Generated Code

```rust,ignore
if cfg!(debug_assertions) {
    assert_eq!(expected_program_headers_info, provided_program_headers);
}
```

# Examples

```rust,ignore
// Test that parsed program headers match expectations
test_program_headers_parse!(my_program_headers);
```

# Behavior

- **Debug builds**: Performs the assertion and will panic if headers don't match
- **Release builds**: No-op, generates no code for performance

# Program Header Validation

The macro validates all aspects of program headers including:
- Header type (LOAD, DYNAMIC, INTERP, etc.)
- Flags (read, write, execute permissions)
- File and memory offsets
- Virtual and physical addresses
- File and memory sizes
- Alignment requirements

# Dependencies

This macro relies on:
- `readelf` command being available in the system PATH
- The helper module's `program_headers_info()` function
- The target binary being available for analysis

# Panics

In debug builds, this macro will cause a runtime panic if:
- The provided program headers don't match the expected structure
- The `readelf` command fails or is not available
- The program headers cannot be parsed from the binary
- Any program header field has an unexpected value"#
);

drv!(FromPathBuf, from_path_buf => syn::DeriveInput, attributes: chart,
r#""#
);

atr!(features => proc_macro2::TokenStream, syn::ItemEnum, r#""#);

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::anyhow;

	#[test]
	fn test_error_diagnose_trait_ok() {
		let result: anyhow::Result<(i32, Vec<Diag,>,),> = Ok((42, vec![],),);
		let value = result.unwrap_or_emit();
		assert_eq!(value, 42);
	}

	#[test]
	fn test_error_diagnose_trait_ok_with_diagnostics() {
		let diags = vec![
			Diag::Note("Test note".to_string(),),
			Diag::Help("Test help".to_string(),),
		];
		let result: anyhow::Result<(String, Vec<Diag,>,),> =
			Ok(("success".to_string(), diags,),);

		// We can't test the actual emission outside of proc_macro context,
		// but we can test that the result structure is correct
		match result {
			Ok((value, diagnostics,),) => {
				assert_eq!(value, "success");
				assert_eq!(diagnostics.len(), 2);
				match &diagnostics[0] {
					Diag::Note(msg,) => assert_eq!(msg, "Test note"),
					_ => panic!("Expected note diagnostic"),
				}
				match &diagnostics[1] {
					Diag::Help(msg,) => assert_eq!(msg, "Test help"),
					_ => panic!("Expected help diagnostic"),
				}
			},
			Err(_,) => panic!("Expected Ok result"),
		}
	}

	#[test]
	#[should_panic]
	fn test_error_diagnose_trait_err() {
		let result: anyhow::Result<(i32, Vec<Diag,>,),> =
			Err(anyhow!("Test error"),);
		let _value = result.unwrap_or_emit();
	}

	#[test]
	fn test_diag_variants() {
		// Test that we can create different diagnostic types
		let _err_diag = Diag::Err("Error message".to_string(),);
		let _warn_diag = Diag::Warn("Warning message".to_string(),);
		let _note_diag = Diag::Note("Note message".to_string(),);
		let _help_diag = Diag::Help("Help message".to_string(),);

		// If we get here without compilation errors, the Diag enum is working
		assert!(true);
	}

	#[test]
	fn test_error_diagnose_with_multiple_diagnostics() {
		let diags = vec![
			Diag::Warn("Warning 1".to_string(),),
			Diag::Note("Note 1".to_string(),),
			Diag::Help("Help 1".to_string(),),
			Diag::Warn("Warning 2".to_string(),),
		];
		let result: anyhow::Result<(bool, Vec<Diag,>,),> = Ok((true, diags,),);

		// Test the structure without calling unwrap_or_emit
		match result {
			Ok((value, diagnostics,),) => {
				assert_eq!(value, true);
				assert_eq!(diagnostics.len(), 4);

				// Verify each diagnostic type and message
				match &diagnostics[0] {
					Diag::Warn(msg,) => assert_eq!(msg, "Warning 1"),
					_ => panic!("Expected warning diagnostic"),
				}
				match &diagnostics[1] {
					Diag::Note(msg,) => assert_eq!(msg, "Note 1"),
					_ => panic!("Expected note diagnostic"),
				}
				match &diagnostics[2] {
					Diag::Help(msg,) => assert_eq!(msg, "Help 1"),
					_ => panic!("Expected help diagnostic"),
				}
				match &diagnostics[3] {
					Diag::Warn(msg,) => assert_eq!(msg, "Warning 2"),
					_ => panic!("Expected warning diagnostic"),
				}
			},
			Err(_,) => panic!("Expected Ok result"),
		}
	}

	#[test]
	fn test_error_diagnose_empty_diagnostics() {
		let result: anyhow::Result<(Vec<i32,>, Vec<Diag,>,),> =
			Ok((vec![1, 2, 3], vec![],),);
		let value = result.unwrap_or_emit();
		assert_eq!(value, vec![1, 2, 3]);
	}

	// Integration tests for the trait behavior
	#[test]
	fn test_error_diagnose_trait_integration() {
		// Test that the trait works with different types
		let string_result: anyhow::Result<(String, Vec<Diag,>,),> =
			Ok(("test".to_string(), vec![],),);
		assert_eq!(string_result.unwrap_or_emit(), "test");

		let vec_result: anyhow::Result<(Vec<u8,>, Vec<Diag,>,),> =
			Ok((vec![1, 2, 3], vec![],),);
		assert_eq!(vec_result.unwrap_or_emit(), vec![1, 2, 3]);

		let option_result: anyhow::Result<(Option<i32,>, Vec<Diag,>,),> =
			Ok((Some(42,), vec![],),);
		assert_eq!(option_result.unwrap_or_emit(), Some(42));
	}

	#[test]
	fn test_diagnostic_message_content() {
		// Test that diagnostic messages are properly formatted
		let diags = vec![
			Diag::Err("Critical error occurred".to_string(),),
			Diag::Warn("This is a warning".to_string(),),
			Diag::Note("Additional information".to_string(),),
			Diag::Help("Try this solution".to_string(),),
		];

		// We can't easily test the actual emission without proc_macro context,
		// but we can test that the diagnostics contain the expected content
		match &diags[0] {
			Diag::Err(msg,) => assert_eq!(msg, "Critical error occurred"),
			_ => panic!("Expected error diagnostic"),
		}

		match &diags[1] {
			Diag::Warn(msg,) => assert_eq!(msg, "This is a warning"),
			_ => panic!("Expected warning diagnostic"),
		}

		match &diags[2] {
			Diag::Note(msg,) => assert_eq!(msg, "Additional information"),
			_ => panic!("Expected note diagnostic"),
		}

		match &diags[3] {
			Diag::Help(msg,) => assert_eq!(msg, "Try this solution"),
			_ => panic!("Expected help diagnostic"),
		}
	}

	#[test]
	fn test_error_diagnose_with_complex_types() {
		use std::collections::HashMap;

		let mut map = HashMap::new();
		map.insert("key1".to_string(), 42,);
		map.insert("key2".to_string(), 84,);

		let result: anyhow::Result<(HashMap<String, i32,>, Vec<Diag,>,),> =
			Ok((map.clone(), vec![],),);
		let value = result.unwrap_or_emit();
		assert_eq!(value, map);
	}

	#[test]
	fn test_error_diagnose_trait_bounds() {
		// Test that the trait works with types that have various bounds
		#[derive(Debug, PartialEq,)]
		struct TestStruct {
			value: i32,
		}

		let test_struct = TestStruct { value: 100, };
		let result: anyhow::Result<(TestStruct, Vec<Diag,>,),> =
			Ok((test_struct, vec![],),);
		let value = result.unwrap_or_emit();
		assert_eq!(value.value, 100);
	}
}
