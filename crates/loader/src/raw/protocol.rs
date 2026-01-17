use super::types::UnsafeHandle;

pub mod device_path;
pub mod file;
pub mod graphic;
pub mod text;

#[derive(Debug,)]
#[repr(C)]
pub struct OpenProtocolInformationEntry {
	pub agent_handle:      UnsafeHandle,
	pub controller_handle: UnsafeHandle,
	pub attributes:        u32,
	pub open_count:        u32,
}
