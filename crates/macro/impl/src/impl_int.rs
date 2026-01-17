//! # Trait Implementation Generation Module
//!
//! This module provides functionality for automatically generating trait
//! implementations for integer types. It includes parsing utilities for type
//! lists and code generation for common integer operations like digit counting
//! and manipulation.

use proc_macro2::TokenTree;
use syn::TypePath;
use syn::parse::Parse;
use syn::spanned::Spanned;

use crate::RsltP;

/// A collection of Rust types parsed from a token stream
///
/// This struct holds a list of `syn::Type` objects that can be used for
/// generating trait implementations or other type-based code generation tasks.
pub struct Types {
	/// Internal storage for the parsed types
	type_list: Vec<syn::Type,>,
}

impl Types {
	/// Returns an iterator over the contained types
	///
	/// # Returns
	///
	/// An iterator that yields references to each `syn::Type` in the collection
	///
	/// # Examples
	///
	/// ```ignore
	/// let types = parse_types("u8, u16, u32");
	/// for ty in types.iter() {
	///     println!("Type: {}", quote::quote!(#ty));
	/// }
	/// ```
	pub fn iter(&self,) -> std::slice::Iter<'_, syn::Type,> {
		self.type_list.iter()
	}
}

impl Parse for Types {
	/// Parses a comma-separated list of type identifiers from a token stream
	///
	/// This implementation allows parsing type lists like "u8, u16, u32, i64"
	/// from procedural macro input. It handles identifiers and punctuation,
	/// converting identifiers to `syn::Type` objects.
	///
	/// # Arguments
	///
	/// * `input` - The parse stream containing the tokens to parse
	///
	/// # Returns
	///
	/// A `syn::Result<Types>` containing the parsed types or an error
	///
	/// # Errors
	///
	/// Returns an error if:
	/// - Unexpected token types are encountered (not identifiers or
	///   punctuation)
	/// - The token stream is malformed
	fn parse(input: syn::parse::ParseStream,) -> syn::Result<Self,> {
		let parsed = input.step(|c| {
			let mut rest = *c;
			let mut type_list = vec![];

			// Process each token in the stream
			while let Some((tt, next,),) = rest.token_tree() {
				match tt {
					// Convert identifiers to types
					TokenTree::Ident(idnt,) => {
						let ty: syn::Type = syn::parse_quote! { #idnt };
						type_list.push(ty,);
						rest = next;
					},
					// Skip punctuation (commas, etc.)
					TokenTree::Punct(_,) => rest = next,
					// Error on unexpected tokens
					_ => {
						return Err(syn::Error::new(
							tt.span(),
							format!("parse failed\ntoken tree is: {tt:#?}"),
						),);
					},
				};
			}
			Ok((Types { type_list, }, rest,),)
		},)?;
		Ok(parsed,)
	}
}

pub fn impl_int(types: Types,) -> RsltP {
	let integers = types.iter().map(implement,);

	Ok((
		quote::quote! {
			#(#integers)*
		},
		vec![],
	),)
}

/// Generates a trait implementation for the `Integer` trait for a given type
///
/// This function creates a complete implementation of the `Integer` trait
/// for primitive integer types, including methods for digit counting,
/// digit extraction, and bit shifting operations.
///
/// # Arguments
///
/// * `ty` - The type for which to generate the implementation
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the generated trait implementation
///
/// # Generated Methods
///
/// The generated implementation includes:
/// - `digit_count()`: Counts the number of decimal digits
/// - `nth_digit(n)`: Extracts the nth digit from the right
/// - `shift_right()`: Removes the rightmost digit and returns it
///
/// # Examples
///
/// ```ignore
/// let ty: syn::Type = syn::parse_quote!(u32);
/// let impl_tokens = implement(&ty);
/// // Generates: impl Integer for u32 { ... }
/// ```
pub fn implement(ty: &syn::Type,) -> proc_macro2::TokenStream {
	let idnt = unwrap_primitive(ty,).unwrap();
	let digit_count = digit_count_impl();
	let nth_digit = nth_digit_impl();
	let shift_right = shift_right_impl(&idnt,);

	quote::quote! {
		impl Integer for #idnt {
			#digit_count
			#nth_digit
			#shift_right
		}
	}
}

