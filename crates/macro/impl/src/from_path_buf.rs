use {
	poison_girl_dev_error::{InvalidManifest, poison_girl_err},
	poison_girl_dev_fs::{CARGO_MANIFEST, all_crates, read_toml},
	poison_girl_dev_util::CaseConvert,
	poison_girl_proc_macro_helper::{diagnostic::Diag, rslt::Rslt},
	proc_macro2::TokenStream,
	quote::format_ident,
	std::path::Path,
};

pub fn from_path_buf(item: syn::DeriveInput,) -> Rslt<TokenStream,>
{
	match item.data {
		syn::Data::Struct(_,) => struct_impl(item,),
		_ => Rslt::new_err(format!("expected struct, found {item:?}"),),
	}
}

fn struct_impl(mut struct_def: syn::DeriveInput,) -> Rslt<TokenStream,>
{
	trim_name(&mut struct_def,);

	// let enum_parts = enum_parts(&struct_def,)??;
	// let enum_name = enum_parts.name.clone();
	// let enum_dumped = enum_parts.dump();
	// let struct_dumped = struct_dump(struct_def, enum_name,)?;
	//
	// Rslt::new(quote::quote! {
	// 	#enum_dumped
	// 	#struct_dumped
	// },)
	enum_parts(&struct_def,).replace_by(|enum_def| {
		struct_dump(struct_def.clone(), enum_def.name.clone(),).replace_by(
			|struct_def| {
				let enum_def = enum_def.dump();
				Rslt::new(quote::quote! {
					#enum_def
					#struct_def
				},)
			},
		)
	},)
}

fn trim_name(struct_def: &mut syn::DeriveInput,)
{
	let mut name = struct_def.ident.to_string();
	name.remove_matches('_',);
	struct_def.ident = format_ident!("{name}");
}

struct EnumParts
{
	name:          Option<syn::Type,>,
	variants:      Vec<proc_macro2::TokenStream,>,
	variants_attr: Vec<Option<proc_macro2::TokenStream,>,>,
	paths:         Vec<proc_macro2::TokenStream,>,
}

impl EnumParts
{
	pub fn dump(&self,) -> proc_macro2::TokenStream
	{
		let name = &self.name;
		let variants = &self.variants;
		let variants_attr = &self.variants_attr;
		let paths = &self.paths;

		quote::quote! {
			#[derive(Default, PartialEq, Eq, Clone, Debug)]
			pub enum #name {
				#(
					#variants_attr
					#variants,
				)*
			}

			impl #name {
				pub fn to_path_buf(&self) -> PathBuf {
					use std::str::FromStr;
					match self {
						#(Self::#variants => PathBuf::from_str(#paths).unwrap(),)*
					}
				}
			}

			impl From<PathBuf,> for #name {
				fn from(value: PathBuf,) -> Self {
					let value = value.to_str().expect("failed to convert PathBuf to &str");
					match value {
						#(#paths => Self::#variants,)*
						a => unreachable!("invalid path {a:#?}"),
					}
				}
			}
		}
	}
}

