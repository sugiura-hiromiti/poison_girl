use super::Boolean;

#[repr(C)]
pub struct InputKey {
	scan_code:    u16,
	unicode_char: u16,
}

#[repr(C)]
pub struct TextOutputMode {
	max_mode:       i32,
	mode:           i32,
	attribute:      i32,
	cursor_column:  i32,
	cursor_row:     i32,
	cursor_visible: Boolean,
}

#[repr(transparent)]
pub struct TextOutputModePtr {
	tom: *mut TextOutputMode,
}

unsafe impl Sync for TextOutputModePtr {}
