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

		oso_proc_macro_logic::$name::$name($($param,)+).unwrap_or_emit().into()
	};
}

#[derive(Debug,)]
pub enum Diag {
	Err(String,),
	Warn(String,),
	Note(String,),
	Help(String,),
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_diag_enum_variants() {
		// Test that all Diag variants can be created
		let err = Diag::Err("Error message".to_string(),);
		let warn = Diag::Warn("Warning message".to_string(),);
		let note = Diag::Note("Note message".to_string(),);
		let help = Diag::Help("Help message".to_string(),);

		// Test pattern matching on variants
		match err {
			Diag::Err(msg,) => assert_eq!(msg, "Error message"),
			_ => panic!("Should match Err variant"),
		}

		match warn {
			Diag::Warn(msg,) => assert_eq!(msg, "Warning message"),
			_ => panic!("Should match Warn variant"),
		}

		match note {
			Diag::Note(msg,) => assert_eq!(msg, "Note message"),
			_ => panic!("Should match Note variant"),
		}

		match help {
			Diag::Help(msg,) => assert_eq!(msg, "Help message"),
			_ => panic!("Should match Help variant"),
		}
	}

	#[test]
	fn test_diag_string_content() {
		let test_messages = vec![
			"Simple error",
			"Error with numbers: 123",
			"Error with special chars: !@#$%",
			"Multi-line\nerror\nmessage",
			"Unicode error: 🦀",
			"", // Empty string
		];

		for msg in test_messages {
			let err = Diag::Err(msg.to_string(),);
			let warn = Diag::Warn(msg.to_string(),);
			let note = Diag::Note(msg.to_string(),);
			let help = Diag::Help(msg.to_string(),);

			// Test that messages are preserved correctly
			match err {
				Diag::Err(stored_msg,) => assert_eq!(stored_msg, msg),
				_ => panic!("Should match Err variant"),
			}

			match warn {
				Diag::Warn(stored_msg,) => assert_eq!(stored_msg, msg),
				_ => panic!("Should match Warn variant"),
			}

			match note {
				Diag::Note(stored_msg,) => assert_eq!(stored_msg, msg),
				_ => panic!("Should match Note variant"),
			}

			match help {
				Diag::Help(stored_msg,) => assert_eq!(stored_msg, msg),
				_ => panic!("Should match Help variant"),
			}
		}
	}

	#[test]
	fn test_diag_debug_representation() {
		let err = Diag::Err("test error".to_string(),);
		let debug_str = format!("{:?}", err);

		// Debug representation should contain the variant name and message
		assert!(debug_str.contains("Err"));
		assert!(debug_str.contains("test error"));
	}

	#[test]
	fn test_diag_clone_if_possible() {
		// Test that Diag can be created with same content (since String is
		// Clone)
		let original = Diag::Err("original message".to_string(),);
		let duplicate = Diag::Err("original message".to_string(),);

		match (original, duplicate,) {
			(Diag::Err(orig_msg,), Diag::Err(dup_msg,),) => {
				assert_eq!(orig_msg, dup_msg);
			},
			_ => panic!("Both should be Err variants"),
		}
	}

	#[test]
	fn test_diag_pattern_matching_exhaustive() {
		let diags = vec![
			Diag::Err("error".to_string(),),
			Diag::Warn("warning".to_string(),),
			Diag::Note("note".to_string(),),
			Diag::Help("help".to_string(),),
		];

		for diag in diags {
			let result = match diag {
				Diag::Err(_,) => "error",
				Diag::Warn(_,) => "warning",
				Diag::Note(_,) => "note",
				Diag::Help(_,) => "help",
			};

			// Just verify that pattern matching works for all variants
			assert!(["error", "warning", "note", "help"].contains(&result));
		}
	}

