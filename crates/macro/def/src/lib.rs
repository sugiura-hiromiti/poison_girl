#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use {
	poison_girl_proc_macro_helper::{
		atr,
		diagnostic::{Diag, NotationDiag},
		drv, fnl,
		rslt::Rslt,
	},
	proc_macro::{Diagnostic, Level},
};

trait ErrorDiagnose {
	type T;
	fn unwrap_or_emit(self,) -> Self::T;
}

impl<V,> ErrorDiagnose for Rslt<V,> {
	type T = V;

	fn unwrap_or_emit(self,) -> Self::T {
		if self.has_err() {
			panic!("{:?}", self.into_err());
		}

		self.notation().iter().for_each(|d| match d {
			NotationDiag::Warn(msg,) => {
				Diagnostic::new(Level::Warning, msg,).emit();
			},
			NotationDiag::Note(msg,) => {
				Diagnostic::new(Level::Note, msg,).emit();
			},
			NotationDiag::Help(msg,) => {
				Diagnostic::new(Level::Help, msg,).emit();
			},
		},);
		self.into_value().unwrap()
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
