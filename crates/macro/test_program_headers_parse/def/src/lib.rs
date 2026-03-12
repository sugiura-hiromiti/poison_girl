#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_test_program_headers_parse as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::fnl,
};

fnl! {
	test_program_headers_parse,
	[with syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>::parse_terminated,],
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
}
