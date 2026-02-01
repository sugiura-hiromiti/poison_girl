use {
	poison_girl_dev_error::ReShape,
	poison_girl_dev_fs::{
		fs::{all_crates, read_toml},
		util::CaseConvert,
	},
	poison_girl_proc_macro_helper::rslt::Rslt,
	proc_macro2::TokenStream,
	quote::{ToTokens, format_ident},
};

pub fn features(
	_attr: TokenStream,
	mut item: syn::ItemEnum,
) -> Rslt<TokenStream,>
{
	let mut hs = std::collections::HashSet::new();
	all_crates()?
		.iter()
		.filter_map(|e| {
			let e = e.join(poison_girl_dev_fs::fs::CARGO_MANIFEST,);
			read_toml(e,).reshape((),)
		},)
		.for_each(|toml| {
			if let Some(toml::Value::Table(t,),) = toml.get("features",) {
				t.into_iter().for_each(|(feature, _,)| {
					hs.insert(feature.clone(),);
				},);
			}
		},);

	hs.iter().for_each(|variant| {
		let variant: String = variant.to_camel();
		let variant = format_ident!("{variant}");
		let variant: syn::Variant = syn::parse_quote!(#variant);
		item.variants.push(variant,);
	},);

	Rslt::new(item.to_token_stream(),)
	// Ok((item.to_token_stream(), vec![],),)
}
