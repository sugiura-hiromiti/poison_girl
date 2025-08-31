//! # oso_error
//!
//! A minimalist, no_std compatible error handling library designed for embedded
//! systems, operating systems, and other environments where the standard
//! library is unavailable.
//!
//! ## Features
//!
//! - `no_std` compatible error type with optional descriptive payload
//! - Lightweight error creation via the `oso_err!` macro
//! - Generic error type that can carry additional context
//! - Convenient type alias `Rslt<T>` for common Result usage
//!
//! ## Usage
//!
//! The crate provides a simple error type `OsoError<V>` that can be used to
//! represent errors in your application. The type parameter `V` allows you to
//! attach additional context to your errors.
//!
//! ### Basic Example
//!
//! ```rust
//! use oso_error::OsoError;
//! use oso_error::Rslt;
//! use oso_error::oso_err;
//!
//! fn divide(a: i32, b: i32,) -> Rslt<i32, &'static str,> {
//! 	if b == 0 {
//! 		// Create a basic error with just the module path
//! 		return Err(oso_err!("Division by zero"),);
//! 	}
//! 	Ok(a / b,)
//! }
//! ```
//!
//! ### With Custom Error Description
//!
//! ```rust
//! extern crate alloc;
//! use alloc::string::String;
//! use oso_error::OsoError;
//! use oso_error::Rslt;
//!
//! #[derive(Debug, Default,)]
//! struct DivisionError {
//! 	numerator:   i32,
//! 	denominator: i32,
//! }
//!
//! fn divide_with_context(a: i32, b: i32,) -> Rslt<i32, DivisionError,> {
//! 	if b == 0 {
//! 		// Create an error with additional context
//! 		let mut err = OsoError { from: module_path!(), desc: None, };
//! 		err.desc(DivisionError { numerator: a, denominator: b, },);
//! 		return Err(err,);
//! 	}
//! 	Ok(a / b,)
//! }
//! ```
//!
//! ### Error Handling
//!
//! ```should_panic
//! use oso_error::OsoError;
//! use oso_error::Rslt;
//! use oso_error::oso_err;
//!
//! fn process_value(val: i32,) -> Rslt<i32, &'static str,> {
//! 	// Some processing that might fail
//! 	if val < 0 {
//! 		return Err(oso_err!("Negative value"),);
//! 	}
//! 	Ok(val * 2,)
//! }
//!
//! fn main() -> Rslt<(), &'static str,> {
//! 	let result = process_value(-5,)?; // This will return early with the error
//! 	// This won't be reached if process_value returns an error
//! 	Ok((),)
//! }
//! ```
//!
//! ## Design Philosophy
//!
//! The `oso_error` crate is designed to be minimal yet flexible, providing just
//! enough functionality for error handling in constrained environments without
//! pulling in unnecessary dependencies or requiring the standard library.

#![no_std]
#![feature(type_alias_impl_trait)]

// extern crate alloc;

use core::fmt::Debug;

pub mod kernel;
pub mod loader;
pub mod parser;

/// A type alias for commonly used Result type with OsoError as the error type.
///
/// This provides a convenient shorthand for functions that return Results with
/// OsoError.
///
/// # Type Parameters
///
/// * `T` - The success type of the Result
/// * `V` - The type of the descriptive payload for the error (defaults to `()`)
///
/// # Examples
///
/// ```rust
/// use oso_error::Rslt;
/// use oso_error::oso_err;
///
/// fn might_fail() -> Rslt<i32, &'static str,> {
/// 	// Some operation that might fail
/// 	if true { Ok(42,) } else { Err(oso_err!("Operation failed"),) }
/// }
/// ```
pub type Rslt<T = (), V = (),> = Result<T, OsoError<V,>,>;

/// A flexible error type for representing errors in no_std environments.
///
/// `OsoError` provides a lightweight error representation that includes the
/// source module and an optional descriptive payload of type `V`.
///
/// # Type Parameters
///
/// * `V` - The type of the descriptive payload, must implement `Debug` and
///   `Default`
///
/// # Fields
///
/// * `from` - A static string identifying the source of the error (typically
///   the module path)
/// * `desc` - An optional descriptive payload providing additional context
///   about the error
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use oso_error::OsoError;
/// use oso_error::oso_err;
///
/// // Create a basic error using the macro
/// let error = oso_err!("Something went wrong");
///
/// // Create an error manually
/// let manual_error = OsoError { from: module_path!(), desc: None::<(),>, };
/// ```
///
/// With custom description:
///
/// ```rust
/// use oso_error::OsoError;
///
/// #[derive(Debug, Default,)]
/// struct NetworkError {
/// 	status_code: u16,
/// 	message:     String,
/// }
///
/// // Create an error with a custom description
/// let mut error =
/// 	OsoError::<NetworkError,> { from: module_path!(), desc: None, };
/// error.desc(NetworkError {
/// 	status_code: 404,
/// 	message:     "Resource not found".into(),
/// },);
/// ```
#[derive(Debug, Default,)]
pub struct OsoError<V,>
where V: Debug
{
	pub from: &'static str,
	pub desc: Option<V,>,
}