/// Extracts the identifier from a primitive type path
///
/// This function unwraps a `syn::Type` to extract the underlying identifier,
/// ensuring it represents a simple primitive type without generic parameters
/// or complex path structures.
///
/// # Arguments
///
/// * `ty` - The type to unwrap
///
/// # Returns
///
/// A `syn::Result<syn::Ident>` containing the type identifier or an error
///
/// # Errors
///
/// Returns an error if:
/// - The type is not a simple path type
/// - The type has generic parameters
/// - The type has a complex path structure
fn unwrap_primitive(ty: &syn::Type,) -> syn::Result<syn::Ident,> {
	// Extract segment as `seg` from `ty`
	let syn::Type::Path(TypePath {
		qself: None,
		path: syn::Path { leading_colon: None, segments: seg, },
	},) = ty
	else {
		return Err(syn::Error::new(
			ty.span(),
			format!("unable to unwrap type: {ty:#?}"),
		),);
	};

	if seg.len() != 1 {
		return Err(syn::Error::new(
			ty.span(),
			format!(
				"type may not primitive: {ty:#?}. if not, remove leading path"
			),
		),);
	}

	// Extract ident of type from `seg`
	let syn::PathSegment { ident: idnt, arguments: syn::PathArguments::None, } =
		seg.first().unwrap()
	else {
		return Err(syn::Error::new(
			seg.span(),
			format!("unable to unwrap path segment: {seg:#?}"),
		),);
	};

	Ok(idnt.clone(),)
}

/// Generates the implementation for the `digit_count` method
///
/// This method counts the number of decimal digits in an integer by
/// repeatedly dividing by 10 until the number becomes 0.
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the method implementation
fn digit_count_impl() -> proc_macro2::TokenStream {
	quote::quote! {
		fn digit_count(&self) -> usize {
			let mut n = self.clone();
			let mut digits = 0;

			// Count digits by dividing by 10
			while n != 0 {
				n = n / 10;
				digits += 1;
			}

			digits
		}
	}
}

/// Generates the implementation for the `nth_digit` method
///
/// This method extracts the nth digit from the right (1-indexed).
/// It shifts the number right n-1 times and then takes modulo 10.
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the method implementation
fn nth_digit_impl() -> proc_macro2::TokenStream {
	quote::quote! {
		/// Extracts the nth digit from the right (1-indexed)
		///
		/// # Arguments
		///
		/// * `n` - The position of the digit to extract (1 = rightmost digit)
		///
		/// # Returns
		///
		/// The digit at position n as a u8
		///
		/// # Panics
		///
		/// Panics if `n` is 0, as digit positions are 1-indexed
		fn nth_digit(&self, n: usize) -> u8 {
			assert_ne!(n, 0);
			let mut origin = self.clone();

			// Shift right n-1 times to get the desired digit in the ones place
			for _i in 1..n {
				origin.shift_right();
			}

			// Extract the ones digit
			(origin % 10) as u8
		}
	}
}

