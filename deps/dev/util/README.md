# oso_dev_util

Shared development utilities for the OSO operating system project, providing common functionality used across development tools and build systems.

## Status

**Current Development Milestone** - Part of the ongoing effort to complete implementations for development utilities.

## Overview

This crate serves as a central repository for utility functions, data structures, and helper code that is shared between various development tools in the OSO project. It aims to reduce code duplication and provide consistent interfaces across the development toolchain.

## Purpose

The `oso_dev_util` crate addresses the need for:
- **Code Reuse**: Common functionality shared between `xtask`, procedural macros, and other development tools
- **Consistency**: Standardized interfaces and behaviors across development utilities
- **Maintainability**: Centralized location for development-related utilities
- **Efficiency**: Avoiding duplicate implementations across the project

## Architecture

### Core Utilities
Provides fundamental utilities that are commonly needed across development tools:
- Configuration parsing and management
- File system operations with enhanced error handling
- Common data structures for development workflows
- Shared constants and type definitions

### Integration Points
Designed to integrate seamlessly with:
- **xtask**: Build system and automation tools
- **Procedural Macros**: Code generation utilities
- **Testing Infrastructure**: Test utilities and helpers
- **CI/CD Systems**: Continuous integration support

## Dependencies

### Core Dependencies
- `anyhow`: Comprehensive error handling with context
- `colored`: Terminal output formatting and coloring
- `toml`: TOML configuration file parsing with parse features

### Internal Dependencies
- `oso_proc_macro_2`: Integration with procedural macro system

## Features

### Configuration Management
Utilities for handling project configuration:
- TOML file parsing and validation
- Configuration merging and inheritance
- Environment-specific configuration handling
- Default value management

### Enhanced Error Handling
Building on `anyhow` for development-specific error scenarios:
- Context-rich error messages
- Development-friendly error formatting
- Integration with colored terminal output
- Error recovery suggestions

### File System Utilities
Enhanced file system operations for development workflows:
- Safe file operations with proper error handling
- Directory traversal and manipulation
- Temporary file management
- Cross-platform path handling

### Terminal Output
Consistent terminal output formatting:
- Colored output for different message types
- Progress indicators and status updates
- Structured logging for development tools
- User-friendly error presentation

## Usage

Add this to your development tool's `Cargo.toml`:

```toml
[dependencies]
oso_dev_util = { path = "../oso_dev_util" }
```

### Basic Example

```rust
use oso_dev_util::prelude::*;
use anyhow::Result;

fn main() -> Result<()> {
    // Parse project configuration
    let config = parse_project_config("Cargo.toml")?;
    
    // Perform file operations with enhanced error handling
    let content = read_file_with_context("src/main.rs")?;
    
    // Output with consistent formatting
    println!("{}", "Build completed successfully".green());
    
    Ok(())
}
```

## Integration with OSO Development Workflow

### Build System Integration
Provides utilities used by the `xtask` build system:
- Project structure detection
- Dependency resolution
- Build artifact management
- Cross-compilation support

### Macro Development Support
Supports procedural macro development with:
- Code generation utilities
- Template processing
- Specification parsing helpers
- Testing infrastructure

### CI/CD Support
Facilitates continuous integration with:
- Environment detection
- Build configuration management
- Test result processing
- Artifact publishing utilities

## Development Philosophy

### Reliability First
All utilities prioritize reliability and clear error reporting over performance, making development workflows more predictable and debuggable.

### Developer Experience
Focuses on providing excellent developer experience through:
- Clear, actionable error messages
- Consistent interfaces across tools
- Comprehensive documentation
- Intuitive API design

### Maintainability
Designed for long-term maintainability:
- Well-documented public interfaces
- Comprehensive test coverage
- Clear separation of concerns
- Minimal external dependencies

## Current Development Status

As part of the current milestone to complete development utility implementations:

### Completed
- ✅ Basic utility framework
- ✅ Configuration parsing infrastructure
- ✅ Error handling patterns
- ✅ Terminal output utilities

### In Progress
- 🔄 File system operation utilities
- 🔄 Build system integration helpers
- 🔄 Testing infrastructure support
- 🔄 Cross-platform compatibility improvements

### Planned
- 📋 Advanced configuration management
- 📋 Plugin system for extensibility
- 📋 Performance monitoring utilities
- 📋 Documentation generation helpers

## Testing

The crate includes comprehensive testing:

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run with verbose output
cargo test -- --nocapture
```

## Contributing

When contributing to `oso_dev_util`:

1. **Focus on Reusability**: Ensure utilities can be used across multiple development tools
2. **Maintain Consistency**: Follow established patterns and interfaces
3. **Document Thoroughly**: Provide clear documentation and examples
4. **Test Comprehensively**: Include tests for both success and failure cases
5. **Consider Cross-Platform**: Ensure utilities work across supported platforms

## Error Handling

The crate uses `anyhow` for error handling with development-specific enhancements:
- Context-rich error messages
- Suggestions for error resolution
- Integration with colored terminal output
- Structured error reporting for tools

## License

MIT OR Apache-2.0

## Related Crates

- `xtask`: Main build system that uses these utilities
- `oso_proc_macro_logic`: Procedural macro implementation that depends on these utilities
- `oso_proc_macro_2`: Procedural macro system integration
