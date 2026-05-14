use quote::quote;

#[derive(Debug, PartialEq, Eq, Clone,)]
pub enum Diag
{
	Err(ErrDiag,),
	Notation(NotationDiag,),
}

impl Diag
{
	pub fn err(s: impl Into<String,>,) -> Self
	{
		Self::Err(ErrDiag(s.into(),),)
	}

	pub fn warn(s: impl Into<String,>,) -> Self
	{
		Self::Notation(NotationDiag::Warn(s.into(),),)
	}

	pub fn note(s: impl Into<String,>,) -> Self
	{
		Self::Notation(NotationDiag::Note(s.into(),),)
	}

	pub fn help(s: impl Into<String,>,) -> Self
	{
		Self::Notation(NotationDiag::Help(s.into(),),)
	}

	pub fn flat(self,) -> FlatDiag
	{
		match self {
			Self::Err(ErrDiag(msg,),) => FlatDiag::Err(msg,),
			Self::Notation(notation_diag,) => match notation_diag {
				NotationDiag::Warn(msg,) => FlatDiag::Warn(msg,),
				NotationDiag::Note(msg,) => FlatDiag::Note(msg,),
				NotationDiag::Help(msg,) => FlatDiag::Help(msg,),
			},
		}
	}
}

pub enum FlatDiag
{
	Err(String,),
	Warn(String,),
	Note(String,),
	Help(String,),
}

impl FlatDiag
{
	pub fn err(s: impl Into<String,>,) -> Self
	{
		Self::Err(s.into(),)
	}

	pub fn warn(s: impl Into<String,>,) -> Self
	{
		Self::Warn(s.into(),)
	}

	pub fn note(s: impl Into<String,>,) -> Self
	{
		Self::Note(s.into(),)
	}

	pub fn help(s: impl Into<String,>,) -> Self
	{
		Self::Help(s.into(),)
	}

	pub fn deflat(self,) -> Diag
	{
		match self {
			Self::Err(msg,) => Diag::Err(ErrDiag(msg,),),
			Self::Warn(msg,) => Diag::Notation(NotationDiag::Warn(msg,),),
			Self::Note(msg,) => Diag::Notation(NotationDiag::Note(msg,),),
			Self::Help(msg,) => Diag::Notation(NotationDiag::Help(msg,),),
		}
	}
}

impl From<FlatDiag,> for Diag
{
	fn from(value: FlatDiag,) -> Self
	{
		value.deflat()
	}
}

impl From<Diag,> for FlatDiag
{
	fn from(value: Diag,) -> Self
	{
		value.flat()
	}
}

#[derive(Debug, PartialEq, Eq, Clone,)]
pub struct ErrDiag(String,);

impl ErrDiag
{
	pub fn new(s: impl Into<String,>,) -> Self
	{
		Self(s.into(),)
	}
}

impl quote::ToTokens for ErrDiag
{
	#[cold]
	#[track_caller]
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream,)
	{
		let Self(s,) = self;
		*tokens = quote! {
			compile_error!(#s)
		};
	}
}

#[derive(Debug, PartialEq, Eq, Clone,)]
pub enum NotationDiag
{
	Warn(String,),
	Note(String,),
	Help(String,),
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::{fail, rslt::test_helper::*, success},
	};

	#[test]
	fn test_diag_enum_variants() -> TestRslt
	{
		// Test that all Diag variants can be created
		let err = Diag::err("Error message",);
		let warn = Diag::warn("Warning message",);
		let note = Diag::note("Note message",);
		let help = Diag::help("Help message",);

		// Test pattern matching on variants
		match err {
			Diag::Err(ErrDiag(msg,),) => assert_eq!(msg, "Error message"),
			_ => fail!("Should match Err variant"),
		}

		match warn {
			Diag::Notation(NotationDiag::Warn(msg,),) => {
				assert_eq!(msg, "Warning message")
			},
			_ => fail!("Should match Warn variant"),
		}

		match note {
			Diag::Notation(NotationDiag::Note(msg,),) => {
				assert_eq!(msg, "Note message")
			},
			_ => fail!("Should match Note variant"),
		}

		match help {
			Diag::Notation(NotationDiag::Help(msg,),) => {
				assert_eq!(msg, "Help message")
			},
			_ => fail!("Should match Help variant"),
		}

		success!()
	}

	#[test]
	fn test_diag_debug_representation()
	{
		let err = Diag::err("test error",);
		let debug_str = format!("{:?}", err);

		// Debug representation should contain the variant name and message
		assert!(debug_str.contains("Err"));
		assert!(debug_str.contains("test error"));
	}

	#[test]
	fn test_diag_empty_messages()
	{
		let empty_diags = vec![
			Diag::err(String::new(),),
			Diag::warn(String::new(),),
			Diag::note(String::new(),),
			Diag::help(String::new(),),
		];

		for diag in empty_diags {
			let msg = match diag.flat() {
				FlatDiag::Err(m,) => m,
				FlatDiag::Warn(m,) => m,
				FlatDiag::Note(m,) => m,
				FlatDiag::Help(m,) => m,
			};
			assert!(msg.is_empty());
		}
	}

	#[test]
	fn test_diag_with_control_characters() -> TestRslt
	{
		// Test with control characters
		let control_chars =
			"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
		let diag = Diag::err(control_chars,);

		match diag.flat() {
			FlatDiag::Err(msg,) => {
				assert_eq!(msg.len(), control_chars.len());
				assert_eq!(msg, control_chars);
			},
			_ => fail!("Should be Err variant"),
		}

		success!()
	}
}
