use {
	poison_girl_proc_macro_helper::rslt::Rslt, proc_macro2::TokenStream,
	syn::Signature,
};

pub fn wrapper(
	static_frame_buffer: syn::Ident,
	trait_def: syn::ItemTrait,
) -> Rslt<TokenStream,>
{
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
	Rslt::new(wrapper_fns,)
}

pub fn method_args(
	sig: &Signature,
) -> impl Iterator<Item = std::boxed::Box<syn::Pat,>,>
{
	sig.inputs.iter().filter_map(|a| match a {
		// Skip receiver arguments (self, &self, &mut self, etc.)
		syn::FnArg::Receiver(_,) => None,

		// Extract the pattern from typed arguments
		syn::FnArg::Typed(pty,) => Some(pty.pat.clone(),),
	},)
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, ok},
		syn::{Signature, parse_quote},
	};

	#[test]
	fn test_method_args_no_receiver()
	{
		let sig: Signature = parse_quote! {
			fn test_function(arg1: i32, arg2: String, arg3: bool) -> i32
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		assert_eq!(args.len(), 3);
	}

	#[test]
	fn test_method_args_with_mut_self_receiver()
	{
		let sig: Signature = parse_quote! {
			fn test_method(&mut self, arg1: i32) -> ()
		};

		let args: Vec<_,> = method_args(&sig,).collect();
		// Should exclude &mut self, only return the 1 typed argument
		assert_eq!(args.len(), 1);
	}

	#[test]
	fn test_wrapper_function_basic() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("FRAME_BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait TestTrait {
				fn test_method(&self, arg: i32,) -> bool;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(!result.has_err());

		assert!(result.notation().is_empty());
		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that wrapper function is generated
		assert!(token_string.contains("pub fn test_method"));
		assert!(token_string.contains("FRAME_BUFFER . test_method"));
		assert!(token_string.contains("trait TestTrait"));

		ok!()
	}

	#[test]
	fn test_wrapper_function_multiple_methods() -> PoisonGirlTestB
	{
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
		assert!(!result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that all wrapper functions are generated
		assert!(token_string.contains("pub fn method1"));
		assert!(token_string.contains("pub fn method2"));
		assert!(token_string.contains("pub fn method3"));
		assert!(token_string.contains("BUFFER . method1"));
		assert!(token_string.contains("BUFFER . method2"));
		assert!(token_string.contains("BUFFER . method3"));
		ok!()
	}

	#[test]
	fn test_wrapper_function_with_const()
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait ConstTrait {
				const fn const_method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that const is preserved
		assert!(token_string.contains("pub const fn const_method"));
	}

	#[test]
	fn test_wrapper_function_with_unsafe() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait UnsafeTrait {
				unsafe fn unsafe_method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that unsafe is preserved
		assert!(token_string.contains("pub unsafe fn unsafe_method"));

		ok!()
	}

	#[test]
	fn test_wrapper_function_with_async() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait AsyncTrait {
				async fn async_method(&self,) -> i32;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that async is preserved
		assert!(token_string.contains("pub async fn async_method"));
		ok!()
	}

	#[test]
	fn test_wrapper_function_with_generics() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait GenericTrait {
				fn generic_method<T,>(&self, arg: T,) -> T;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that generics are preserved (format may vary)
		assert!(token_string.contains("generic_method"));
		assert!(token_string.contains("< T"));
		ok!()
	}

	#[test]
	fn test_wrapper_function_with_return_type() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait ReturnTrait {
				fn return_method(&self,) -> Result<String, Error,>;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that return type is preserved (format may vary)
		assert!(token_string.contains("return_method"));
		assert!(token_string.contains("Result"));
		assert!(token_string.contains("String"));
		assert!(token_string.contains("Error"));
		ok!()
	}

	#[test]
	fn test_wrapper_function_filters_non_functions() -> PoisonGirlTestB
	{
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
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that only function gets wrapper, but trait is preserved
		assert!(token_string.contains("pub fn method"));
		assert!(token_string.contains("type AssocType"));
		assert!(token_string.contains("const CONST_VAL"));
		ok!()
	}

	#[test]
	fn test_wrapper_function_empty_trait() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait EmptyTrait {
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that trait is preserved even if empty
		assert!(token_string.contains("trait EmptyTrait"));
		ok!()
	}

	#[test]
	fn test_wrapper_function_with_where_clause() -> PoisonGirlTestB
	{
		let static_frame_buffer =
			syn::Ident::new("BUFFER", proc_macro2::Span::call_site(),);
		let trait_def: syn::ItemTrait = parse_quote! {
			trait WhereTrait {
				fn where_method<T,>(&self, arg: T,) -> T where T: Clone;
			}
		};

		let result = wrapper(static_frame_buffer, trait_def,);
		assert!(result.has_err());
		assert!(result.notation().is_empty());

		let tokens = result?;
		let token_string = tokens.to_string();

		// Check that where clause is preserved (though it might be formatted
		// differently)
		assert!(token_string.contains("pub fn where_method"));
		assert!(token_string.contains("Clone"));
		ok!()
	}
}
