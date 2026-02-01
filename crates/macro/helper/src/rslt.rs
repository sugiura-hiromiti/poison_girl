use {
	crate::diagnostic::{Diag, ErrDiag, NotationDiag},
	poison_girl_dev_error::PoisonGirlB,
	std::{
		convert::Infallible,
		fmt::Debug,
		ops::{FromResidual, Try},
		process::Termination,
	},
};

pub struct Rslt<V,>
{
	val:      Option<V,>,
	notation: Vec<NotationDiag,>,
	err:      Option<ErrDiag,>,
}

impl<V,> Default for Rslt<V,>
{
	fn default() -> Self
	{
		Self {
			val:      Default::default(),
			notation: Default::default(),
			err:      Default::default(),
		}
	}
}

impl<V,> Rslt<V,>
{
	pub fn new(val: V,) -> Self
	{
		Self { val: Some(val,), notation: vec![], err: None, }
	}

	pub fn new_err(e: impl Debug,) -> Self
	{
		Self {
			val:      None,
			notation: vec![],
			err:      Some(ErrDiag::new(format!("{e:?}"),),),
		}
	}

	pub fn with_err(mut self, err: ErrDiag,) -> Self
	{
		self.err = Some(err,);
		self
	}

	pub fn inject_err(mut self, err: Option<ErrDiag,>,) -> Self
	{
		if !self.has_err() && err.is_some() {
			self.err = err;
		}
		self
	}

	pub fn add_notation(mut self, nt: NotationDiag,) -> Self
	{
		self.notation.push(nt,);
		self
	}

	pub fn add_notations(mut self, mut nts: Vec<NotationDiag,>,) -> Self
	{
		self.notation.append(&mut nts,);
		self
	}

	pub fn with_diag(self, diag: impl Into<Diag,>,) -> Self
	{
		match diag.into() {
			Diag::Err(err_diag,) => self.with_err(err_diag,),
			Diag::Notation(notation_diag,) => self.add_notation(notation_diag,),
		}
	}

	pub fn with_diags(self, diags: Vec<impl Into<Diag,>,>,) -> Self
	{
		diags.into_iter().fold(self, |acc, diag| acc.with_diag(diag,),)
	}

	pub fn has_err(&self,) -> bool
	{
		self.err.is_some()
	}

	pub fn err(&self,) -> Option<&ErrDiag,>
	{
		self.err.as_ref()
	}

	pub fn into_err(self,) -> Option<ErrDiag,>
	{
		self.err
	}

	pub fn value(&self,) -> Option<&V,>
	{
		self.val.as_ref()
	}

	pub fn value_mut(&mut self,) -> Option<&mut V,>
	{
		self.val.as_mut()
	}

	pub fn notation(&self,) -> &[NotationDiag]
	{
		&self.notation
	}

	pub fn into_value(self,) -> Option<V,>
	{
		self.val
	}

	pub fn into_notation(self,) -> Vec<NotationDiag,>
	{
		self.notation
	}

	pub fn unwrap(self,) -> Option<V,>
	{
		match self.err {
			Some(e,) => panic!("Error Diagnostic: {e:?}"),
			None => self.into_value(),
		}
	}

	pub fn replace<V2,>(self, val: V2,) -> Rslt<V2,>
	{
		let Self { notation, err, .. } = self;
		Rslt { val: Some(val,), notation, err, }
	}

	pub fn replace_by<V2,>(self, f: impl FnOnce(V,) -> Rslt<V2,>,)
	-> Rslt<V2,>
	{
		let Self { val, notation, err, } = self;
		match val {
			Some(v,) => {
				let new = f(v,).add_notations(notation,);
				match (new.has_err(), err,) {
					(false, Some(e,),) => new.with_err(e,),
					_ => new,
				}
			},
			None => Rslt { val: None, notation, err, },
		}
	}
}