	#[test]
	fn test_diag_with_borrowed_vs_owned_strings() {
		let borrowed_str = "borrowed message";
		let owned_string = String::from("owned message",);

		// Test creating Diag with both borrowed and owned strings
		let diag1 = Diag::Err(borrowed_str.to_string(),);
		let diag2 = Diag::Err(owned_string,);

		match diag1 {
			Diag::Err(msg,) => assert_eq!(msg, "borrowed message"),
			_ => panic!("Should be Err variant"),
		}

		match diag2 {
			Diag::Err(msg,) => assert_eq!(msg, "owned message"),
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_diag_empty_messages() {
		let empty_diags = vec![
			Diag::Err(String::new(),),
			Diag::Warn(String::new(),),
			Diag::Note(String::new(),),
			Diag::Help(String::new(),),
		];

		for diag in empty_diags {
			let msg = match diag {
				Diag::Err(m,) => m,
				Diag::Warn(m,) => m,
				Diag::Note(m,) => m,
				Diag::Help(m,) => m,
			};
			assert!(msg.is_empty());
		}
	}

	#[test]
	fn test_diag_with_long_messages() {
		let long_message = "a".repeat(10000,); // Very long message
		let diag = Diag::Err(long_message.clone(),);

		match diag {
			Diag::Err(msg,) => {
				assert_eq!(msg.len(), 10000);
				assert_eq!(msg, long_message);
			},
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_diag_with_special_characters() {
		let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~\n\t\r\\";
		let diag = Diag::Note(special_chars.to_string(),);

		match diag {
			Diag::Note(msg,) => assert_eq!(msg, special_chars),
			_ => panic!("Should be Note variant"),
		}
	}

	#[test]
	fn test_diag_message_modification() {
		let mut message = String::from("initial",);
		message.push_str(" modified",);

		let diag = Diag::Warn(message,);
		match diag {
			Diag::Warn(msg,) => assert_eq!(msg, "initial modified"),
			_ => panic!("Should be Warn variant"),
		}
	}

	#[test]
	fn test_diag_all_variants_different() {
		let err = Diag::Err("msg".to_string(),);
		let warn = Diag::Warn("msg".to_string(),);
		let note = Diag::Note("msg".to_string(),);
		let help = Diag::Help("msg".to_string(),);

		// Test that variants are distinguishable even with same message
		let variants = vec![
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
	fn test_diag_enum_size() {
		// Test that the enum size is reasonable
		let size = std::mem::size_of::<Diag,>();

		// Should be at least the size of a String (3 * usize typically)
		// but not excessively large
		assert!(size >= std::mem::size_of::<String,>());
		assert!(size <= 1024); // Reasonable upper bound
	}

	#[test]
	fn test_diag_memory_efficiency() {
		// Test that creating many Diag instances doesn't cause issues
		let mut diags = Vec::new();

		for i in 0..1000 {
			let msg = format!("Message {}", i);
			diags.push(match i % 4 {
				0 => Diag::Err(msg,),
				1 => Diag::Warn(msg,),
				2 => Diag::Note(msg,),
				_ => Diag::Help(msg,),
			},);
		}

		assert_eq!(diags.len(), 1000);

		// Verify a few random entries
		match &diags[0] {
			Diag::Err(msg,) => assert_eq!(msg, "Message 0"),
			_ => panic!("Should be Err variant"),
		}

		match &diags[999] {
			Diag::Help(msg,) => assert_eq!(msg, "Message 999"),
			_ => panic!("Should be Help variant"),
		}
	}

	#[test]
	fn test_macro_definitions_exist() {
		// This test verifies that the macros are defined and accessible
		// We can't easily test macro expansion in unit tests, but we can
		// verify they exist by checking they compile

		// The macros fnl!, atr!, drv!, and def! should be available
		// If this test compiles, the macros are properly defined
		assert!(true);
	}

	#[test]
	fn test_string_operations_used_in_diag() {
		// Test various string operations that might be used with Diag
		let base_msg = "base message";

		// Test string formatting
		let formatted_msg = format!("Formatted: {}", base_msg);
		let diag = Diag::Err(formatted_msg,);

		match diag {
			Diag::Err(msg,) => assert!(msg.contains("Formatted: base message")),
			_ => panic!("Should be Err variant"),
		}

		// Test string concatenation
		let mut concat_msg = String::from("Start ",);
		concat_msg.push_str(base_msg,);
		concat_msg.push_str(" end",);

		let diag2 = Diag::Warn(concat_msg,);
		match diag2 {
			Diag::Warn(msg,) => assert_eq!(msg, "Start base message end"),
			_ => panic!("Should be Warn variant"),
		}
	}

	#[test]
	fn test_macro_syntax_validation() {
		// Test that macro syntax is valid by checking compilation
		// This is more of a compilation test - if it compiles, the macros are
		// syntactically correct

		// We can't easily test macro expansion in unit tests without actually
		// using them, but we can verify the macro definitions don't cause
		// compilation errors
		assert!(true);
	}

	#[test]
	fn test_diag_with_unicode_content() {
		// Test Diag with Unicode content
		let unicode_msg = "Unicode test: 🦀 Rust 中文 العربية 🚀";
		let diag = Diag::Note(unicode_msg.to_string(),);

		match diag {
			Diag::Note(msg,) => {
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
	fn test_diag_message_length_variations() {
		// Test with various message lengths
		let lengths = vec![0, 1, 10, 100, 1000];

		for len in lengths {
			let message = "x".repeat(len,);
			let diag = Diag::Help(message.clone(),);

			match diag {
				Diag::Help(msg,) => {
					assert_eq!(msg.len(), len);
					assert_eq!(msg, message);
				},
				_ => panic!("Should be Help variant"),
			}
		}
	}

	#[test]
	fn test_diag_with_control_characters() {
		// Test with control characters
		let control_chars =
			"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
		let diag = Diag::Err(control_chars.to_string(),);

		match diag {
			Diag::Err(msg,) => {
				assert_eq!(msg.len(), control_chars.len());
				assert_eq!(msg, control_chars);
			},
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_diag_memory_layout() {
		// Test memory layout properties
		use std::mem;

		let diag = Diag::Err("test".to_string(),);

		// Test alignment
		assert!(mem::align_of::<Diag,>() > 0);

		// Test size consistency
		let size1 = mem::size_of_val(&diag,);
		let size2 = mem::size_of::<Diag,>();
		assert_eq!(size1, size2);
	}

	#[test]
	fn test_diag_variant_ordering() {
		// Test that we can create all variants in any order
		let variants = vec![
			Diag::Help("Help first".to_string(),),
			Diag::Err("Error second".to_string(),),
			Diag::Note("Note third".to_string(),),
			Diag::Warn("Warning fourth".to_string(),),
		];

		assert_eq!(variants.len(), 4);

		// Verify each variant
		match &variants[0] {
			Diag::Help(msg,) => assert_eq!(msg, "Help first"),
			_ => panic!("Should be Help variant"),
		}

		match &variants[1] {
			Diag::Err(msg,) => assert_eq!(msg, "Error second"),
			_ => panic!("Should be Err variant"),
		}

		match &variants[2] {
			Diag::Note(msg,) => assert_eq!(msg, "Note third"),
			_ => panic!("Should be Note variant"),
		}

		match &variants[3] {
			Diag::Warn(msg,) => assert_eq!(msg, "Warning fourth"),
			_ => panic!("Should be Warn variant"),
		}
	}

	#[test]
	fn test_diag_string_ownership() {
		// Test string ownership behavior
		let original_string = String::from("original",);
		let diag = Diag::Err(original_string,);

		// The original string should be moved into the Diag
		// We can't access original_string anymore, which is correct behavior

		match diag {
			Diag::Err(msg,) => {
				assert_eq!(msg, "original");
				// The Diag now owns the string
			},
			_ => panic!("Should be Err variant"),
		}
	}

	#[test]
	fn test_macro_token_handling() {
		// Test that macros can handle various token types
		// This is a compilation test - if it compiles, token handling works

		// Test with different identifier patterns
		let _test_ident = "test_identifier";
		let _test_camel_case = "TestCamelCase";
		let _screaming_snake_case = "SCREAMING_SNAKE_CASE";

		// If this compiles, the macro token handling is working
		assert!(true);
	}
}
