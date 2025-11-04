//! # Function Wrapper Generation Utilities
//!
//! This module provides utilities for generating wrapper functions and
//! extracting method arguments from function signatures. It's primarily used in
//! procedural macros that need to analyze and transform function definitions.

use crate::RsltP;
use syn::Signature;

pub fn wrapper(
	static_frame_buffer: syn::Ident,
	trait_def: syn::ItemTrait,
) -> RsltP {
	// Generate wrapper functions for each trait method
	let wrapper_fns = trait_def.items.clone().into_iter().filter_map(|i| {
		if let syn::TraitItem::Fn(method,) = i {
			let sig = method.sig;

			// Extract function signature components
			let constness = sig.constness;
			let asyncness = sig.asyncness;
			let unsafety = sig.unsafety;
			let abi = &sig.abi;
			let fn_name = &sig.ident;
			let generics = &sig.generics;

			// Filter out 'self' parameters for the wrapper function
			let fn_params = sig.inputs.iter().filter(|a| matches!(a, &&syn::FnArg::Typed(_)),);

			// Generate method arguments for the delegation call
			let method_args = method_args(&sig);
			let variadic = &sig.variadic;
			let output = &sig.output;

			// Generate the wrapper function declaration
			let decl = quote::quote! {
				pub #unsafety #asyncness #constness #abi fn #fn_name #generics(#(#fn_params),* #variadic) #output {
					#static_frame_buffer.#fn_name(#(#method_args),*)
				}
			};
			Some(decl,)
		} else {
			// Skip non-function trait items
			None
		}
	},);

	// Combine wrapper functions with the original trait definition
	let wrapper_fns = quote::quote! {
		#(#wrapper_fns)*
		#trait_def
	};
	Ok((wrapper_fns, vec![],),)
}

/// Extracts method arguments from a function signature, excluding the receiver
/// (`self`)
///
/// This function analyzes a function signature and returns an iterator over all
/// the argument patterns, filtering out any receiver arguments (like `self`,
/// `&self`, `&mut self`, etc.). This is useful when generating wrapper
/// functions or when you need to forward arguments to another function.
///
/// # Arguments
///
/// * `sig` - A reference to a `syn::Signature` representing the function
///   signature to analyze
///
/// # Returns
///
/// An iterator that yields `Box<syn::Pat>` for each non-receiver argument in
/// the signature. The patterns represent the argument names and destructuring
/// patterns.
///
/// # Examples
///
/// ```ignore
/// use syn::{parse_quote, Signature};
///
/// let sig: Signature = parse_quote! {
///     fn example(&self, arg1: i32, arg2: String) -> bool
/// };
///
/// let args: Vec<_> = method_args(&sig).collect();
/// assert_eq!(args.len(), 2); // Only arg1 and arg2, self is filtered out
/// ```
///
/// # Use Cases
///
/// - Generating wrapper functions that need to forward arguments
/// - Creating proxy methods that delegate to other implementations
/// - Analyzing function signatures in procedural macros
/// - Building function call expressions with the same arguments
pub fn method_args(
	sig: &Signature,
) -> impl Iterator<Item = std::boxed::Box<syn::Pat,>,> {
	sig.inputs.iter().filter_map(|a| match a {
		// Skip receiver arguments (self, &self, &mut self, etc.)
		syn::FnArg::Receiver(_,) => None,

		// Extract the pattern from typed arguments
		syn::FnArg::Typed(pty,) => Some(pty.pat.clone(),),
	},)
}
#[cfg(test)]
mod tests {
	use super::*;
	use syn::Signature;
	use syn::parse_quote;

