#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_test_elf_header_parse as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::fnl,
};

fnl! {
	test_elf_header_parse,
	[as proc_macro2::TokenStream,],
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
}
