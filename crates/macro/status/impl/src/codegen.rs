use {
	crate::model::{StatusCode, StatusCodeInfo},
	proc_macro2::Span,
};

trait TokenParts
{
	fn token_parts(
		&self,
		is_err: bool,
	) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream,),>;
}

impl TokenParts for Vec<StatusCodeInfo,>
{
	fn token_parts(
		&self,
		is_err: bool,
	) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream,),>
	{
		self.iter()
			.map(|sci| {
				// Create identifier from the status code mnemonic
				let mnemonic =
					syn::Ident::new(&sci.mnemonic, Span::call_site(),);

				// Create literal from the status code value
				let value = syn::Lit::Int(syn::LitInt::new(
					&format!("{}", sci.value),
					Span::call_site(),
				),);

				// Generate appropriate match arm based on error status
				let match_arms = if is_err {
					err_match(&mnemonic, &sci.desc,)
				} else {
					ok_match(&mnemonic,)
				};

				// Generate associated constant with documentation
				let assoc = assoc_const(&mnemonic, &value, &sci.desc,);

				(match_arms, assoc,)
			},)
			.collect()
	}
}

pub fn impl_status(spec_page: &StatusCode,) -> proc_macro2::TokenStream
{
	// Generate token parts for success status codes (non-error)
	let (success_match, success_assoc,): (Vec<_,>, Vec<_,>,) =
		spec_page.success.token_parts(false,).into_iter().unzip();

	// Generate token parts for warning status codes (non-error)
	let (warn_match, warn_assoc,): (Vec<_,>, Vec<_,>,) =
		spec_page.warn.token_parts(false,).into_iter().unzip();

	// Generate token parts for error status codes (error)
	let (error_match, error_assoc,): (Vec<_,>, Vec<_,>,) =
		spec_page.error.token_parts(true,).into_iter().unzip();

	quote::quote! {
		impl Status {
			// Associated constants for all status codes
			#(#success_assoc)*
			#(#warn_assoc)*
			#(#error_assoc)*

			/// Converts the status to a Result type.
			///
			/// Returns Ok(Self) for success and warning status codes,
			/// and Err(UefiError) for error status codes.
			pub fn x_or(self) -> poison_girl_no_std_error::PoisonGirlB<Self> {
				use alloc::string::ToString;
				match self {
					// Success status codes return Ok
					#(#success_match)*
					// Warning status codes return Ok
					#(#warn_match)*
					// Error status codes return Err
					#(#error_match)*
					// Unknown status codes return custom error
					Self(code) => poison_girl_no_std_error::Y(poison_girl_no_std_error::poison_girl_err!(poison_girl_no_std_error::UefiError::CustomStatus(code))),
				}
			}

			/// Converts the status to a Result with custom transformation.
			///
			/// Similar to ok_or(), but allows applying a transformation function
			/// to the success value before returning.
			pub fn x_or_with<T>(self, with: impl FnOnce(Self) -> T) -> poison_girl_no_std_error::PoisonGirlB<T,> {
				let status = self.x_or()?;
				poison_girl_no_std_error::X(with(status))
			}
		}
	}
}

fn ok_match(mnemonic: &syn::Ident,) -> proc_macro2::TokenStream
{
	quote::quote! {
		Self::#mnemonic => poison_girl_no_std_error::X(Self::#mnemonic,),
	}
}

fn err_match(mnemonic: &syn::Ident, msg: &String,) -> proc_macro2::TokenStream
{
	let mnemonic_str = mnemonic.to_string();
	quote::quote! {
	Self::#mnemonic => {
		let mut mnemonic = concat!(#mnemonic_str, ": ", #msg);
		poison_girl_no_std_error::Y(poison_girl_no_std_error::poison_girl_err!(poison_girl_no_std_error::UefiError::Status(mnemonic)))
	},
	}
}

fn assoc_const(
	mnemonic: &syn::Ident,
	value: &syn::Lit,
	msg: &String,
) -> proc_macro2::TokenStream
{
	quote::quote! {
		#[doc = #msg]
		pub const #mnemonic: Self = Self(#value);
	}
}
