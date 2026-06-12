#![feature(iterator_try_collect)]
use {
	poison_girl_dev_error::{InvalidManifest, X, Y, poison_girl_err},
	poison_girl_dev_fs::{all_crates, read_toml},
	poison_girl_dev_util::case_conversion::CaseConvert,
	poison_girl_macro_error::rslt::Rslt,
	proc_macro2::TokenStream,
	quote::format_ident,
	std::{collections::HashMap, path::PathBuf},
};

/// TODO: from_path_bufマクロと共通化出来る箇所を探す
pub fn features(attr: syn::Expr, mut item: syn::ItemEnum,)
-> Rslt<TokenStream,>
{
	let crates = all_crates()?;
	let path_feature_map = crates
		.iter()
		.map(|e| {
			let manifest = e.join(poison_girl_dev_fs::CARGO_MANIFEST,);
			read_toml(manifest,).map(|m| {
				(
					e,
					m.unwrap_or(toml::map::Map::new(),)
						.remove("features",)
						.unwrap_or(toml::Value::Table(toml::map::Map::new(),),),
				)
			},)
		},)
		.try_collect::<Vec<_,>>()?
		.into_iter()
		.map(|(path, features,)| {
			let toml::Value::Table(t,) = features else {
				return Y(poison_girl_err!(InvalidManifest::new(
					"features section should be table"
				)),);
			};

			let features: Vec<_,> = t.into_iter().map(|(k, _,)| k.to_camel(),).collect();
			X((path.to_owned(), features,),)

			// t.into_iter().map(|(k, _,)| {
			// let variant: String = k.to_camel();
			// let variant = format_ident!("{variant}");
			// let variant: syn::Variant = syn::parse_quote!(#variant);
			// variant
			// },);
		},)
		// .try_for_each(|variant| {
		// 	variant?.for_each(|v| item.variants.push(v,),);
		// 	PoisonGirlB::X((),)
		// },);
		.try_collect::<HashMap<_,_>>()?;

	let mut feature_path_map: HashMap<String, Vec<PathBuf,>,> = HashMap::new();
	path_feature_map.iter().for_each(|(path, feature,)| {
		feature.clone().into_iter().for_each(|f| {
			feature_path_map.entry(f,).or_default().push(path.clone(),);
		},);
	},);
	// if true {
	// 	return Rslt::new_err("fuck ---------------------------",);
	// }

	let (path_list, features_list,): (Vec<_,>, Vec<_,>,) = path_feature_map
		.into_iter()
		.map(|(k, v,)| -> (String, Vec<syn::Ident,>,) {
			(
				k.display().to_string(),
				v.into_iter().map(|v| format_ident!("{v}"),).collect(),
			)
		},)
		.unzip();
	let (feature_list, paths_list,): (Vec<_,>, Vec<_,>,) = feature_path_map
		.into_iter()
		.map(|(k, v,)| -> (syn::Ident, Vec<String,>,) {
			(
				format_ident!("{k}"),
				v.into_iter().map(|v| v.display().to_string(),).collect(),
			)
		},)
		.unzip();

	let enum_name = item.ident.clone();
	let conversion_partner = expr_path_guard(attr,)?;
	let partner_name = last_expr_path_segment(&conversion_partner,)?;
	let from_fn_name = format_ident!("from_{partner_name}");
	let into_fn_name = format_ident!("into_{partner_name}");

	feature_list.iter().for_each(|feature| {
		let variant: syn::Variant = syn::parse_quote!(#feature);
		item.variants.push(variant,)
	},);

	// let item = item.into_token_stream();
	let def_and_impls = quote::quote! {
		#item

		impl #enum_name {
			pub fn #from_fn_name(value: #partner_name) -> Vec<Self> {
				let path_buf_value: std::path::PathBuf = value.into();
				let str_value = path_buf_value.display().to_string();
				let str_value = str_value.as_str();

				match str_value {
					#(#path_list => vec![ #(Self::#features_list,)* ],)*
					_ => panic!("invalid path given")
				}
			}

			pub fn #into_fn_name(self) -> Vec<#partner_name> {
				match self {
					#(Self::#feature_list => vec![ #(#partner_name::from(PathBuf::from(#paths_list)),)* ],)*
				}
			}
		}
	};

	Rslt::new(def_and_impls,)
}

fn expr_path_guard(should_path: syn::Expr,) -> Rslt<syn::ExprPath,>
{
	if let syn::Expr::Path(p,) = should_path {
		Rslt::new(p,)
	} else {
		Rslt::new_err("expect syn::Expr::Path. but not.",)
	}
}

fn last_expr_path_segment(expr_path: &syn::ExprPath,) -> Rslt<syn::Ident,>
{
	let syn::ExprPath { path: syn::Path { segments, .. }, .. } = expr_path;
	let last_segment = segments.last()?.ident.clone();
	Rslt::new(last_segment,)
}