/// A macro for creating OsoError instances with minimal boilerplate.
///
/// This macro automatically sets the `from` field to the current module path
/// and initializes the error with default values.
///
/// # Parameters
///
/// * `$causal` - An expression describing the cause of the error (currently
///   unused in the implementation)
///
/// # Returns
///
/// An instance of `OsoError` with the `from` field set to the current module
/// path.
///
/// # Examples
///
/// ```rust
/// use oso_error::Rslt;
/// use oso_error::oso_err;
///
/// fn validate_input(input: i32,) -> Rslt<(), &'static str,> {
/// 	if input < 0 {
/// 		return Err(oso_err!("Negative input not allowed"),);
/// 	}
/// 	Ok((),)
/// }
/// ```
#[macro_export]
macro_rules! oso_err {
	($causal:expr) => {
		$crate::OsoError { from: module_path!(), desc: Some($causal,), }
	};
	() => {
		$crate::OsoError { from: module_path!(), ..Default::default() }
	};
}

impl<V: Debug + Default,> OsoError<V,> {
	/// Adds a descriptive payload to the error.
	///
	/// This method allows you to attach additional context to an error after
	/// creation.
	///
	/// # Parameters
	///
	/// * `val` - The descriptive payload to attach to the error
	///
	/// # Returns
	///
	/// A mutable reference to self, allowing for method chaining
	///
	/// # Examples
	///
	/// ```rust
	/// extern crate alloc;
	/// use alloc::string::String;
	/// use oso_error::OsoError;
	/// use oso_error::Rslt;
	///
	/// #[derive(Debug, Default,)]
	/// struct FileError {
	/// 	path:      String,
	/// 	operation: String,
	/// }
	///
	/// fn read_file(path: &str,) -> Rslt<String, FileError,> {
	/// 	// Simulate a file operation failure
	/// 	let mut err = OsoError { from: module_path!(), desc: None, };
	/// 	err.desc(FileError {
	/// 		path:      path.into(),
	/// 		operation: "read".into(),
	/// 	},);
	/// 	Err(err,)
	/// }
	/// ```
	pub fn desc(&mut self, val: V,) -> &mut Self {
		self.desc = Some(val,);
		self
	}
}

impl<V: Debug + Default,> From<OsoError<V,>,> for core::fmt::Error {
	fn from(_value: OsoError<V,>,) -> Self {
		core::fmt::Error
	}
}

impl From<OsoError<(),>,> for OsoError<&str,> {
	fn from(_value: OsoError<(),>,) -> Self {
		oso_err!()
	}
}

// #[derive(Debug,)]
// pub enum OsoLoaderError {
// 	Uefi(String,),
// 	EfiParse(String,),
// }
//
// impl Error for OsoLoaderError {}
//
// impl Display for OsoLoaderError {
// 	fn fmt(&self, f: &mut core::fmt::Formatter<'_,>,) -> core::fmt::Result {
// 		let represent = match self {
// 			OsoLoaderError::Uefi(e,) => format!("{e:?}"),
// 			OsoLoaderError::EfiParse(e,) => format!("{e:?}"),
// 		};
// 		write!(f, "{represent}")
// 	}
// }
//
// impl From<OsoLoaderError,> for core::fmt::Error {
// 	fn from(_value: OsoLoaderError,) -> Self {
// 		core::fmt::Error
// 	}
// }
//
// impl From<ParseIntError,> for OsoLoaderError {
// 	fn from(value: ParseIntError,) -> Self {
// 		Self::Uefi(value.to_string(),)
// 	}
// }

// pub mod error {
// 	#[derive(Debug,)]
// 	pub enum KernelError {
// 		Graphics(GraphicError,),
// 	}
//
// 	#[derive(Debug,)]
// 	pub enum GraphicError {
// 		InvalidCoordinate,
// 	}
// 	impl From<KernelError,> for core::fmt::Error {
// 		fn from(_value: KernelError,) -> Self {
// 			core::fmt::Error
// 		}
// 	}
// }