/// Generates the implementation for the `shift_right` method
///
/// This method removes the rightmost decimal digit and returns it.
/// The implementation differs for signed and unsigned types to handle
/// negative numbers correctly.
///
/// # Arguments
///
/// * `idnt` - The type identifier to determine if it's signed or unsigned
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the method implementation
fn shift_right_impl(idnt: &syn::Ident,) -> proc_macro2::TokenStream {
	// Different handling for signed vs unsigned types
	let return_value = if idnt.to_string().contains("u",) {
		// Unsigned types: direct conversion
		quote::quote! {
			first_digit as u8
		}
	} else {
		// Signed types: handle negative numbers by taking absolute value
		quote::quote! {
			if first_digit < 0 {
				-first_digit as u8
			} else {
				first_digit as u8
			}
		}
	};

	quote::quote! {
		/// Removes and returns the rightmost decimal digit
		///
		/// This method modifies the number by removing its rightmost digit
		/// (equivalent to integer division by 10) and returns that digit.
		///
		/// # Returns
		///
		/// The rightmost digit as a u8
		fn shift_right(&mut self) -> u8 {
			// Extract the rightmost digit
			let first_digit = *self % 10;

			// Remove the rightmost digit
			*self = *self / 10;

			// Return the digit, handling sign for signed types
			#return_value
		}
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use quote::quote;
	use syn::Type;
	use syn::parse_quote;

	#[test]
	fn test_types_parse_single_type() {
		let input = quote! { u32 };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let type_list: Vec<_,> = types.iter().collect();
		assert_eq!(type_list.len(), 1);
	}

	#[test]
	fn test_types_parse_multiple_types() {
		let input = quote! { u8, u16, u32, u64 };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let type_list: Vec<_,> = types.iter().collect();
		assert_eq!(type_list.len(), 4);
	}

	#[test]
	fn test_types_parse_with_extra_commas() {
		let input = quote! { u8, , u16, , u32, };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let type_list: Vec<_,> = types.iter().collect();
		assert_eq!(type_list.len(), 3);
	}

	#[test]
	fn test_types_parse_signed_types() {
		let input = quote! { i8, i16, i32, i64 };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let type_list: Vec<_,> = types.iter().collect();
		assert_eq!(type_list.len(), 4);
	}

	#[test]
	fn test_types_parse_mixed_types() {
		let input = quote! { u8, i16, u32, i64, usize, isize };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let type_list: Vec<_,> = types.iter().collect();
		assert_eq!(type_list.len(), 6);
	}

	#[test]
	fn test_types_parse_empty_input() {
		let input = quote! {};
		let types: Types =
			syn::parse2(input,).expect("Failed to parse empty types",);

		let type_list: Vec<_,> = types.iter().collect();
		assert_eq!(type_list.len(), 0);
	}

	#[test]
	fn test_types_parse_error_on_invalid_token() {
		let input = quote! { u32, "invalid", u64 };
		let result: Result<Types, _,> = syn::parse2(input,);

		assert!(result.is_err());
	}

	#[test]
	fn test_unwrap_primitive_u32() {
		let ty: Type = parse_quote! { u32 };
		let ident = unwrap_primitive(&ty,).expect("Failed to unwrap u32",);

		assert_eq!(ident.to_string(), "u32");
	}

	#[test]
	fn test_unwrap_primitive_i64() {
		let ty: Type = parse_quote! { i64 };
		let ident = unwrap_primitive(&ty,).expect("Failed to unwrap i64",);

		assert_eq!(ident.to_string(), "i64");
	}

	#[test]
	fn test_unwrap_primitive_usize() {
		let ty: Type = parse_quote! { usize };
		let ident = unwrap_primitive(&ty,).expect("Failed to unwrap usize",);

		assert_eq!(ident.to_string(), "usize");
	}

	#[test]
	fn test_unwrap_primitive_error_on_generic() {
		let ty: Type = parse_quote! { Vec<i32> };
		let result = unwrap_primitive(&ty,);

		assert!(result.is_err());
	}

	#[test]
	fn test_unwrap_primitive_error_on_path() {
		let ty: Type = parse_quote! { std::collections::HashMap };
		let result = unwrap_primitive(&ty,);

		assert!(result.is_err());
	}

	#[test]
	fn test_unwrap_primitive_error_on_reference() {
		let ty: Type = parse_quote! { &str };
		let result = unwrap_primitive(&ty,);

		assert!(result.is_err());
	}

	#[test]
	fn test_implement_generates_valid_tokens() {
		let ty: Type = parse_quote! { u32 };
		let implementation = implement(&ty,);

		// Should generate valid Rust code
		let code_str = implementation.to_string();
		assert!(code_str.contains("impl Integer for u32"));
		assert!(code_str.contains("fn digit_count"));
		assert!(code_str.contains("fn nth_digit"));
		assert!(code_str.contains("fn shift_right"));
	}

	#[test]
	fn test_implement_unsigned_type() {
		let ty: Type = parse_quote! { u64 };
		let implementation = implement(&ty,);

		let code_str = implementation.to_string();
		assert!(code_str.contains("impl Integer for u64"));
		// For unsigned types, should use direct conversion
		assert!(code_str.contains("first_digit as u8"));
	}

	#[test]
	fn test_implement_signed_type() {
		let ty: Type = parse_quote! { i32 };
		let implementation = implement(&ty,);

		let code_str = implementation.to_string();
		assert!(code_str.contains("impl Integer for i32"));
		// For signed types, should handle negative numbers
		assert!(code_str.contains("if first_digit < 0"));
	}

	#[test]
	fn test_digit_count_impl_structure() {
		let implementation = digit_count_impl();
		let code_str = implementation.to_string();

		assert!(code_str.contains("fn digit_count"));
		assert!(code_str.contains("-> usize"));
		assert!(code_str.contains("while n != 0"));
		assert!(code_str.contains("n = n / 10"));
		assert!(code_str.contains("digits += 1"));
	}

	#[test]
	fn test_nth_digit_impl_structure() {
		let implementation = nth_digit_impl();
		let code_str = implementation.to_string();

		assert!(code_str.contains("fn nth_digit"));
		assert!(code_str.contains("n : usize"));
		assert!(code_str.contains("-> u8"));
		assert!(code_str.contains("assert_ne ! (n , 0)"));
		assert!(code_str.contains("origin . shift_right ()"));
		assert!(code_str.contains("(origin % 10) as u8"));
	}

	#[test]
	fn test_shift_right_impl_unsigned() {
		let ident: syn::Ident = syn::parse_str("u32",).unwrap();
		let implementation = shift_right_impl(&ident,);
		let code_str = implementation.to_string();

		assert!(code_str.contains("fn shift_right"));
		assert!(code_str.contains("-> u8"));
		assert!(code_str.contains("first_digit = * self % 10"));
		assert!(code_str.contains("* self = * self / 10"));
		assert!(code_str.contains("first_digit as u8"));
		assert!(!code_str.contains("if first_digit < 0"));
	}

	#[test]
	fn test_shift_right_impl_signed() {
		let ident: syn::Ident = syn::parse_str("i32",).unwrap();
		let implementation = shift_right_impl(&ident,);
		let code_str = implementation.to_string();

		assert!(code_str.contains("fn shift_right"));
		assert!(code_str.contains("-> u8"));
		assert!(code_str.contains("first_digit = * self % 10"));
		assert!(code_str.contains("* self = * self / 10"));
		assert!(code_str.contains("if first_digit < 0"));
		assert!(code_str.contains("- first_digit as u8"));
	}

	#[test]
	fn test_types_iter_functionality() {
		let input = quote! { u8, u16, u32 };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let mut count = 0;
		for _ty in types.iter() {
			count += 1;
		}

		assert_eq!(count, 3);
	}

	#[test]
	fn test_complete_workflow() {
		// Test the complete workflow from parsing to implementation
		let input = quote! { u8, i16, u32 };
		let types: Types = syn::parse2(input,).expect("Failed to parse types",);

		let implementations: Vec<_,> =
			types.iter().map(|ty| implement(ty,),).collect();

		assert_eq!(implementations.len(), 3);

		// Check that each implementation is valid
		for (_i, impl_tokens,) in implementations.iter().enumerate() {
			let code_str = impl_tokens.to_string();
			assert!(code_str.contains("impl Integer for"));
			assert!(code_str.contains("fn digit_count"));
			assert!(code_str.contains("fn nth_digit"));
			assert!(code_str.contains("fn shift_right"));
		}
	}
}