impl<V,> Rslt<Vec<V,>,>
{
	pub fn add(self, one: Rslt<V,>,) -> Self
	{
		let Rslt { val, notation, err, } = one;
		let Rslt { val: val2, notation, err, } =
			self.inject_err(err,).add_notations(notation,);

		let val = match (val, val2,) {
			(None, v,) => v,
			(Some(v,), None,) => Some(vec![v],),
			(Some(val,), Some(mut vval,),) => {
				vval.push(val,);
				Some(vval,)
			},
		};

		Self { val, notation, err, }
	}
}

impl<V,> Try for Rslt<V,>
{
	type Output = Option<V,>;
	type Residual = Rslt<Infallible,>;

	fn from_output(output: Self::Output,) -> Self
	{
		Self { val: output, notation: vec![], err: None, }
	}

	fn branch(self,) -> std::ops::ControlFlow<Self::Residual, Self::Output,>
	{
		if self.has_err() {
			let Self { notation, err, .. } = self;
			std::ops::ControlFlow::Break(Rslt { val: None, notation, err, },)
		} else {
			std::ops::ControlFlow::Continue(self.unwrap(),)
		}
	}
}

impl<V,> FromResidual for Rslt<V,>
{
	fn from_residual(residual: <Self as Try>::Residual,) -> Self
	{
		let Rslt { val, notation, err, } = residual;
		match val {
			Some(_,) => unreachable!(),
			None => Self { val: None, notation, err, },
		}
	}
}

impl<V,> FromResidual<PoisonGirlB<Infallible,>,> for Rslt<V,>
{
	fn from_residual(residual: PoisonGirlB<Infallible,>,) -> Self
	{
		match residual {
			poison_girl_dev_error::X(_,) => unreachable!(),
			poison_girl_dev_error::Y(e,) => Rslt::new_err(e,),
		}
	}
}

impl<V, E: Debug,> FromResidual<Result<Infallible, E,>,> for Rslt<V,>
{
	fn from_residual(residual: Result<Infallible, E,>,) -> Self
	{
		match residual {
			Ok(_,) => unreachable!(),
			Err(e,) => Rslt::new_err(e,),
		}
	}
}

impl<V,> FromResidual<Option<Infallible,>,> for Rslt<V,>
{
	fn from_residual(residual: Option<Infallible,>,) -> Self
	{
		match residual {
			Some(_,) => unreachable!(),
			None => Rslt::new_err("option is none",),
		}
	}
}