fn enum_parts(struct_def: &syn::DeriveInput,) -> Rslt<EnumParts,>
{
	let name = detect_chart_type(struct_def,);

	let crate_list = all_crates()?;
	crate_list
		.iter()
		.enumerate()
		.map(|(i, pb,)| {
			let path = pb.to_str().ok_or("failed convert PathBuf to &str",)?;
			let path = quote::quote! {#path};
			extract_variant_name(pb,).replace_by(|name| {
				let variant = format_ident!("{name}");
				let variant = quote::quote! {
					#variant
				};

				let attr = if i == 0 {
					Some(quote::quote! {
						#[default]
					},)
				} else {
					None
				};
				Rslt::new((variant, attr, path,),)
			},)
		},)
		.fold(Rslt::new(vec![],), |acc, item| acc.push_elem(item,),)
		.replace_by(|val| {
			let len = val.len();
			let mut variants = Vec::with_capacity(len,);
			let mut variants_attr = Vec::with_capacity(len,);
			let mut paths = Vec::with_capacity(len,);
			val.into_iter().for_each(|(v, a, p,)| {
				variants.push(v,);
				variants_attr.push(a,);
				paths.push(p,);
			},);
			Rslt::new(EnumParts {
				name,
				variants: variants.clone(),
				variants_attr,
				paths,
			},)
			.with_diags(
				variants
					.iter()
					.map(|v| Diag::help(format!("{v:?}"),),)
					.collect(),
			)
		},)
}

fn extract_variant_name(p: impl AsRef<Path,>,) -> Rslt<String,>
{
	let manifest = p.as_ref().join(CARGO_MANIFEST,);
	let manifest = read_toml(manifest,)?;
	let toml::Value::String(package_name,) = manifest
		.get("package",)
		.ok_or(InvalidManifest::new("package",),)?
		.get("name",)
		.ok_or(InvalidManifest::new("name",),)?
	else {
		return Rslt::new_err(poison_girl_err!(InvalidManifest::new(format!(
			"{manifest:?}"
		))),);
	};

	// 頭の`poison_girl_`部分は長ったらしいので除く
	let name = if package_name != "poison_girl" {
		package_name.split("poison_girl_",).nth(1,).unwrap()
	} else {
		package_name
	}
	.to_string()
	.to_camel();
	Rslt::new(name,)
}

fn detect_chart_type(struct_def: &syn::DeriveInput,) -> Option<syn::Type,>
{
	let syn::Data::Struct(syn::DataStruct { fields, .. },) = &struct_def.data
	else {
		panic!("expected struct, found {struct_def:?}")
	};
	fields.iter().find(|f| {
		f.attrs.iter().any(
			|attr| matches!(attr.meta, syn::Meta::Path(ref p) if p.get_ident() == Some(&format_ident!("chart"))),
		)
	},).map(|f| f.ty.clone())
}

fn struct_dump(
	mut struct_def: syn::DeriveInput,
	enum_name: Option<syn::Type,>,
) -> Rslt<proc_macro2::TokenStream,>
{
	let syn::Data::Struct(syn::DataStruct { ref mut fields, .. },) =
		struct_def.data
	else {
		return Rslt::new_err(
			"unexpected derive input. this macro only support struct derive",
		);
	};

	let fields = fields_invest(&enum_name, fields,)??;

	let ident = &struct_def.ident;
	let generics = &struct_def.generics;

	Rslt::new(quote::quote! {
		// #struct_def

		impl #generics From<PathBuf> for #ident #generics {
			fn from(value: PathBuf,) -> Self {
				Self {
					#(#fields,)*
				}
			}
		}

	},)
}

fn fields_invest(
	enum_name: &Option<syn::Type,>,
	fields: &mut syn::Fields,
) -> Rslt<Vec<proc_macro2::TokenStream,>,>
{
	match fields {
		syn::Fields::Named(syn::FieldsNamed { named: f, .. },)
		| syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed: f, .. },) => f
			.iter_mut()
			.map(|f| {
				let is_attred = is_attred(f,);
				if is_attred {
					f.ty = syn::parse_quote! {
						#enum_name
					};
					f.attrs = vec![];
				}

				field_construct(enum_name, f.clone(),)
			},)
			.fold(Rslt::new(vec![],), |acc, field| acc.push_elem(field,),),
		syn::Fields::Unit => unreachable!(),
	}
}

fn field_construct(
	enum_name: &Option<syn::Type,>,
	f: syn::Field,
) -> Rslt<proc_macro2::TokenStream,>
{
	let construct = match f.ty {
		syn::Type::Path(syn::TypePath {
			path: syn::Path { segments, .. },
			..
		},) => {
			let field_name = &f.ident;
			let id = if let Some(field_name,) = field_name {
				quote::quote! {
					#field_name
				}
			} else {
				quote::quote! {}
			};

			if let Some(last,) = segments.last() {
				let Some(syn::Type::Path(syn::TypePath {
					path: syn::Path { segments, .. },
					..
				},),) = enum_name
				else {
					unimplemented!()
				};
				let enum_last = &segments.last().unwrap().ident;

				if last.ident == "PathBuf" {
					quote::quote! {
						#id: value.clone()
					}
				} else if &last.ident == enum_last {
					quote::quote! {
						#id: #enum_name::from(value.clone())
					}
				} else {
					quote::quote! {
						#id:
					}
				}
			} else {
				return Rslt::new_err("invalid type",);
			}
		},
		a => unimplemented!("type {a:#?} not supported"),
	};

	Rslt::new(construct,)
}

fn is_attred(f: &mut syn::Field,) -> bool
{
	f.attrs
		.iter()
		.any(|a| matches!(&a.meta, syn::Meta::Path(p) if p.is_ident("chart")),)
}

