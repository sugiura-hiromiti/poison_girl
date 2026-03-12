#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_features as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::atr,
};

atr! {
	features,
	[as proc_macro2::TokenStream,],
	[as syn::ItemEnum,],
	r#""#
}
