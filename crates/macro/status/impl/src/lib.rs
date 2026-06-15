#![feature(str_as_str)]
#![feature(iterator_try_collect)]

use {poison_girl_macro_error::rslt::Rslt, proc_macro2::TokenStream};

mod codegen;
mod html;
mod model;
mod source;

pub use {
	codegen::impl_status,
	html::{get_element_by_id, status_spec_page},
	model::{StatusCode, StatusCodeInfo},
};

#[cfg(test)]
pub(crate) use html::{
	get_elements_by_attribute, get_elements_by_name, status_codes_info,
	table_data, table_rows,
};

pub fn status(version: syn::Lit,) -> Rslt<TokenStream,>
{
	let syn::Lit::Float(version,) = version else {
		return Rslt::new_err(format!(
			"version is floating point literal. found {version:?}"
		),);
	};

	status_spec_page(version,).replace_by(|spec_page| {
		let c_enum_impl = impl_status(&spec_page,);
		let enum_def = quote::quote! {
			#[repr(transparent)]
			#[derive(Eq, PartialEq, Clone, Debug,)]
			pub struct Status(pub usize);

			#c_enum_impl
		};
		Rslt::new(enum_def,)
	},)
}

#[cfg(test)] mod tests;
