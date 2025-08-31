# oso_error

A minimalist, no_std compatible error handling library designed for embedded systems, operating systems, and other environments where the standard library is unavailable.

## Features

- `no_std` compatible error type with optional descriptive payload
- Lightweight error creation via the `oso_err!` macro
- Generic error type that can carry additional context
- Convenient type alias `Rslt<T>` for common Result usage

## Usage

### Basic Error Creation

```rust
use oso_error::{oso_err, Rslt};

fn process_data(input: i32) -> Rslt<i32> {
    if input < 0 {
        return Err(oso_err!("Negative input"));
    }
    Ok(input * 2)
}
```

### With Custom Error Description

```rust
use oso_error::{OsoError, Rslt};
use alloc::string::String;

#[derive(Debug, Default)]
struct ValidationError {
    field: String,
    reason: String,
}

fn validate_user(name: &str, age: i32) -> Rslt<(), ValidationError> {
    if name.is_empty() {
        let mut err = OsoError { from: module_path!(), desc: None };
        err.desc(ValidationError {
            field: "name".into(),
            reason: "Name cannot be empty".into(),
        });
        return Err(err);
    }
    
    if age < 0 || age > 120 {
        let mut err = OsoError { from: module_path!(), desc: None };
        err.desc(ValidationError {
            field: "age".into(),
            reason: "Age must be between 0 and 120".into(),
        });
        return Err(err);
    }
    
    Ok(())
}
```

### Error Propagation

```rust
use oso_error::{oso_err, Rslt};

fn step_one() -> Rslt<i32> {
    // Some operation that might fail
    Ok(42)
}

fn step_two(value: i32) -> Rslt<String> {
    if value < 50 {
        return Err(oso_err!("Value too small"));
    }
    Ok(format!("Processed: {}", value))
}

fn process() -> Rslt<String> {
    let value = step_one()?;
    let result = step_two(value)?;
    Ok(result)
}
```

## Design Philosophy

The `oso_error` crate is designed to be minimal yet flexible, providing just enough functionality for error handling in constrained environments without pulling in unnecessary dependencies or requiring the standard library.

## License

[Add your license information here]
