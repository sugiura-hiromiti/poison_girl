# OSO Procedural Macros

This crate provides procedural macros for the OSO operating system project. It includes compile-time code generation utilities that help reduce boilerplate and ensure consistency across the OSO kernel and related components.

## Features

### Font Data Processing (`fonts_data!`)
Converts font files to embedded data structures at compile time.

```rust
// Generate font data from files in the "assets/fonts" directory
let fonts = fonts_data!("assets/fonts");
```

### Integer Type Implementation (`impl_int!`)
Generates implementations for integer types to reduce boilerplate.

```rust
// Generate implementations for multiple integer types
impl_int!(u8, u16, u32, u64);
```

### Wrapper Function Generation (`#[gen_wrapper_fn]`)
Generates wrapper functions for trait methods that delegate to a static instance.

```rust
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

### UEFI Status Code Generation (`status_from_spec!`)
Generates UEFI status code definitions from the official UEFI specification at compile time.

```rust
// Generate status codes from UEFI 2.9 specification
status_from_spec!(2.9);
```

This macro:
- Downloads the UEFI specification from the official website
- Parses status code definitions
- Generates a complete `Status` struct with associated constants
- Includes error handling methods (`ok_or()`, `ok_or_with()`)

### ELF Testing Utilities
Provides compile-time validation for ELF parsing implementations.

#### ELF Header Testing (`test_elf_header_parse!`)
```rust
// Test that a parsed ELF header matches expectations
test_elf_header_parse!(my_elf_header);
```

#### Program Headers Testing (`test_program_headers_parse!`)
```rust
// Test that parsed program headers match expectations
test_program_headers_parse!(my_program_headers);
```

These macros use `readelf` to extract expected values from the actual binary and generate compile-time assertions that only run in debug builds.

## Architecture

The crate is structured into two main components:

### Main Library (`lib.rs`)
Contains the procedural macro definitions and their primary logic. Each macro is thoroughly documented with:
- Parameter descriptions
- Return value specifications
- Usage examples
- Panic conditions
- Dependencies

### Helper Module (`helper.rs`)
Contains utility functions for:
- UEFI status code implementation generation
- ELF header parsing and validation
- Program header parsing and validation
- Token stream generation utilities

## Dependencies

### Runtime Dependencies
- `anyhow`: Error handling
- `colored`: Terminal output coloring
- `oso_proc_macro_logic`: Core logic implementations
- `proc-macro2`: Token stream manipulation
- `quote`: Code generation
- `syn`: Rust syntax parsing

### System Dependencies
- **Internet access**: Required for `status_from_spec!` macro to download UEFI specifications
- **`readelf` command**: Required for ELF testing macros (`test_elf_header_parse!`, `test_program_headers_parse!`)

## Usage in OSO Project

This crate is designed specifically for the OSO operating system project and provides:

1. **Compile-time validation**: Ensures ELF parsing implementations match actual binary structure
2. **Standards compliance**: Automatically generates up-to-date UEFI status codes from official specifications
3. **Code generation**: Reduces boilerplate in kernel and bootloader code
4. **Type safety**: Provides strongly-typed interfaces for low-level operations

## Build Requirements

- Rust 2024 edition
- Internet connectivity (for UEFI specification downloads)
- `readelf` utility (typically part of binutils)

## Debug vs Release Behavior

Some macros behave differently in debug vs release builds:

- **Debug builds**: ELF testing macros perform full validation
- **Release builds**: ELF testing macros generate no code for optimal performance

## Error Handling

The macros provide comprehensive error reporting:
- Compile-time errors for invalid inputs
- Network errors when downloading specifications
- Parsing errors with detailed diagnostics
- System command failures with context

## Contributing

When adding new procedural macros:

1. Add comprehensive documentation following the existing patterns
2. Include usage examples in doc comments
3. Specify all panic conditions and dependencies
4. Add appropriate error handling and diagnostics
5. Consider debug vs release build behavior
6. Update this README with new functionality

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.
