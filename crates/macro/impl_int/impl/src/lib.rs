use {
	poison_girl_macro_error::rslt::Rslt,
	proc_macro2::TokenTree,
	syn::{TypePath, parse::Parse, spanned::Spanned},
};

pub struct Types
{
	/// Internal storage for the parsed types
	type_list: Vec<syn::Type,>,
}

impl Types
{
	pub fn iter(&self,) -> std::slice::Iter<'_, syn::Type,>
	{
		self.type_list.iter()
	}
}

impl Parse for Types
{
	fn parse(input: syn::parse::ParseStream,) -> syn::Result<Self,>
	{
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

pub fn impl_int(types: Types,) -> Rslt<proc_macro2::TokenStream,>
{
	let integers = types.iter().map(implement,);

	Rslt::new(quote::quote! {
		#(#integers)*
	},)
}

pub fn implement(ty: &syn::Type,) -> Rslt<proc_macro2::TokenStream,>
{
	let idnt = unwrap_primitive(ty,)?;
	let digit_count = digit_count_impl();
	let nth_digit = nth_digit_impl();
	let shift_right = shift_right_impl(&idnt,);

	Rslt::new(quote::quote! {
		impl Integer for #idnt {
			#digit_count
			#nth_digit
			#shift_right
		}
	},)
}

fn unwrap_primitive(ty: &syn::Type,) -> Rslt<syn::Ident,>
{
	// Extract segment as `seg` from `ty`
	let syn::Type::Path(TypePath {
		qself: None,
		path: syn::Path { leading_colon: None, segments: seg, },
	},) = ty
	else {
		return Rslt::new_err(syn::Error::new(
			ty.span(),
			format!("unable to unwrap type: {ty:#?}"),
		),);
	};

	if seg.len() != 1 {
		return Rslt::new_err(syn::Error::new(
			ty.span(),
			format!(
				"type may not primitive: {ty:#?}. if not, remove leading path"
			),
		),);
	}

	// Extract ident of type from `seg`
	let syn::PathSegment { ident: idnt, arguments: syn::PathArguments::None, } =
		seg.first()?
	else {
		return Rslt::new_err(syn::Error::new(
			seg.span(),
			format!("unable to unwrap path segment: {seg:#?}"),
		),);
	};

	Rslt::new(idnt.clone(),)
}

fn digit_count_impl() -> proc_macro2::TokenStream
{
	quote::quote! {
		/// 1 indexed
		fn digit_count(&self) -> usize {
			let mut n = self.clone();
			if n == 0 {
				return 1;
			}

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

fn nth_digit_impl() -> proc_macro2::TokenStream
{
	quote::quote! {
		fn nth_digit(&self, n: usize) -> u8 {
			let mut origin = self.clone();

			// Shift right n times to get the desired digit in the ones place
			for _i in 0..n {
				origin.shift_right();
			}

			// Extract the ones digit
			origin.shift_right()
		}
	}
}

fn shift_right_impl(idnt: &syn::Ident,) -> proc_macro2::TokenStream
{
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
mod tests
{
	use {
		super::*,
		poison_girl_macro_error::{rslt::test_helper::TestRslt, success},
		quote::quote,
		syn::{Type, parse_quote},
	};

	#[test]
	fn test_unwrap_primitive_u32() -> TestRslt
	{
		let ty: Type = parse_quote! { u32 };
		let ident = unwrap_primitive(&ty,)?;

		assert_eq!(ident.to_string(), "u32");
		success!()
	}

	#[test]
	fn test_unwrap_primitive_i64() -> TestRslt
	{
		let ty: Type = parse_quote! { i64 };
		let ident = unwrap_primitive(&ty,)?;

		assert_eq!(ident.to_string(), "i64");
		success!()
	}

	#[test]
	fn test_unwrap_primitive_usize() -> TestRslt
	{
		let ty: Type = parse_quote! { usize };
		let ident = unwrap_primitive(&ty,)?;

		assert_eq!(ident.to_string(), "usize");
		success!()
	}

	#[test]
	fn test_unwrap_primitive_error_on_generic()
	{
		let ty: Type = parse_quote! { Vec<i32> };
		let result = unwrap_primitive(&ty,);

		assert!(result.has_err());
	}

	#[test]
	fn test_unwrap_primitive_error_on_path()
	{
		let ty: Type = parse_quote! { std::collections::HashMap };
		let result = unwrap_primitive(&ty,);

		assert!(result.has_err());
	}

	#[test]
	fn test_unwrap_primitive_error_on_reference()
	{
		let ty: Type = parse_quote! { &str };
		let result = unwrap_primitive(&ty,);

		assert!(result.has_err());
	}

	#[test]
	fn test_complete_workflow() -> TestRslt
	{
		// Test the complete workflow from parsing to implementation
		let input = quote! { u8, i16, u32 };
		let types: Types = syn::parse2(input,)?;

		let implementations: Vec<_,> = types.iter().map(implement,).collect();

		assert_eq!(implementations.len(), 3);

		// Check that each implementation is valid
		for impl_tokens in implementations.into_iter() {
			let code_str = impl_tokens?.to_string();
			assert!(code_str.contains("impl Integer for"));
			assert!(code_str.contains("fn digit_count"));
			assert!(code_str.contains("fn nth_digit"));
			assert!(code_str.contains("fn shift_right"));
		}
		success!()
	}
}
