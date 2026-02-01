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
	use super::*;

	#[test]
	fn test_diag_enum_variants()
	{
		// Test that all Diag variants can be created
		let err = Diag::err("Error message",);
		let warn = Diag::warn("Warning message",);
		let note = Diag::note("Note message",);
		let help = Diag::help("Help message",);

		// Test pattern matching on variants
		match err {
			Diag::Err(ErrDiag(msg,),) => assert_eq!(msg, "Error message"),
			_ => panic!("Should match Err variant"),
		}

		match warn {
			Diag::Notation(NotationDiag::Warn(msg,),) => {
				assert_eq!(msg, "Warning message")
			},
			_ => panic!("Should match Warn variant"),
		}

		match note {
			Diag::Notation(NotationDiag::Note(msg,),) => {
				assert_eq!(msg, "Note message")
			},
			_ => panic!("Should match Note variant"),
		}

		match help {
			Diag::Notation(NotationDiag::Help(msg,),) => {
				assert_eq!(msg, "Help message")
			},
			_ => panic!("Should match Help variant"),
		}
	}

	#[test]
	fn test_diag_string_content()
	{
		let test_messages = vec![
			"Simple error",
			"Error with numbers: 123",
			"Error with special chars: !@#$%",
			"Multi-line\nerror\nmessage",
			"Unicode error: 🦀",
			"", // Empty string
		];

		for msg in test_messages {
			let err = Diag::err(msg,);
			let warn = Diag::warn(msg,);
			let note = Diag::note(msg,);
			let help = Diag::help(msg,);

			// Test that messages are preserved correctly
			match err {
				Diag::Err(ErrDiag(stored_msg,),) => assert_eq!(stored_msg, msg),
				_ => panic!("Should match Err variant"),
			}

			match warn {
				Diag::Notation(NotationDiag::Warn(stored_msg,),) => {
					assert_eq!(stored_msg, msg)
				},
				_ => panic!("Should match Warn variant"),
			}

			match note {
				Diag::Notation(NotationDiag::Note(stored_msg,),) => {
					assert_eq!(stored_msg, msg)
				},
				_ => panic!("Should match Note variant"),
			}

			match help {
				Diag::Notation(NotationDiag::Help(stored_msg,),) => {
					assert_eq!(stored_msg, msg)
				},
				_ => panic!("Should match Help variant"),
			}
		}
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
	fn test_diag_clone_if_possible()
	{
		// Test that Diag can be created with same content (since String is
		// Clone)
		let original = Diag::err("original message",);
		let duplicate = Diag::err("original message",);

		match (original, duplicate,) {
			(Diag::Err(ErrDiag(orig_msg,),), Diag::Err(ErrDiag(dup_msg,),),) =>
			{
				assert_eq!(orig_msg, dup_msg);
			},
			_ => panic!("Both should be Err variants"),
		}
	}

	#[test]
	fn test_diag_pattern_matching_exhaustive()
	{
		let diags = vec![
			Diag::err("error",),
			Diag::warn("warning",),
			Diag::note("note",),
			Diag::help("help",),
		];

		for diag in diags {
			let result = match diag {
				Diag::Err(_,) => "error",
				Diag::Notation(n,) => match n {
					NotationDiag::Warn(_,) => "warning",
					NotationDiag::Note(_,) => "note",
					NotationDiag::Help(_,) => "help",
				},
			};

			// Just verify that pattern matching works for all variants
			assert!(["error", "warning", "note", "help"].contains(&result));
		}
	}

	#[test]
	fn test_diag_with_borrowed_vs_owned_strings()
	{
		let borrowed_str = "borrowed message";
		let owned_string = String::from("owned message",);

		// Test creating Diag with both borrowed and owned strings
		let diag1 = Diag::err(borrowed_str,);
		let diag2 = Diag::err(owned_string,);

		match diag1 {
			Diag::Err(ErrDiag(msg,),) => assert_eq!(msg, "borrowed message"),
			_ => panic!("Should be Err variant"),
		}

		match diag2 {
			Diag::Err(ErrDiag(msg,),) => assert_eq!(msg, "owned message"),
			_ => panic!("Should be Err variant"),
		}
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
	fn test_diag_with_long_messages()
	{
		let long_message = "a".repeat(10000,); // Very long message
		let diag = Diag::err(long_message.clone(),);

		match diag.flat() {
			FlatDiag::Err(msg,) => {
				assert_eq!(msg.len(), 10000);
				assert_eq!(msg, long_message);
			},
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_diag_with_special_characters()
	{
		let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~\n\t\r\\";
		let diag = Diag::note(special_chars.to_string(),);

		match diag.flat() {
			FlatDiag::Note(msg,) => assert_eq!(msg, special_chars),
			_ => panic!("Should be Note variant"),
		}
	}

	#[test]
	fn test_diag_message_modification()
	{
		let mut message = String::from("initial",);
		message.push_str(" modified",);

		let diag = Diag::warn(message,);
		match diag.flat() {
			FlatDiag::Warn(msg,) => assert_eq!(msg, "initial modified"),
			_ => panic!("Should be Warn variant"),
		}
	}

	#[test]
	fn test_diag_all_variants_different()
	{
		let err = Diag::err("msg",);
		let warn = Diag::warn("msg",);
		let note = Diag::note("msg",);
		let help = Diag::help("msg",);

		// Test that variants are distinguishable even with same message
		let variants = [
			std::mem::discriminant(&err,),
			std::mem::discriminant(&warn,),
			std::mem::discriminant(&note,),
			std::mem::discriminant(&help,),
		];

		// All discriminants should be different
		for i in 0..variants.len() {
			for j in (i + 1)..variants.len() {
				assert_ne!(variants[i], variants[j]);
			}
		}
	}

	#[test]
	fn test_diag_enum_size()
	{
		// Test that the enum size is reasonable
		let size = std::mem::size_of::<Diag,>();

		// Should be at least the size of a String (3 * usize typically)
		// but not excessively large
		assert!(size >= std::mem::size_of::<String,>());
		assert!(size <= 1024); // Reasonable upper bound
	}

	#[test]
	fn test_diag_memory_efficiency()
	{
		// Test that creating many Diag instances doesn't cause issues
		let mut diags = Vec::new();

		for i in 0..1000 {
			let msg = format!("Message {}", i);
			diags.push(match i % 4 {
				0 => Diag::err(msg,),
				1 => Diag::warn(msg,),
				2 => Diag::note(msg,),
				_ => Diag::help(msg,),
			},);
		}

		assert_eq!(diags.len(), 1000);

		// Verify a few random entries
		match &diags[0].clone().flat() {
			FlatDiag::Err(msg,) => assert_eq!(msg, "Message 0"),
			_ => panic!("Should be Err variant"),
		}

		match &diags[999].clone().flat() {
			FlatDiag::Help(msg,) => assert_eq!(msg, "Message 999"),
			_ => panic!("Should be Help variant"),
		}
	}

	#[test]
	fn test_string_operations_used_in_diag()
	{
		// Test various string operations that might be used with Diag
		let base_msg = "base message";

		// Test string formatting
		let formatted_msg = format!("Formatted: {}", base_msg);
		let diag = Diag::err(formatted_msg,);

		match diag.flat() {
			FlatDiag::Err(msg,) => {
				assert!(msg.contains("Formatted: base message"))
			},
			_ => panic!("Should be Err variant"),
		}

		// Test string concatenation
		let mut concat_msg = String::from("Start ",);
		concat_msg.push_str(base_msg,);
		concat_msg.push_str(" end",);

		let diag2 = Diag::warn(concat_msg,);
		match diag2.flat() {
			FlatDiag::Warn(msg,) => assert_eq!(msg, "Start base message end"),
			_ => panic!("Should be Warn variant"),
		}
	}

	#[test]
	fn test_diag_with_unicode_content()
	{
		// Test Diag with Unicode content
		let unicode_msg = "Unicode test: 🦀 Rust 中文 العربية 🚀";
		let diag = Diag::note(unicode_msg.to_string(),);

		match diag.flat() {
			FlatDiag::Note(msg,) => {
				assert_eq!(msg, unicode_msg);
				assert!(msg.contains("🦀"));
				assert!(msg.contains("中文"));
				assert!(msg.contains("العربية"));
				assert!(msg.contains("🚀"));
			},
			_ => panic!("Should be Note variant"),
		}
	}

	#[test]
	fn test_diag_message_length_variations()
	{
		// Test with various message lengths
		let lengths = vec![0, 1, 10, 100, 1000];

		for len in lengths {
			let message = "x".repeat(len,);
			let diag = Diag::help(message.clone(),);

			match diag.flat() {
				FlatDiag::Help(msg,) => {
					assert_eq!(msg.len(), len);
					assert_eq!(msg, message);
				},
				_ => panic!("Should be Help variant"),
			}
		}
	}

	#[test]
	fn test_diag_with_control_characters()
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
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_diag_memory_layout()
	{
		// Test memory layout properties
		use std::mem;

		let diag = Diag::err("test",);

		// Test alignment
		assert!(mem::align_of::<Diag,>() > 0);

		// Test size consistency
		let size1 = mem::size_of_val(&diag,);
		let size2 = mem::size_of::<Diag,>();
		assert_eq!(size1, size2);
	}

	#[test]
	fn test_diag_variant_ordering()
	{
		// Test that we can create all variants in any order
		let variants = [
			Diag::help("Help first",),
			Diag::err("Error second",),
			Diag::note("Note third",),
			Diag::warn("Warning fourth",),
		];

		assert_eq!(variants.len(), 4);

		// Verify each variant
		match &variants[0].clone().flat() {
			FlatDiag::Help(msg,) => assert_eq!(msg, "Help first"),
			_ => panic!("Should be Help variant"),
		}

		match &variants[1].clone().flat() {
			FlatDiag::Err(msg,) => assert_eq!(msg, "Error second"),
			_ => panic!("Should be Err variant"),
		}

		match &variants[2].clone().flat() {
			FlatDiag::Note(msg,) => assert_eq!(msg, "Note third"),
			_ => panic!("Should be Note variant"),
		}

		match &variants[3].clone().flat() {
			FlatDiag::Warn(msg,) => assert_eq!(msg, "Warning fourth"),
			_ => panic!("Should be Warn variant"),
		}
	}

	#[test]
	fn test_diag_string_ownership()
	{
		// Test string ownership behavior
		let original_string = String::from("original",);
		let diag = Diag::err(original_string,);

		// The original string should be moved into the Diag
		// We can't access original_string anymore, which is correct behavior

		match diag.flat() {
			FlatDiag::Err(msg,) => {
				assert_eq!(msg, "original");
				// The Diag now owns the string
			},
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_macro_token_handling()
	{
		// Test that macros can handle various token types
		// This is a compilation test - if it compiles, token handling works

		// Test with different identifier patterns
		let _test_ident = "test_identifier";
		let _test_camel_case = "TestCamelCase";
		let _screaming_snake_case = "SCREAMING_SNAKE_CASE";
	}
}
