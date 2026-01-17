#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(transparent)]
pub struct VariableAttributes(pub u32,);

impl VariableAttributes {
	pub const APPEND_WRITE: u32 = 0x40;
	pub const ATHENTICATED_WRITE_ACCESS: u32 = 0x10;
	pub const BOOTSERVICE_ACCESS: u32 = 0x02;
	pub const ENHANCED_AUTHENTICATED_ACCESS: u32 = 0x80;
	pub const HARDWARE_ERROR_RECORD: u32 = 0x08;
	pub const NON_VOLATILE: u32 = 0x01;
	pub const RUNTIME_ACCESS: u32 = 0x04;
	pub const TIME_BASED_AUTHENTICATED_WRITE_ACCESS: u32 = 0x20;
}
