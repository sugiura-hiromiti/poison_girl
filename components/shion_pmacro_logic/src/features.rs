use crate::RsltP;
use anyhow::Result as Rslt;
use oso_dev_util_helper::fs::all_crates;
use oso_dev_util_helper::fs::read_toml;
use oso_dev_util_helper::util::CaseConvert;
use quote::ToTokens;
use quote::format_ident;

pub fn features(
	_attr: proc_macro2::TokenStream,
	mut item: syn::ItemEnum,
) -> RsltP {
	let mut hs = std::collections::HashSet::new();
	all_crates()?
		.iter()
		.filter_map(|e| {
			let e = e.join(oso_dev_util_helper::fs::CARGO_MANIFEST,);
			read_toml(e,)
		},)
		.try_for_each(|toml| -> Rslt<(),> {
			if let Some(toml::Value::Table(t,),) = toml?.get("features",) {
				t.into_iter().for_each(|(feature, _,)| {
					hs.insert(feature.clone(),);
				},);
			}
			Ok((),)
		},)?;

	hs.iter().for_each(|variant| {
		let variant: String = variant.to_camel();
		let variant = format_ident!("{variant}");
		let variant: syn::Variant = syn::parse_quote!(#variant);
		item.variants.push(variant,);
	},);

	Ok((item.to_token_stream(), vec![],),)
}