#[cfg(test)]
mod tests
{
	use {super::*, itertools::Itertools, quote::quote, syn::parse_quote};

	#[test]
	fn test_from_path_buf_with_enum()
	{
		// Create a test enum as DeriveInput
		let test_enum: syn::DeriveInput = parse_quote! {
			pub enum TestCrate {
				OsoKernel,
				OsoBootloader,
			}
		};

		// from_path_buf expects a struct, so this should return an error
		let result = from_path_buf(test_enum,);

		// Should return an error since from_path_buf expects a struct
		assert!(result.has_err());
	}

	#[test]
	fn test_camel_case_conversion_logic()
	{
		// Test the camel case conversion logic used in enum_impl
		let test_name = "oso_kernel_test";
		let camel_cased = test_name
			.split('_',)
			.map(|s| s[..1].to_uppercase() + &s[1..],)
			.join("",);

		assert_eq!(camel_cased, "OsoKernelTest");
	}

	#[test]
	fn test_camel_case_single_word()
	{
		let test_name = "kernel";
		let camel_cased = test_name
			.split('_',)
			.map(|s| s[..1].to_uppercase() + &s[1..],)
			.join("",);

		assert_eq!(camel_cased, "Kernel");
	}

	#[test]
	fn test_camel_case_empty_parts()
	{
		let test_name = "oso__kernel"; // Double underscore
		let camel_cased = test_name
			.split('_',)
			.map(|s| {
				if s.is_empty() {
					String::new()
				} else {
					s[..1].to_uppercase() + &s[1..]
				}
			},)
			.join("",);

		assert_eq!(camel_cased, "OsoKernel");
	}

	#[test]
	fn test_path_string_conversion()
	{
		use std::path::PathBuf;

		// Test that PathBuf can be converted to string
		let path = PathBuf::from("/test/path",);
		let path_str = path.to_str();

		assert!(path_str.is_some());
		assert_eq!(path_str.unwrap(), "/test/path");
	}

	#[test]
	fn test_path_with_non_utf8_handling()
	{
		use std::path::PathBuf;

		// Create a path that might have UTF-8 issues
		let path = PathBuf::from("test_path",);
		let path_str = path.to_str();

		// For normal paths, this should work
		assert!(path_str.is_some());
	}

	#[test]
	fn test_quote_format_ident_functionality()
	{
		// Test that quote::format_ident works as expected
		let ident_name = "TestVariant";
		let ident = quote::format_ident!("{}", ident_name);

		let token_string = quote! { #ident }.to_string();
		assert!(token_string.contains("TestVariant"));
	}

	#[test]
	fn test_token_stream_generation()
	{
		// Test that we can generate basic token streams
		let test_tokens = quote! {
			impl From<PathBuf> for TestEnum {
				fn from(value: PathBuf) -> Self {
					match value.to_str().unwrap() {
						"/test/path" => Self::TestVariant,
					}
				}
			}
		};

		let token_string = test_tokens.to_string();
		assert!(token_string.contains("impl From"));
		assert!(token_string.contains("PathBuf"));
		assert!(token_string.contains("TestEnum"));
		assert!(token_string.contains("fn from"));
		assert!(token_string.contains("match"));
	}

	#[test]
	fn test_itertools_join_functionality()
	{
		// Test that itertools join works as expected
		let parts = ["Hello", "World", "Test",];
		let joined = parts.iter().map(|s| s.to_string(),).join("",);

		assert_eq!(joined, "HelloWorldTest");
	}

	#[test]
	fn test_itertools_join_with_separator()
	{
		let parts = ["Hello", "World",];
		let joined = parts.iter().map(|s| s.to_string(),).join("_",);

		assert_eq!(joined, "Hello_World");
	}

	#[test]
	fn test_syn_item_matching()
	{
		// Test that syn::Item matching works correctly
		let enum_item: syn::Item = syn::parse_quote! {
			enum TestEnum { A, B }
		};

		let struct_item: syn::Item = syn::parse_quote! {
			struct TestStruct;
		};

		let fn_item: syn::Item = syn::parse_quote! {
			fn test_fn() {}
		};

		// Test pattern matching
		match enum_item {
			syn::Item::Enum(_,) => (),
			_ => panic!("Should match enum"),
		}

		match struct_item {
			syn::Item::Struct(_,) => (),
			_ => panic!("Should match struct"),
		}

		match fn_item {
			syn::Item::Fn(_,) => (),
			_ => panic!("Should match function"),
		}
	}
}