impl<V,> Termination for Rslt<V,>
{
	fn report(self,) -> std::process::ExitCode
	{
		if self.has_err() {
			std::process::ExitCode::FAILURE
		} else {
			std::process::ExitCode::SUCCESS
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_proc_macro2_token_stream_operations()
	{
		// Test basic proc_macro2::TokenStream operations
		let tokens1 = quote::quote! { fn test1() {} };
		let tokens2 = quote::quote! { fn test2() {} };

		// Test combining token streams
		let combined = quote::quote! {
			#tokens1
			#tokens2
		};

		let combined_str = combined.to_string();
		assert!(combined_str.contains("test1"));
		assert!(combined_str.contains("test2"));
	}

	#[test]
	fn test_quote_macro_functionality()
	{
		// Test various quote! macro patterns
		let ident =
			syn::Ident::new("TestStruct", proc_macro2::Span::call_site(),);
		let ty: syn::Type = syn::parse_quote! { i32 };

		let tokens = quote::quote! {
			struct #ident {
				field: #ty,
			}
		};

		let token_str = tokens.to_string();
		assert!(token_str.contains("struct TestStruct"));
		assert!(token_str.contains("field : i32"));
	}

	#[test]
	fn test_syn_parsing_functionality()
	{
		// Test syn parsing capabilities
		let input = "fn test(arg: i32) -> bool { true }";
		let parsed: syn::ItemFn =
			syn::parse_str(input,).expect("Failed to parse function",);

		assert_eq!(parsed.sig.ident.to_string(), "test");
		assert_eq!(parsed.sig.inputs.len(), 1);
	}

	#[test]
	fn test_itertools_functionality()
	{
		// Test itertools features used in the crate
		use itertools::Itertools;

		let items = vec!["a", "b", "c"];
		let joined = items.iter().join(", ",);
		assert_eq!(joined, "a, b, c");

		let chunks: Vec<Vec<&str,>,> = items
			.iter()
			.chunks(2,)
			.into_iter()
			.map(|chunk| chunk.cloned().collect(),)
			.collect();
		assert_eq!(chunks.len(), 2);
		assert_eq!(chunks[0], vec!["a", "b"]);
		assert_eq!(chunks[1], vec!["c"]);
	}

	#[test]
	fn test_colored_output_functionality()
	{
		// Test that colored output doesn't panic (even if colors aren't visible
		// in tests)
		use colored::Colorize;

		let colored_text = "Test message".red().bold();
		let colored_str = colored_text.to_string();

		// The exact output depends on terminal support, but it shouldn't panic
		assert!(!colored_str.is_empty());
	}

	#[test]
	fn test_multiple_module_interaction()
	{
		// Test that modules can work together without conflicts

		// Create some diagnostics
		let diags = vec![
			Diag::err("Error from module interaction",),
			Diag::warn("Warning from module interaction",),
		];

		// Test that we can create a result with diagnostics
		let result =
			Rslt::new(quote::quote! { fn test() {} },).with_diags(diags,);
		assert!(!result.has_err());
		assert_eq!(result.notation().len(), 2);

		let tokens = result.unwrap();
		assert!(!tokens.unwrap().is_empty());
	}

	#[test]
	fn test_unstable_features_compilation()
	{
		// Test that unstable features compile correctly

		// Test str_as_str feature (if used)
		let test_str = "test";
		let _str_slice = test_str.as_str();

		// Test iter_array_chunks feature (if used)
		let items = [1, 2, 3, 4, 5, 6,];
		let _chunks: Vec<[i32; 2],> =
			items.iter().array_chunks().map(|[a, b,]| [*a, *b,],).collect();

		// Test iterator_try_collect feature (if used)
		let results: Vec<Result<i32, &str,>,> = vec![Ok(1,), Ok(2,), Ok(3,)];
		let _collected: Result<Vec<i32,>, &str,> =
			results.into_iter().try_collect();

		// If this compiles, the features are working
		assert!(true);
	}

	#[test]
	fn test_rslt_p_complex_scenarios()
	{
		// Test RsltP with complex token streams and multiple diagnostics

		fn complex_function() -> Rslt<proc_macro2::TokenStream,>
		{
			let complex_tokens = quote::quote! {
				pub struct ComplexStruct<T> where T: Clone + Send + Sync {
					field1: T,
					field2: Option<T>,
					field3: Vec<T>,
				}

				impl<T> ComplexStruct<T> where T: Clone + Send + Sync {
					pub fn new(value: T) -> Self {
						Self {
							field1: value.clone(),
							field2: Some(value.clone()),
							field3: vec![value],
						}
					}
				}
			};

			let complex_diags = vec![
				Diag::note("Complex structure created",),
				Diag::warn("This is a test warning",),
				Diag::help("Consider using simpler types",),
			];

			Rslt::new(complex_tokens,).with_diags(complex_diags,)
		}

		let result = complex_function();
		assert!(!result.has_err());

		assert_eq!(result.notation().len(), 3);
		let tokens = result.unwrap();
		assert!(!tokens.as_ref().unwrap().is_empty());

		// Verify token stream contains expected content
		let token_str = tokens.unwrap().to_string();
		assert!(token_str.contains("ComplexStruct"));
		assert!(token_str.contains("Clone"));
		assert!(token_str.contains("Send"));
		assert!(token_str.contains("Sync"));
	}

	#[test]
	fn test_proc_macro2_advanced_features()
	{
		// Test advanced proc_macro2 features
		use proc_macro2::{
			Delimiter, Group, Ident, Literal, Punct, Spacing, Span,
		};

		// Test creating various token types
		let ident = Ident::new("test_ident", Span::call_site(),);
		let _literal = Literal::string("test string",);
		let _punct = Punct::new(':', Spacing::Alone,);

		// Test creating groups
		let tokens = quote::quote! { field: "value" };
		let group = Group::new(Delimiter::Brace, tokens,);

		// Test combining into a token stream
		let combined = quote::quote! {
			struct #ident {
				#group
			}
		};

		let combined_str = combined.to_string();
		assert!(combined_str.contains("test_ident"));
		assert!(combined_str.contains("field"));
		assert!(combined_str.contains("value"));
	}

