use crate::into_null_terminated_utf16;
use crate::raw::types::Boolean;
use crate::raw::types::Status;
use crate::raw::types::text::InputKey;
use crate::raw::types::text::TextOutputModePtr;
use core::ffi::c_void;
use oso_error::Rslt;
use oso_error::loader::UefiError;

#[repr(C)]
pub struct TextInputProtocol {
	reset: unsafe extern "efiapi" fn(
		this: *mut Self,
		extended_verif: Boolean,
	) -> Status,
	read_key_stroke: unsafe extern "efiapi" fn(
		this: *mut Self,
		key: *const InputKey,
	) -> Status,
	wait_for_key:    *mut c_void,
}

#[repr(C)]
pub struct TextOutputProtocol {
	reset: unsafe extern "efiapi" fn(
		this: *mut Self,
		extended_verif: Boolean,
	) -> Status,
	output: unsafe extern "efiapi" fn(
		this: *mut Self,
		string: *const u16,
	) -> Status,
	test: unsafe extern "efiapi" fn(
		this: *mut Self,
		string: *const u16,
	) -> Status,
	query_mode: unsafe extern "efiapi" fn(
		this: *mut Self,
		mode_number: usize,
		columns: *mut usize,
		rows: *mut usize,
	),
	set_mode: unsafe extern "efiapi" fn(
		this: *mut Self,
		mode_number: usize,
	) -> Status,
	set_attr:
		unsafe extern "efiapi" fn(this: *mut Self, attribute: usize,) -> Status,
	clear:         unsafe extern "efiapi" fn(this: *mut Self,) -> Status,
	set_cursor: unsafe extern "efiapi" fn(
		this: *mut Self,
		column: usize,
		row: usize,
	) -> Status,
	enable_cursor:
		unsafe extern "efiapi" fn(this: *mut Self, visible: Boolean,) -> Status,
	mode:          TextOutputModePtr,
}

impl TextOutputProtocol {
	/// # Params
	///
	/// this function expects `s` to be encoded as utf8
	pub fn output(&mut self, s: impl AsRef<str,>,) -> Rslt<Status, UefiError,> {
		let utf16_repr = into_null_terminated_utf16(s,);
		let utf16_repr = utf16_repr.as_ptr();
		unsafe { (self.output)(self, utf16_repr,) }.ok_or()
	}

	/// wrapper function of `(TextOutputProtocol.test)(ptr_of_u16)` call
	pub fn test(&mut self, s: impl AsRef<str,>,) -> bool {
		let utf16_repr = into_null_terminated_utf16(s,);
		let utf16_repr = utf16_repr.as_ptr();
		unsafe { (self.test)(self, utf16_repr,) }.is_success()
	}

	pub fn clear(&mut self,) -> Rslt<Status, UefiError,> {
		unsafe { (self.clear)(self,) }.ok_or()
	}
}
