//! # Parser Generation Framework
//!
//! This module provides the core traits and types for building composable
//! parsers in the OSO operating system. The framework is designed around traits
//! that allow for flexible parser construction and zero-cost abstractions.
//!
//! ## Core Concepts
//!
//! - **ParserGenerator**: Trait for types that can generate parsers
//! - **Context**: Represents the parsing context and target data
//! - **Parser**: The actual parser that performs parsing operations
//! - **ParserComponents**: Building blocks for constructing complex parsers
//!
//! ## Design Philosophy
//!
//! The parser framework emphasizes compile-time composition and type safety,
//! making it suitable for system-level parsing tasks where performance and
//! reliability are critical.

use oso_error::Rslt;
use oso_error::parser::ParserError;

// ==================== Parser Generation Framework ====================

/// Trait for types that can generate parsers.
///
/// This trait is marked with `#[const_trait]` to allow for compile-time
/// parser generation, enabling zero-cost abstractions and compile-time
/// optimizations.
///
/// # Type Parameters
///
/// - `C`: The context type that implements the `Context` trait
///
/// # Examples
///
/// ```rust,ignore
/// use oso_no_std_shared::parser::generator::{ParserGenerator, Context};
///
/// struct MyParserGen;
///
/// impl<C: Context> ParserGenerator<C> for MyParserGen {
///     fn parser<PC: Context>(&self) -> impl Parser<PC> {
///         // Return a parser implementation
///         todo!()
///     }
/// }
/// ```
#[const_trait]
pub trait ParserGenerator<C: Context,> {
	// Future expansion possibilities:
	// fn parser_lists(&self,) -> &[impl Parser<C,>];
	// fn parser<F: Fn(&Self,) -> Rslt<R, ParserError,>, R,>(&self,) -> F;

	/// Generate a parser for the specified context type.
	///
	/// This method creates a parser that can operate on the given context type.
	/// The parser is returned as an `impl Parser<PC>` to allow for flexible
	/// implementation types while maintaining type safety.
	///
	/// # Type Parameters
	///
	/// - `PC`: The parser context type that implements `Context`
	///
	/// # Returns
	///
	/// A parser implementation that can parse data of type `PC::Output`
	fn parser<PC: Context,>(&self,) -> impl Parser<PC,>;
}

/// Trait representing the parsing context and target data structure.
///
/// The context provides information about the data being parsed, including
/// the output type, size constraints, and current parsing position. This
/// trait is designed to be implemented by types that represent the target
/// of parsing operations.
///
/// # Associated Types
///
/// - `Output`: The type that will be produced by successful parsing
///
/// # Associated Constants
///
/// - `SIZE`: The size in bytes of the output type (defaults to
///   `size_of::<Self::Output>()`)
pub trait Context {
	/// The type that will be produced by successful parsing
	type Output;

	/// The size in bytes of the output type
	///
	/// This constant provides compile-time size information that can be used
	/// for buffer allocation and bounds checking during parsing.
	const SIZE: usize = size_of::<Self::Output,>();

	/// Get the current position in the parsing context.
	///
	/// This method returns the current byte offset or position within the
	/// data being parsed, which is useful for error reporting and seeking.
	///
	/// # Returns
	///
	/// The current position as a byte offset
	fn pos(&self,) -> usize;

	/// Get the number of fields in the target data structure.
	///
	/// This method is currently a placeholder for future functionality
	/// that might involve field-level parsing operations.
	fn field_count() {}
}

// ==================== Parser Component Framework ====================

/// Trait for individual parser components that can be composed together.
///
/// Parser components represent small, focused parsing operations that can
/// be combined to create more complex parsers. Each component can transform
/// the parsing context and produce a result.
///
/// # Type Parameters
///
/// - `C`: The context type that implements the `Context` trait
///
/// # Design Notes
///
/// This trait is designed to support a compositional approach to parser
/// construction, where complex parsers are built from simpler components.
pub trait ParserComponents<C: Context,> {
	/// Apply this parser component to the given context.
	///
	/// This method performs the parsing operation represented by this
	/// component, potentially modifying the context and producing a result of
	/// type `R`.
	///
	/// # Type Parameters
	///
	/// - `R`: The result type produced by this parser component
	///
	/// # Parameters
	///
	/// - `context`: Mutable reference to the parsing context
	///
	/// # Returns
	///
	/// A result containing either the parsed value of type `R` or a
	/// `ParserError`
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use oso_no_std_shared::parser::generator::{ParserComponents, Context};
	/// use oso_error::Rslt;
	///
	/// struct IntegerParser;
	///
	/// impl<C: Context> ParserComponents<C> for IntegerParser {
	///     fn map<R>(&self, context: &mut C) -> Rslt<R> {
	///         // Parse an integer from the context
	///         todo!()
	///     }
	/// }
	/// ```
	fn map<R,>(&self, context: &mut C,) -> Rslt<R,>;
}

// ==================== Final Parser Interface ====================

/// Trait for complete parsers that can parse data from a context.
///
/// This trait represents the final parser interface that combines all the
/// parsing components and logic needed to transform input data into the
/// desired output type. Parsers implementing this trait should be ready
/// to perform complete parsing operations.
///
/// # Type Parameters
///
/// - `C`: The context type that implements the `Context` trait
///
/// # Future Enhancement
///
/// TODO: Implement this trait for the `Tree` data structure to enable
/// tree-based parsing operations.
pub trait Parser<C: Context,> {
	/// Parse data from the context and produce the output.
	///
	/// This method performs the complete parsing operation, consuming or
	/// processing the context data to produce the final parsed result.
	///
	/// # Returns
	///
	/// A result containing either the successfully parsed data of type
	/// `C::Output` or a `ParserError` describing what went wrong.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use oso_no_std_shared::parser::generator::{Parser, Context};
	/// use oso_error::Rslt;
	/// use oso_error::parser::ParserError;
	///
	/// struct JsonParser;
	///
	/// impl<C: Context> Parser<C> for JsonParser {
	///     fn parse(&self) -> Rslt<C::Output, ParserError> {
	///         // Perform JSON parsing
	///         todo!()
	///     }
	/// }
	/// ```
	fn parse(&self,) -> Rslt<C::Output, ParserError,>;
}
