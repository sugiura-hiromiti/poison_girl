//! TODO: compile_errorとpanicの境界をより明確に

#[macro_export]
macro_rules! fnl {
	(
		$name:ident,
		[$(as $ty:ty,)? $(with $path:path,)?],
		$doc:literal
	)=>{
		#[proc_macro]
		#[doc = $doc]
		pub fn $name(item: proc_macro::TokenStream,)
		-> proc_macro::TokenStream
		{
			$crate::def! { $name, [ item $(as $ty,)? $(with $path,)? ], }
		}
	};
}

#[macro_export]
macro_rules! atr {
	(
		$name:ident,
		[$(as $ty:ty,)? $(with $path:path,)?],
		[$(as $ty2:ty,)? $(with $path2:path,)?],
		$doc:literal
	) => {
		#[proc_macro_attribute]
		#[doc = $doc]
		pub fn $name(
			attr: proc_macro::TokenStream,
			item: proc_macro::TokenStream,
		) -> proc_macro::TokenStream
		{
			$crate::def! { $name, [attr $(as $ty,)? $(with $path,)?], [item $(as $ty2,)? $(with $path2,)?], }
		}
	};
}

#[macro_export]
macro_rules! drv {
	(
		$derive:ident,
		$name:ident,
		[$(as $ty:ty,)? $(with $path:path,)?],
		$(attributes: $($attributes:ident,)+)?
		$doc:literal
	) => { 		#[proc_macro_derive($derive
$($(, attributes($attributes))+)?)] 		#[doc = $doc]
		pub fn $name(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
			$crate::def! { $name, [item  $(as $ty,)? $(with $path,)? ], }
		}
	};
}

/// TODO: panic!ではなくcompile errorに寄せる
#[macro_export]
macro_rules! def {
	(
		$name:ident,
		$([ $param:ident $(as $ty:ty,)? $(with $path:path,)? ],)+
	)=>{
		$(
			let $param = syn::parse_macro_input!($param $(as $ty)? $(with $path)?);
		)?

		// poison_girl_proc_macro_impl::$name::$name($($param,)+).unwrap_or_emit().into()
		let rslt = poison_girl_proc_macro_impl::$name($($param,)+);
		if let Some(err) = rslt.err() {
			let msg = format!("{err:?}");
			return match format!("compile_error!({msg:?});").parse() {
				Ok(tokens) => tokens,
				Err(_,) => proc_macro::TokenStream::new(),
			};
		}

		rslt.notation().iter().for_each(|d| match d {
			poison_girl_macro_error::diagnostic::NotationDiag::Warn(msg,) => {
				proc_macro::Diagnostic::new(proc_macro::Level::Warning, msg,).emit();
			},
			poison_girl_macro_error::diagnostic::NotationDiag::Note(msg,) => {
				proc_macro::Diagnostic::new(proc_macro::Level::Note, msg,).emit();
			},
			poison_girl_macro_error::diagnostic::NotationDiag::Help(msg,) => {
				proc_macro::Diagnostic::new(proc_macro::Level::Help, msg,).emit();
			},
		});
		match rslt.into_value() {
			Some(value,) => value.into(),
			None => {
				let msg = "proc macro returned neither value nor error";
				match format!("compile_error!({msg:?});").parse() {
					Ok(tokens) => tokens,
					Err(_,) => proc_macro::TokenStream::new(),
				}
			},
		}
	};
}