	#[test]
	fn test_syn_advanced_parsing()
	{
		// Test advanced syn parsing capabilities

		// Test parsing complex function signatures
		let complex_fn = "pub async unsafe fn complex_function<T: Clone + \
		                  Send>(
			arg1: &mut T,
			arg2: Option<Vec<T>>,
			arg3: impl Iterator<Item = T>
		) -> Result<Box<dyn Iterator<Item = T>>, Box<dyn std::error::Error + Send + \
		                  Sync>>
		where
			T: 'static + Clone + Send + Sync
		{
			todo!()
		}";

		let parsed: syn::ItemFn = syn::parse_str(complex_fn,)
			.expect("Failed to parse complex function",);

		assert_eq!(parsed.sig.ident.to_string(), "complex_function");
		assert!(parsed.sig.asyncness.is_some());
		assert!(parsed.sig.unsafety.is_some());
		assert_eq!(parsed.sig.inputs.len(), 3);
		assert!(parsed.sig.generics.params.len() > 0);

		// Test parsing complex types
		let complex_type = "Result<Box<dyn Iterator<Item = T>>, Box<dyn \
		                    std::error::Error + Send + Sync>>";
		let parsed_type: syn::Type = syn::parse_str(complex_type,)
			.expect("Failed to parse complex type",);

		match parsed_type {
			syn::Type::Path(_,) => assert!(true),
			_ => panic!("Expected path type"),
		}
	}

	#[test]
	fn test_quote_macro_edge_cases()
	{
		// Test quote! macro with various edge cases

		// Test with empty content
		let empty = quote::quote! {};
		assert!(empty.is_empty());

		// Test with repetition
		let items = vec!["a", "b", "c"];
		let repeated = quote::quote! {
			vec![#(#items),*]
		};
		let repeated_str = repeated.to_string();
		assert!(repeated_str.contains("vec"));
		assert!(repeated_str.contains("a"));
		assert!(repeated_str.contains("b"));
		assert!(repeated_str.contains("c"));

		// Test with conditional compilation
		let conditional = quote::quote! {
			#[cfg(test)]
			mod test_module {
				#[test]
				fn test_function() {}
			}
		};
		let conditional_str = conditional.to_string();
		assert!(conditional_str.contains("cfg"));
		assert!(conditional_str.contains("test"));
		assert!(conditional_str.contains("test_module"));

		// Test with nested quotes
		let nested = quote::quote! {
			macro_rules! test_macro {
				() => {
					quote::quote! { fn generated() {} }
				};
			}
		};
		let nested_str = nested.to_string();
		assert!(nested_str.contains("macro_rules"));
		assert!(nested_str.contains("test_macro"));
	}

	#[test]
	fn test_concurrent_operations()
	{
		// Test that our types work correctly in concurrent scenarios
		use std::{
			sync::{Arc, Mutex},
			thread,
		};

		let counter = Arc::new(Mutex::new(0,),);
		let mut handles = vec![];

		for _ in 0..5 {
			let counter = Arc::clone(&counter,);
			let handle = thread::spawn(move || {
				// Test that our Result type works in threads
				let result = Rslt::new(42,);
				assert!(!result.has_err());

				let mut num = counter.lock().unwrap();
				*num += 1;
			},);
			handles.push(handle,);
		}

		for handle in handles {
			handle.join().unwrap();
		}

		assert_eq!(*counter.lock().unwrap(), 5);
	}
}
