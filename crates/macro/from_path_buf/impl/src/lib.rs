#![feature(string_remove_matches)]
use {
	poison_girl_dev_error::{InvalidManifest, poison_girl_err},
	poison_girl_dev_fs::{CARGO_MANIFEST, all_crates, read_toml},
	poison_girl_dev_util::case_conversion::CaseConvert,
	poison_girl_macro_error::rslt::Rslt,
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
	bin_names:     Vec<proc_macro2::TokenStream,>,
	package_names: Vec<String,>,
}

impl EnumParts
{
	pub fn dump(&self,) -> proc_macro2::TokenStream
	{
		let Self {
			name,
			variants,
			variants_attr,
			paths,
			bin_names,
			package_names,
		} = self;

		quote::quote! {
			#[derive(Default, PartialEq, Eq, Clone, Debug, Copy)]
			pub enum #name {
				#(
					#variants_attr
					#variants,
				)*
			}

			impl #name {
				pub fn to_path_buf(&self) -> PathBuf {
					match self {
						#(Self::#variants => PathBuf::from(#paths),)*
					}
				}

				pub fn bin_name(&self)-> &str {
					match self {
						#(Self::#variants => #bin_names,)*
					}
				}

				pub fn all_variants() -> Vec<Self> {
					vec![
						#(Self::#variants,)*
					]
				}

				pub fn package_name(&self) -> &str {
					match self {
						#(Self::#variants => #package_names,)*
					}
				}
			}

			impl From<#name,> for PathBuf {
				fn from(value: #name,) -> Self {
					value.to_path_buf()
				}
			}

			impl #name {
				pub fn try_from_path_buf(value: PathBuf,) -> Result<Self, PathBuf> {
					let Some(value_str) = value.to_str() else {
						return Err(value,);
					};
					match value_str {
						#(#paths => Ok(Self::#variants,),)*
						_ => Err(value,),
					}
				}
			}

			impl From<PathBuf,> for #name {
				fn from(value: PathBuf,) -> Self {
					match Self::try_from_path_buf(value,) {
						Ok(value,) => value,
						Err(_,) => Self::default(),
					}
				}
			}
		}
	}
}

// TODO: Rstl型に?する場合, diagnosticsの蓄積はどう扱えば良いかを考える
// eg: early returnを flushとして捉え、diagnosticsをeprintln!する

fn enum_parts(struct_def: &syn::DeriveInput,) -> Rslt<EnumParts,>
{
	let name = detect_chart_type(struct_def,)?;

	let crate_list = all_crates()?;
	crate_list
		.iter()
		.enumerate()
		.map(|(i, pb,)| {
			let path = pb.to_str().ok_or("failed convert PathBuf to &str",)?;
			let path = quote::quote! {#path};
			extract_manifest(pb,).replace_by(|manifest| {
				let name = manifest.name();
				let variant = format_ident!("{name}");
				let variant = quote::quote! {
					#variant
				};
				let bin_name = manifest.bin_name();
				let bin_name = quote::quote! {
					#bin_name
				};

				let attr = if i == 0 {
					Some(quote::quote! {
						#[default]
					},)
				} else {
					None
				};
				Rslt::new((
					variant,
					attr,
					path,
					bin_name,
					manifest.package_name,
				),)
			},)
		},)
		.fold(Rslt::new(vec![],), |acc, item| acc.push_elem(item,),)
		.replace_by(|val| {
			let len = val.len();
			let mut variants = Vec::with_capacity(len,);
			let mut variants_attr = Vec::with_capacity(len,);
			let mut paths = Vec::with_capacity(len,);
			let mut bin_names = Vec::with_capacity(len,);
			let mut package_names = Vec::with_capacity(len,);

			val.into_iter().for_each(|(v, a, p, b, package_name,)| {
				variants.push(v,);
				variants_attr.push(a,);
				paths.push(p,);
				bin_names.push(b,);
				package_names.push(package_name,);
			},);
			let rslt = Rslt::new(EnumParts {
				name,
				variants,
				variants_attr,
				paths,
				bin_names,
				package_names,
			},);

			#[cfg(feature = "debug-diagnostics")]
			{
				rslt.with_diags(
					variants
						.iter()
						.map(|v| {
							poison_girl_macro_error::diagnostic::Diag::help(
								format!("{v:?}"),
							)
						},)
						.collect(),
				)
			}

			#[cfg(not(feature = "debug-diagnostics"))]
			{
				rslt
			}
		},)
}

struct Manifest
{
	/// the package name which is written in Cargo.toml
	package_name: String,
	/// the name of variant
	name:         String,
	/// the name of build artifact
	bin_name:     Option<String,>,
}

impl Manifest
{
	pub fn new(
		package_name: impl Into<String,>,
		name: impl Into<String,>,
		bin_name: Option<impl Into<String,>,>,
	) -> Self
	{
		Self {
			package_name: package_name.into(),
			name:         name.into(),
			bin_name:     bin_name.map(|bin_name| bin_name.into(),),
		}
	}

	pub fn name(&self,) -> String
	{
		self.name.clone()
	}

	pub fn bin_name(&self,) -> String
	{
		match &self.bin_name {
			Some(n,) => n.clone(),
			None => self.package_name.clone(),
		}
	}
}

fn extract_manifest(p: impl AsRef<Path,>,) -> Rslt<Manifest,>
{
	let manifest = p.as_ref().join(CARGO_MANIFEST,);
	let manifest = read_toml(manifest,)??;

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

	let bin_name = manifest
		.get("bin",)
		.and_then(toml::Value::as_array,)
		.into_iter()
		.flatten()
		.next()
		.map(|v| {
			v.get("name",)
				.and_then(toml::Value::as_str,)
				.unwrap_or(package_name,)
				.to_owned()
		},);

	// 頭の`poison_girl_`部分は長ったらしいので除く
	let name: String = if package_name != "poison_girl" {
		package_name.split("poison_girl_",).nth(1,)?
	} else {
		package_name
	}
	.to_string()
	.to_camel();
	Rslt::new(Manifest::new(package_name, name, bin_name,),)
}

fn detect_chart_type(
	struct_def: &syn::DeriveInput,
) -> Rslt<Option<syn::Type,>,>
{
	let syn::Data::Struct(syn::DataStruct { fields, .. },) = &struct_def.data
	else {
		return Rslt::new_err(format!("expected struct, found {struct_def:?}"),);
	};
	let ty = fields.iter().find(|f| {
		f.attrs.iter().any(
			|attr| matches!(attr.meta, syn::Meta::Path(ref p) if p.get_ident() == Some(&format_ident!("chart"))),
		)
	},).map(|f| f.ty.clone());
	Rslt::new(ty,)
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

	let fields = fields_invest(&enum_name, fields,)?;

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
		syn::Fields::Unit => Rslt::new_err("Unit type field (such as `None`)",),
	}
}

/// from method内でどのように各フィールドがassignされるかを定義する
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
					return Rslt::new_err("enum名の解決に失敗しました",);
				};
				let enum_last = &segments.last()?.ident;

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
		a => return Rslt::new_err(format!("type {a:#?} not supported"),),
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
	use {super::*, syn::parse_quote};
	#[test]

	fn test_from_path()
	{
		// Create a test enum as DeriveInput
		let test_enum: syn::DeriveInput = parse_quote! {
			pub enum TestCrate {
				Kernel,
				BootLoader
			}
		};

		// from_path_buf expects a struct, so this should return an error
		let result = from_path_buf(test_enum,);

		// Should return an error since from_path_buf expects a struct
		assert!(result.has_err());
	}
}