	#[test]
	fn test_method_args_no_receiver() {
		let sig: Signature = parse_quote! {
			fn test_function(arg1: i32, arg2: String, arg3: bool) -> i32
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		assert_eq!(args.len(), 3);
	}

	#[test]
	fn test_method_args_with_self_receiver() {
		let sig: Signature = parse_quote! {
			fn test_method(&self, arg1: i32, arg2: String) -> bool
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &self, only return the 2 typed arguments
		assert_eq!(args.len(), 2);
	}

	#[test]
	fn test_method_args_with_mut_self_receiver() {
		let sig: Signature = parse_quote! {
			fn test_method(&mut self, arg1: i32) -> ()
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &mut self, only return the 1 typed argument
		assert_eq!(args.len(), 1);
	}

	#[test]
	fn test_method_args_with_owned_self_receiver() {
		let sig: Signature = parse_quote! {
			fn test_method(self, arg1: String, arg2: Vec<i32>) -> String
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude self, only return the 2 typed arguments
		assert_eq!(args.len(), 2);
	}

	#[test]
	fn test_method_args_no_arguments() {
		let sig: Signature = parse_quote! {
			fn test_function() -> ()
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		assert_eq!(args.len(), 0);
	}

	#[test]
	fn test_method_args_only_receiver() {
		let sig: Signature = parse_quote! {
			fn test_method(&self) -> i32
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &self, return empty
		assert_eq!(args.len(), 0);
	}

	#[test]
	fn test_method_args_complex_types() {
		let sig: Signature = parse_quote! {
			fn complex_method(
				&self,
				arg1: Vec<String>,
				arg2: HashMap<String, i32>,
				arg3: Option<Box<dyn Trait>>,
				arg4: &mut [u8]
			) -> Result<(), Error>
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &self, return 4 typed arguments
		assert_eq!(args.len(), 4);
	}

	#[test]
	fn test_method_args_pattern_matching() {
		let sig: Signature = parse_quote! {
			fn pattern_method(&self, (x, y): (i32, i32), [a, b, c]: [u8; 3]) -> ()
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &self, return 2 pattern arguments
		assert_eq!(args.len(), 2);
	}

	#[test]
	fn test_method_args_with_lifetimes() {
		let sig: Signature = parse_quote! {
			fn lifetime_method<'a>(&self, arg1: &'a str, arg2: &'a mut Vec<i32>) -> &'a str
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &self, return 2 typed arguments with lifetimes
		assert_eq!(args.len(), 2);
	}

	#[test]
	fn test_method_args_with_generics() {
		let sig: Signature = parse_quote! {
			fn generic_method<T, U>(&self, arg1: T, arg2: Vec<U>, arg3: Option<T>) -> Result<T, U>
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &self, return 3 typed arguments with generics
		assert_eq!(args.len(), 3);
	}

	#[test]
	fn test_method_args_preserves_pattern_structure() {
		let sig: Signature = parse_quote! {
			fn test_fn(&self, arg1: i32, arg2: String,)
		};

		let args: Vec<_,> = method_args(&sig,).collect();

		// Check that we can convert patterns back to tokens
		for arg in args {
			let _tokens = quote::quote! { #arg };
			// If this compiles, the pattern structure is preserved
		}
	}

	#[test]
	fn test_wrapper_function_basic() {
		let static_frame_buffer =
			syn::Ident::new("FRAME_BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait TestTrait {
				fn test_method(&self, arg: i32,) -> bool;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that wrapper function is generated
		assert!(token_string.contains("pub fn test_method"));
		assert!(token_string.contains("FRAME_BUFFER . test_method"));
		assert!(token_string.contains("trait TestTrait"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_multiple_methods() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait MultiTrait {
				fn method1(&self,) -> i32;
				fn method2(&mut self, arg: String,) -> bool;
				fn method3(arg1: i32, arg2: f64,) -> String;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that all wrapper functions are generated
		assert!(token_string.contains("pub fn method1"));
		assert!(token_string.contains("pub fn method2"));
		assert!(token_string.contains("pub fn method3"));
		assert!(token_string.contains("BUFFER . method1"));
		assert!(token_string.contains("BUFFER . method2"));
		assert!(token_string.contains("BUFFER . method3"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_with_const() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait ConstTrait {
				const fn const_method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that const is preserved
		assert!(token_string.contains("pub const fn const_method"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_with_unsafe() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait UnsafeTrait {
				unsafe fn unsafe_method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that unsafe is preserved
		assert!(token_string.contains("pub unsafe fn unsafe_method"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_with_async() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait AsyncTrait {
				async fn async_method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that async is preserved
		assert!(token_string.contains("pub async fn async_method"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_with_generics() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait GenericTrait {
				fn generic_method<T,>(&self, arg: T,) -> T;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that generics are preserved (format may vary)
		assert!(token_string.contains("generic_method"));
		assert!(token_string.contains("< T"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_with_return_type() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait ReturnTrait {
				fn return_method(&self,) -> Result<String, Error,>;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that return type is preserved (format may vary)
		assert!(token_string.contains("return_method"));
		assert!(token_string.contains("Result"));
		assert!(token_string.contains("String"));
		assert!(token_string.contains("Error"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_filters_non_functions() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait MixedTrait {
				type AssocType;
				const CONST_VAL: i32;
				fn method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that only function gets wrapper, but trait is preserved
		assert!(token_string.contains("pub fn method"));
		assert!(token_string.contains("type AssocType"));
		assert!(token_string.contains("const CONST_VAL"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_empty_trait() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait EmptyTrait {
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that trait is preserved even if empty
		assert!(token_string.contains("trait EmptyTrait"));
		assert!(diags.is_empty());
	}

	#[test]
	fn test_wrapper_function_with_where_clause() {
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait WhereTrait {
				fn where_method<T,>(&self, arg: T,) -> T where T: Clone;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.is_ok());

		let (tokens, diags,) = result.unwrap();
		let token_string = tokens.to_string();

		// Check that where clause is preserved (though it might be formatted
		// differently)
		assert!(token_string.contains("pub fn where_method"));
		assert!(token_string.contains("Clone"));
		assert!(diags.is_empty());
	}
}
