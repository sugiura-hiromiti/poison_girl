#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_status as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::fnl,
};

fnl! {
	status,
	[as syn::Lit,],
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
}
