#![feature(str_as_str)]
#![feature(iter_array_chunks)]
#![feature(iterator_try_collect)]
#![feature(try_trait_v2)]

pub mod diagnostic;
pub mod rslt;

#[macro_export]
macro_rules! fnl {
	($name:ident => $ty:ty, $doc:literal) => {
		#[proc_macro]
		#[doc = $doc]
		pub fn $name(
			item: proc_macro::TokenStream,
		) -> proc_macro::TokenStream {
			$crate::def! { $name, item => $ty, }
		}
	};
}

#[macro_export]
macro_rules! atr {
	($name:ident => $ty:ty, $ty2:ty, $doc:literal) => {
		#[proc_macro_attribute]
		#[doc = $doc]
		pub fn $name(
			attr: proc_macro::TokenStream,
			item: proc_macro::TokenStream,
		) -> proc_macro::TokenStream {
			$crate::def! { $name, attr => $ty, item => $ty2, }
		}
	};
}

#[macro_export]
macro_rules! drv {
	($derive:ident, $name:ident => $ty:ty, $(attributes: $($attributes:ident,)+)? $doc:literal) => {
		#[proc_macro_derive($derive $($(, attributes($attributes))+)?)]
		#[doc = $doc]
		pub fn $name(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
			$crate::def! { $name, item => $ty, }

		}
	};
}

#[macro_export]
macro_rules! def {
	($name:ident, $($param:ident => $ty:ty,)+)=>{
		$(
			let $param = syn::parse_macro_input!($param as $ty);
		)?

		poison_girl_proc_macro_impl::$name::$name($($param,)+).unwrap_or_emit().into()
	};
}
