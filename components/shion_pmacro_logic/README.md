# oso_proc_macro_logic

Core implementation logic for OSO procedural macros, providing the underlying functionality for compile-time code generation and specification-based development.

## Status

**Almost Complete** - Most functionality is implemented with several components still work-in-progress.

## Overview

This crate contains the actual implementation logic for the procedural macros defined in `oso_proc_macro`. It handles the complex parsing, web scraping, and code generation tasks that enable the OSO project's macro-driven development approach.

## Architecture

The crate is organized around several key areas of functionality:

### Specification Processing
- **Web Scraping**: Downloads and parses official hardware specifications from the internet
- **HTML Parsing**: Extracts structured data from specification documents
- **Code Generation**: Converts specification data into Rust code

### ELF Processing
- **Binary Analysis**: Interfaces with system tools like `readelf` for binary inspection
- **Header Validation**: Generates compile-time tests for ELF parsing correctness
- **Cross-reference Validation**: Ensures parsed data matches actual binary structure

### UEFI Integration
- **Status Code Generation**: Automatically generates UEFI status codes from official specifications
- **Protocol Definitions**: Processes UEFI protocol specifications into Rust interfaces
- **Version Management**: Handles multiple UEFI specification versions

## Key Features

### Specification-Driven Development
The core philosophy of this crate is to generate implementations directly from authoritative sources:

```rust
// Downloads UEFI 2.9 specification and generates status codes
let status_codes = generate_uefi_status_codes("2.9")?;
```

### Compile-Time Validation
Ensures that generated code matches real-world expectations:

```rust
// Validates ELF header parsing against actual binaries
validate_elf_header_parsing(&header_impl, &binary_path)?;
```

### Zero-Runtime-Cost Generation
All processing happens at compile time, resulting in optimal runtime performance.

## Dependencies

### Core Dependencies
- `anyhow`: Comprehensive error handling
- `colored`: Terminal output formatting
- `proc-macro2`: Token stream manipulation
- `quote`: Code generation utilities
- `syn`: Rust syntax tree parsing and manipulation

### Web Processing
- `ureq`: HTTP client for downloading specifications
- `html5ever`: HTML parsing engine
- `markup5ever`: Markup processing utilities
- `markup5ever_rcdom`: DOM tree construction
- `string_cache`: Efficient string interning
- `tendril`: String handling for parsing

### Utilities
- `itertools`: Iterator extension methods
- `oso_dev_util`: Shared development utilities

### Development Dependencies
- `tempfile`: Temporary file management for testing

## Functionality

### UEFI Specification Processing
Downloads and processes UEFI specifications to generate:
- Status code definitions with proper error handling
- Protocol interface definitions
- Constant value definitions
- Documentation comments from specifications

### ELF Binary Analysis
Integrates with system tools to:
- Extract header information from compiled binaries
- Generate validation tests for parsing implementations
- Ensure cross-platform compatibility
- Provide compile-time guarantees about binary structure

### HTML Document Processing
Sophisticated HTML parsing capabilities for:
- Extracting structured data from specification documents
- Converting HTML tables to Rust data structures
- Processing embedded code examples
- Maintaining links to source documentation

## Testing

The crate includes comprehensive testing infrastructure:

### Benchmarks
Performance benchmarks are available via:
```bash
cargo bench
```

### Unit Tests
Standard unit tests cover core functionality:
```bash
cargo test
```

### Integration Tests
Tests integration with external specifications and tools.

## System Requirements

### Network Access
Required for downloading specifications from official sources:
- UEFI specifications from uefi.org
- Hardware vendor documentation
- Standards organization documents

### System Tools
- `readelf`: For ELF binary analysis (part of binutils)
- Internet connectivity for specification downloads
- Sufficient disk space for caching downloaded specifications

## Error Handling

Comprehensive error handling covers:
- **Network Errors**: Failed downloads, timeouts, invalid URLs
- **Parsing Errors**: Malformed HTML, unexpected document structure
- **System Errors**: Missing tools, file system issues
- **Generation Errors**: Invalid code generation, syntax errors

All errors include detailed context and suggestions for resolution.

## Performance Considerations

### Caching
- Downloaded specifications are cached to avoid repeated network requests
- Parsed data structures are cached between compilation runs
- Generated code is optimized for minimal compile-time impact

### Memory Usage
- Streaming parsers minimize memory usage for large documents
- Efficient string interning reduces memory overhead
- Lazy evaluation defers expensive operations until needed

## Development Status

### Completed Features
- ✅ UEFI specification downloading and parsing
- ✅ ELF header validation generation
- ✅ HTML document processing pipeline
- ✅ Code generation framework
- ✅ Error handling and diagnostics

### Work in Progress
- 🔄 Additional hardware specification support
- 🔄 Enhanced caching mechanisms
- 🔄 Cross-platform tool integration
- 🔄 Performance optimizations

### Planned Features
- 📋 Support for additional specification formats
- 📋 Interactive specification browsing
- 📋 Automated specification update checking
- 📋 Enhanced debugging tools

## Contributing

When contributing to this crate:

1. **Maintain Specification Accuracy**: Ensure generated code matches official specifications
2. **Handle Errors Gracefully**: Provide helpful error messages and recovery suggestions
3. **Document External Dependencies**: Clearly specify required system tools and network access
4. **Test Thoroughly**: Include tests for both success and failure cases
5. **Consider Performance**: Optimize for compile-time performance

## License

MIT OR Apache-2.0

## Related Crates

- `oso_proc_macro`: Public procedural macro interfaces
- `oso_dev_util`: Shared development utilities
- `oso_error`: Error handling primitives
